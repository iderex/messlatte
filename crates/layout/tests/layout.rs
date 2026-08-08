//! What the layout check refuses, and what it does not.
//!
//! Every fixture below is a fixture vocabulary. It is deliberately not this
//! repository's own crate names, because a case built out of the real tree
//! proves the state of the tree on the day it ran rather than proving the
//! guard. The one case that does read the real tree is the first, and all it
//! claims is that the tree passes today.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use messlatte_layout::{declaration_from_str, graph_from_workspace, refuse, Graph, Refusal, Side};
use messlatte_suites::Scratch;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

/// A vocabulary of six single-letter crates, one per role. The roles carry the
/// three directions that matter: a method may not reach the generator, a scorer
/// may not reach a method, and nothing below the binary may reach the plotting
/// layer.
const FIXTURE: &str = r#"
[roles]
units = []
generator = ["units"]
method = ["units"]
scoring = ["units"]
plotting = ["units"]
binary = ["units", "generator", "method", "scoring", "plotting"]

[crates]
u = "units"
g = "generator"
m = "method"
s = "scoring"
p = "plotting"
b = "binary"
"#;

/// Every edge the fixture declaration allows and the fixture graph carries
/// before a case adds anything. `b` does not yet reach `p`, which leaves one
/// allowed edge available as a one-change neighbour.
const BASE: &[(&str, &str)] = &[
    ("g", "u"),
    ("m", "u"),
    ("s", "u"),
    ("p", "u"),
    ("b", "g"),
    ("b", "m"),
    ("b", "s"),
];

fn edges(extra: &[(&str, &str)]) -> BTreeSet<(String, String)> {
    BASE.iter()
        .chain(extra.iter())
        .map(|(from, to)| ((*from).to_string(), (*to).to_string()))
        .collect()
}

/// The fixture graph with `extra` added to both derivations, so a case about
/// forbidden edges is not also a case about a stale lock file.
fn graph(extra: &[(&str, &str)]) -> Graph {
    Graph {
        members: ["u", "g", "m", "s", "p", "b"]
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        manifest_edges: edges(extra),
        lock_edges: edges(extra),
    }
}

fn forbidden(from: &str, from_role: &str, to: &str, to_role: &str) -> Refusal {
    Refusal::ForbiddenEdge {
        from: from.to_string(),
        from_role: from_role.to_string(),
        to: to.to_string(),
        to_role: to_role.to_string(),
    }
}

fn check(extra: &[(&str, &str)]) -> Vec<Refusal> {
    refuse(
        &declaration_from_str(FIXTURE).expect("the fixture parses"),
        &graph(extra),
    )
}

#[test]
fn the_workspace_as_it_stands_is_refused_by_nothing() {
    let root = workspace_root();
    let declaration = declaration_from_str(
        &std::fs::read_to_string(root.join("layout.toml")).expect("layout.toml is readable"),
    )
    .expect("layout.toml parses");
    let graph = graph_from_workspace(&root).expect("the workspace graph derives");

    assert!(
        !graph.members.is_empty(),
        "the derivation found no workspace members, so a passing verdict would mean nothing"
    );
    assert!(
        !graph.manifest_edges.is_empty(),
        "the derivation found no edges, so a passing verdict would mean nothing"
    );

    let refusals = refuse(&declaration, &graph);
    assert_eq!(refusals, Vec::new(), "{refusals:?}");
}

#[test]
fn the_fixture_workspace_is_refused_by_nothing() {
    assert_eq!(check(&[]), Vec::new());
}

#[test]
fn one_allowed_edge_added_to_the_fixture_is_refused_by_nothing() {
    assert_eq!(check(&[("b", "p")]), Vec::new());
}

#[test]
fn a_method_reaching_the_generator_is_refused() {
    assert_eq!(
        check(&[("m", "g")]),
        vec![forbidden("m", "method", "g", "generator")]
    );
}

#[test]
fn a_scorer_reaching_a_method_is_refused() {
    assert_eq!(
        check(&[("s", "m")]),
        vec![forbidden("s", "scoring", "m", "method")]
    );
}

#[test]
fn anything_below_the_binary_reaching_the_plotting_layer_is_refused() {
    assert_eq!(
        check(&[("s", "p")]),
        vec![forbidden("s", "scoring", "p", "plotting")]
    );
}

#[test]
fn a_workspace_member_the_declaration_does_not_place_is_refused() {
    let mut graph = graph(&[]);
    graph.members.insert("x".to_string());

    assert_eq!(
        refuse(&declaration_from_str(FIXTURE).unwrap(), &graph),
        vec![Refusal::UndeclaredCrate {
            krate: "x".to_string()
        }]
    );
}

#[test]
fn a_declared_crate_that_is_not_a_workspace_member_is_refused() {
    let mut graph = graph(&[]);
    graph.members.remove("p");

    assert_eq!(
        refuse(&declaration_from_str(FIXTURE).unwrap(), &graph),
        vec![Refusal::DeclaredCrateAbsent {
            krate: "p".to_string()
        }]
    );
}

#[test]
fn a_role_the_declaration_does_not_define_is_refused() {
    // The near-miss this case is for is a single letter: `methods` where the
    // roles table defines `method`. It matters more than a typo usually does,
    // because an edge leaving a crate whose role is undefined is not judged at
    // all. `m` reaching the generator would pass unnoticed under this
    // declaration, and the only thing standing between that and a green verdict
    // is the refusal below.
    let declaration = declaration_from_str(&FIXTURE.replace("m = \"method\"", "m = \"methods\""))
        .expect("the fixture parses");

    assert_eq!(
        refuse(&declaration, &graph(&[])),
        vec![
            Refusal::UnknownRole {
                krate: "m".to_string(),
                role: "methods".to_string()
            },
            // Edges into the mis-placed crate are still judged, and `binary` is
            // not allowed to depend on a role that does not exist.
            forbidden("b", "binary", "m", "methods"),
        ]
    );

    // Stated as its own assertion rather than left implicit: the edge out of
    // the mis-placed crate is invisible to the edge rule.
    let with_forbidden_edge = refuse(&declaration, &graph(&[("m", "g")]));
    assert!(
        !with_forbidden_edge.contains(&forbidden("m", "methods", "g", "generator")),
        "the edge rule judged an edge out of a crate whose role is undefined"
    );
}

#[test]
fn an_edge_in_the_manifests_and_not_in_the_lock_file_is_refused() {
    let mut graph = graph(&[]);
    graph
        .manifest_edges
        .insert(("b".to_string(), "p".to_string()));

    assert_eq!(
        refuse(&declaration_from_str(FIXTURE).unwrap(), &graph),
        vec![Refusal::LockDisagrees {
            from: "b".to_string(),
            to: "p".to_string(),
            side: Side::Manifests
        }]
    );
}

#[test]
fn an_edge_in_the_lock_file_and_not_in_the_manifests_is_refused() {
    let mut graph = graph(&[]);
    graph.lock_edges.insert(("b".to_string(), "p".to_string()));

    assert_eq!(
        refuse(&declaration_from_str(FIXTURE).unwrap(), &graph),
        vec![Refusal::LockDisagrees {
            from: "b".to_string(),
            to: "p".to_string(),
            side: Side::Lock
        }]
    );
}

/// Write a two-crate workspace into a scratch directory and return it. The
/// method crate reaches the generator crate under `heading`, so the caller
/// chooses which dependency table carries the forbidden edge.
///
/// The caller keeps the returned value for as long as it reads the workspace,
/// because the directory is removed when it is dropped. It is a `Scratch`
/// rather than a path under the temporary directory chosen here, which is what
/// #14 requires of a default-suite case and also what keeps two runs of this
/// suite at once from writing over each other: the previous form named one
/// fixed directory per case and emptied it on the way in.
fn scratch_workspace(label: &str, heading: &str) -> Scratch {
    let scratch = Scratch::new(label).expect("a scratch directory can be made");
    scratch
        .write(
            "Cargo.toml",
            b"[workspace]\nresolver = \"2\"\nmembers = [\"g\", \"m\"]\n",
        )
        .unwrap();
    scratch
        .write(
            "g/Cargo.toml",
            b"[package]\nname = \"g\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
    scratch
        .write(
            "m/Cargo.toml",
            format!("[package]\nname = \"m\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[{heading}]\ng = {{ path = \"../g\" }}\n").as_bytes(),
        )
        .unwrap();
    scratch.write("g/src/lib.rs", b"").unwrap();
    scratch.write("m/src/lib.rs", b"").unwrap();
    scratch
        .write(
            "Cargo.lock",
            b"version = 4\n\n[[package]]\nname = \"g\"\nversion = \"0.0.0\"\n\n[[package]]\nname = \"m\"\nversion = \"0.0.0\"\ndependencies = [\n \"g\",\n]\n",
        )
        .unwrap();
    scratch
}

#[test]
fn a_forbidden_edge_declared_only_as_a_development_dependency_is_still_refused() {
    // A method reaching the generator from its own tests is the same import
    // with a smaller blast radius, so the derivation reads dev-dependencies
    // into the same edge set. This case is about the derivation and not about
    // the rule, so it builds a workspace on disk rather than a graph in memory.
    let workspace = scratch_workspace("layout-dev-dependency", "dev-dependencies");
    let graph = graph_from_workspace(workspace.path()).expect("the scratch workspace derives");

    assert_eq!(
        graph.manifest_edges,
        [("m".to_string(), "g".to_string())].into_iter().collect(),
        "the derivation did not read the dev-dependency table"
    );

    let declaration = declaration_from_str(
        "[roles]\ngenerator = []\nmethod = []\n\n[crates]\ng = \"generator\"\nm = \"method\"\n",
    )
    .unwrap();

    assert_eq!(
        refuse(&declaration, &graph),
        vec![forbidden("m", "method", "g", "generator")]
    );
}

#[test]
fn a_normal_dependency_is_read_by_the_same_derivation() {
    // The neighbour of the case above: one word different in the manifest, the
    // same edge derived, so the previous case is about the heading and not
    // about something else the derivation happens to do.
    let workspace = scratch_workspace("layout-normal-dependency", "dependencies");
    let graph = graph_from_workspace(workspace.path()).expect("the scratch workspace derives");

    assert_eq!(
        graph.manifest_edges,
        [("m".to_string(), "g".to_string())].into_iter().collect()
    );
}

// What the guard says, as opposed to what it decides (#99).
//
// Every case above compares refusal values, and a value carries no sentence. The
// string is what a person meets when the gate goes red, and a message naming the
// wrong field sends that person to the wrong file while every case above stays
// green. So each arm below is rendered from a refusal the guard actually
// produced, and the assertion is on the whole sentence rather than on a fragment
// of it: a fragment passes for an arm that dropped the rest of the clause.
//
// The fixture crates are single letters and the roles are words, which is what
// makes a swapped field visible. `m` and `g` in a sentence that has both of them
// twice cannot be transposed without changing the string.

/// The one refusal `refusals` holds, rendered. More than one means the case set
/// up something it did not mean to.
fn only_message(refusals: &[Refusal]) -> String {
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    refusals[0].to_string()
}

#[test]
fn the_message_for_an_undeclared_workspace_member_names_the_member() {
    let mut graph = graph(&[]);
    graph.members.insert("x".to_string());

    assert_eq!(
        only_message(&refuse(&declaration_from_str(FIXTURE).unwrap(), &graph)),
        "workspace member x carries no role in the layout declaration"
    );
}

#[test]
fn the_message_for_a_declared_crate_that_is_absent_says_which_side_is_missing() {
    // The near-miss is this arm and the one above swapped. Both are about one
    // crate name and a declaration, and a reader who is told the wrong direction
    // edits the wrong file: one is repaired by adding a line to `layout.toml`,
    // the other by removing one. The two sentences are asserted in full so that
    // an arm rendering its neighbour's words fails here.
    let mut graph = graph(&[]);
    graph.members.remove("p");

    assert_eq!(
        only_message(&refuse(&declaration_from_str(FIXTURE).unwrap(), &graph)),
        "the layout declaration places p, which is not a workspace member"
    );
}

#[test]
fn the_message_for_an_unknown_role_names_the_crate_and_the_role() {
    let declaration = declaration_from_str(&FIXTURE.replace("m = \"method\"", "m = \"methods\""))
        .expect("the fixture parses");
    let refusals = refuse(&declaration, &graph(&[]));

    let messages: Vec<String> = refusals.iter().map(ToString::to_string).collect();
    assert_eq!(
        messages,
        vec![
            "m is placed in role methods, which the layout declaration does not define".to_string(),
            "b in role binary depends on m in role methods, and binary may not depend on methods"
                .to_string(),
        ]
    );
}

#[test]
fn the_message_for_a_forbidden_edge_names_both_ends_and_both_roles() {
    // Every field of this arm appears in the sentence, and two of them appear
    // twice. A rendering that reads the dependency backwards, which is the arm's
    // one plausible mistake, cannot produce this string.
    assert_eq!(
        only_message(&check(&[("m", "g")])),
        "m in role method depends on g in role generator, and method may not depend on generator"
    );
}

#[test]
fn the_two_directions_of_a_lock_disagreement_name_different_files() {
    // The arm the issue names as the one to watch. Its two directions differ by
    // which file is named and by nothing else, so a copied-and-not-edited second
    // direction is invisible to every case that compares values: `Side` still
    // distinguishes them and the string no longer does. Both sentences are
    // asserted, and then asserted to differ, because two identical strings would
    // satisfy either assertion alone.
    let mut in_the_manifests = graph(&[]);
    in_the_manifests
        .manifest_edges
        .insert(("b".to_string(), "p".to_string()));

    let mut in_the_lock = graph(&[]);
    in_the_lock
        .lock_edges
        .insert(("b".to_string(), "p".to_string()));

    let declaration = declaration_from_str(FIXTURE).unwrap();
    let from_the_manifests = only_message(&refuse(&declaration, &in_the_manifests));
    let from_the_lock = only_message(&refuse(&declaration, &in_the_lock));

    assert_eq!(
        from_the_manifests,
        "b depends on p in the manifests and not in the lock file"
    );
    assert_eq!(
        from_the_lock,
        "b depends on p in the lock file and not in the manifests"
    );
    assert_ne!(from_the_manifests, from_the_lock);
}
