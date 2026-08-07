//! The workspace layout, derived from the manifests and the lock file and
//! judged against the declaration in `layout.toml`.
//!
//! The declaration says which role each workspace member carries and which
//! roles a role may depend on. This crate derives what the tree actually
//! declares and returns a refusal for every disagreement. It decides nothing on
//! its own: every rule it applies is a line somebody wrote in `layout.toml`.
//!
//! What it does not see, written here rather than left for somebody to
//! discover. It reads `[dependencies]`, `[dev-dependencies]` and
//! `[build-dependencies]` and no target-conditional table, so a dependency
//! declared under a `[target.'cfg(...)'.dependencies]` heading is invisible to
//! it. It keys a dependency by its table key, so a dependency renamed with
//! `package = "..."` is invisible to it as well. Both are gaps in the
//! derivation rather than in the rule.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

/// A dependency of one crate on another, both named by package name.
pub type Edge = (String, String);

/// What the layout declaration says.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Declaration {
    /// A role, and the set of roles it may depend on.
    pub roles: BTreeMap<String, BTreeSet<String>>,
    /// A workspace member, and the role it carries.
    pub crates: BTreeMap<String, String>,
}

/// What the tree says.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graph {
    pub members: BTreeSet<String>,
    pub manifest_edges: BTreeSet<Edge>,
    pub lock_edges: BTreeSet<Edge>,
}

/// Which derivation carries an edge the other one does not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Manifests,
    Lock,
}

/// One disagreement between the declaration and the tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Refusal {
    /// A workspace member the declaration does not place.
    UndeclaredCrate { krate: String },
    /// A crate the declaration places that is not a workspace member.
    DeclaredCrateAbsent { krate: String },
    /// A crate placed in a role the declaration never defines.
    UnknownRole { krate: String, role: String },
    /// A dependency the declaration does not allow.
    ForbiddenEdge {
        from: String,
        from_role: String,
        to: String,
        to_role: String,
    },
    /// An edge one derivation carries and the other does not.
    LockDisagrees {
        from: String,
        to: String,
        side: Side,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::UndeclaredCrate { krate } => {
                write!(f, "workspace member {krate} carries no role in the layout declaration")
            }
            Refusal::DeclaredCrateAbsent { krate } => write!(
                f,
                "the layout declaration places {krate}, which is not a workspace member"
            ),
            Refusal::UnknownRole { krate, role } => write!(
                f,
                "{krate} is placed in role {role}, which the layout declaration does not define"
            ),
            Refusal::ForbiddenEdge {
                from,
                from_role,
                to,
                to_role,
            } => write!(
                f,
                "{from} in role {from_role} depends on {to} in role {to_role}, and {from_role} may not depend on {to_role}"
            ),
            Refusal::LockDisagrees { from, to, side } => match side {
                Side::Manifests => write!(
                    f,
                    "{from} depends on {to} in the manifests and not in the lock file"
                ),
                Side::Lock => write!(
                    f,
                    "{from} depends on {to} in the lock file and not in the manifests"
                ),
            },
        }
    }
}

/// Every disagreement between a declaration and a graph, sorted and without
/// duplicates. An empty result is the only passing result.
pub fn refuse(declaration: &Declaration, graph: &Graph) -> Vec<Refusal> {
    let mut out: Vec<Refusal> = Vec::new();

    for member in &graph.members {
        if !declaration.crates.contains_key(member) {
            out.push(Refusal::UndeclaredCrate {
                krate: member.clone(),
            });
        }
    }

    for krate in declaration.crates.keys() {
        if !graph.members.contains(krate) {
            out.push(Refusal::DeclaredCrateAbsent {
                krate: krate.clone(),
            });
        }
    }

    for (krate, role) in &declaration.crates {
        if !declaration.roles.contains_key(role) {
            out.push(Refusal::UnknownRole {
                krate: krate.clone(),
                role: role.clone(),
            });
        }
    }

    for (from, to) in graph.manifest_edges.union(&graph.lock_edges) {
        let (Some(from_role), Some(to_role)) =
            (declaration.crates.get(from), declaration.crates.get(to))
        else {
            continue;
        };
        let Some(allowed) = declaration.roles.get(from_role) else {
            continue;
        };
        if !allowed.contains(to_role) {
            out.push(Refusal::ForbiddenEdge {
                from: from.clone(),
                from_role: from_role.clone(),
                to: to.clone(),
                to_role: to_role.clone(),
            });
        }
    }

    for (from, to) in graph.manifest_edges.difference(&graph.lock_edges) {
        out.push(Refusal::LockDisagrees {
            from: from.clone(),
            to: to.clone(),
            side: Side::Manifests,
        });
    }
    for (from, to) in graph.lock_edges.difference(&graph.manifest_edges) {
        out.push(Refusal::LockDisagrees {
            from: from.clone(),
            to: to.clone(),
            side: Side::Lock,
        });
    }

    out.sort();
    out.dedup();
    out
}

/// Read a layout declaration out of TOML text.
pub fn declaration_from_str(text: &str) -> Result<Declaration, String> {
    let value: toml::Table = text
        .parse()
        .map_err(|e| format!("the layout declaration is not TOML: {e}"))?;

    let mut declaration = Declaration::default();

    let roles = value
        .get("roles")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "the layout declaration has no roles table".to_string())?;
    for (role, allowed) in roles {
        let list = allowed
            .as_array()
            .ok_or_else(|| format!("role {role} is not a list of roles"))?;
        let mut set = BTreeSet::new();
        for entry in list {
            let name = entry
                .as_str()
                .ok_or_else(|| format!("role {role} lists something that is not a role name"))?;
            set.insert(name.to_string());
        }
        declaration.roles.insert(role.clone(), set);
    }

    let crates = value
        .get("crates")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "the layout declaration has no crates table".to_string())?;
    for (krate, role) in crates {
        let role = role
            .as_str()
            .ok_or_else(|| format!("crate {krate} is placed in something that is not a role"))?;
        declaration.crates.insert(krate.clone(), role.to_string());
    }

    Ok(declaration)
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn parse(path: &Path) -> Result<toml::Table, String> {
    read(path)?
        .parse()
        .map_err(|e| format!("{} is not TOML: {e}", path.display()))
}

/// Derive the graph of a real workspace from its manifests and its lock file.
pub fn graph_from_workspace(root: &Path) -> Result<Graph, String> {
    let root_manifest = parse(&root.join("Cargo.toml"))?;
    let members = root_manifest
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .ok_or_else(|| "the workspace manifest lists no members".to_string())?;

    let mut declared: Vec<(String, Vec<String>)> = Vec::new();
    for member in members {
        let relative = member
            .as_str()
            .ok_or_else(|| "a workspace member is not a path".to_string())?;
        let manifest = parse(&root.join(relative).join("Cargo.toml"))?;
        let name = manifest
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| format!("the manifest at {relative} names no package"))?
            .to_string();
        let mut dependencies = Vec::new();
        for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(table) = manifest.get(kind).and_then(|v| v.as_table()) {
                dependencies.extend(table.keys().cloned());
            }
        }
        declared.push((name, dependencies));
    }

    let names: BTreeSet<String> = declared.iter().map(|(name, _)| name.clone()).collect();

    let mut manifest_edges = BTreeSet::new();
    for (name, dependencies) in &declared {
        for dependency in dependencies {
            if names.contains(dependency) {
                manifest_edges.insert((name.clone(), dependency.clone()));
            }
        }
    }

    let lock = parse(&root.join("Cargo.lock"))?;
    let mut lock_edges = BTreeSet::new();
    if let Some(packages) = lock.get("package").and_then(|p| p.as_array()) {
        for package in packages {
            let Some(name) = package.get("name").and_then(|n| n.as_str()) else {
                continue;
            };
            if !names.contains(name) {
                continue;
            }
            let Some(dependencies) = package.get("dependencies").and_then(|d| d.as_array()) else {
                continue;
            };
            for dependency in dependencies {
                let Some(entry) = dependency.as_str() else {
                    continue;
                };
                let Some(dependency) = entry.split_whitespace().next() else {
                    continue;
                };
                if names.contains(dependency) {
                    lock_edges.insert((name.to_string(), dependency.to_string()));
                }
            }
        }
    }

    Ok(Graph {
        members: names,
        manifest_edges,
        lock_edges,
    })
}
