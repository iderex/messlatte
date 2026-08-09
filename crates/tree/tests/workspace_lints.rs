//! Every workspace member takes the workspace lint set (#83).
//!
//! The root manifest forbids `unsafe_code` and denies the security-relevant
//! clippy lints beside it, and a crate reaches that set only by writing
//! `[lints]` with `workspace = true` in its own manifest. A member that forgets
//! the line is not linted at a slightly weaker level; it is outside the set
//! entirely, and `forbid` cannot refuse what was never applied to it. Nothing
//! about a fresh crate makes anybody notice, because the crate compiles, the
//! gate is green, and the missing line is three words in a file nobody rereads.
//!
//! The subject is what git stores rather than the working directory, for the
//! same reason the two guards beside this one give: the manifest somebody else
//! checks out is the one in the index.
//!
//! A checkout with no git available reds this case rather than passing it.

use std::path::{Path, PathBuf};
use std::process::Command;

use messlatte_tree::{takes_workspace_lints, workspace_members};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

/// One tracked file, as git holds it in the index.
fn staged(root: &Path, path: &str) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", &format!(":{path}")])
        .output()
        .expect("git is on the path and the workspace root is a git checkout");
    assert!(
        output.status.success(),
        "git could not read {path} out of the index: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    String::from_utf8(output.stdout).expect("a manifest is text")
}

#[test]
fn every_workspace_member_takes_the_workspace_lint_set() {
    let root = workspace_root();
    let members = workspace_members(&staged(&root, "Cargo.toml")).expect("the members list reads");

    let missing: Vec<&String> = members
        .iter()
        .filter(|member| !takes_workspace_lints(&staged(&root, &format!("{member}/Cargo.toml"))))
        .collect();

    assert!(
        missing.is_empty(),
        "these members do not write `[lints]` with `workspace = true`, so the workspace lint \
         set including the forbidden unsafe code does not reach them: {missing:?}"
    );
}

#[test]
fn the_members_this_runs_against_are_not_a_short_list() {
    // Without this, a manifest whose members list stopped being readable the way
    // `workspace_members` reads it would leave the case above judging one member
    // or none, and a green result would mean the guard lost most of its subject.
    let root = workspace_root();
    let members = workspace_members(&staged(&root, "Cargo.toml")).expect("the members list reads");

    let tracked = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "--", "crates/"])
        .output()
        .expect("git is on the path and the workspace root is a git checkout");
    let manifests = String::from_utf8_lossy(&tracked.stdout)
        .lines()
        .filter(|path| path.ends_with("/Cargo.toml"))
        .count();

    assert_eq!(
        members.len(),
        manifests,
        "the manifest declares {} members and git tracks {manifests} crate manifests under \
         crates/, so one of them is outside the workspace and outside its lint set",
        members.len()
    );
}

#[test]
fn a_member_with_no_lints_table_does_not_take_the_set() {
    assert!(!takes_workspace_lints(
        "[package]\nname = \"x\"\n\n[dependencies]\n"
    ));
}

#[test]
fn a_member_declaring_its_own_lints_does_not_take_the_set() {
    // The near-miss. `[lints.rust]` is a crate choosing its own levels, which
    // reads almost the same and is the opposite of inheriting the workspace's.
    assert!(!takes_workspace_lints(
        "[lints.rust]\nunsafe_code = \"allow\"\n"
    ));
}

#[test]
fn a_lints_table_that_declines_the_workspace_does_not_take_the_set() {
    assert!(!takes_workspace_lints("[lints]\nworkspace = false\n"));
}

#[test]
fn a_lints_table_taking_the_workspace_takes_the_set() {
    assert!(takes_workspace_lints(
        "[package]\nname = \"x\"\n\n[lints]\nworkspace = true\n"
    ));
}

#[test]
fn a_later_table_ends_the_lints_table() {
    // `workspace = true` under `[dependencies]` is a different key with a
    // different meaning, and reading it as this one would pass a crate that
    // never opted in.
    assert!(!takes_workspace_lints(
        "[lints]\n\n[dependencies]\nserde = { workspace = true }\n"
    ));
}

#[test]
fn the_members_list_is_the_quoted_paths_it_holds() {
    let manifest =
        "[workspace]\nresolver = \"2\"\nmembers = [\n  \"crates/a\",\n  \"crates/b\",\n]\n";
    assert_eq!(
        workspace_members(manifest),
        Ok(vec!["crates/a".to_string(), "crates/b".to_string()])
    );
}

#[test]
fn a_manifest_with_no_members_list_is_an_error_and_not_an_empty_list() {
    // Folding this into an empty list would make the case above pass against a
    // manifest it could not read, which is the shape these guards refuse.
    assert!(workspace_members("[package]\nname = \"x\"\n").is_err());
}

#[test]
fn a_members_list_holding_nothing_is_an_error() {
    assert!(workspace_members("[workspace]\nmembers = []\n").is_err());
}
