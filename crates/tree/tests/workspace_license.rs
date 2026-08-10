//! Every workspace member takes the workspace license (#34, entry 1).
//!
//! The root manifest names the license once and every member reaches it by
//! writing `license.workspace = true`. A member that forgets the line carries no
//! license at all, and the failure that produces is quiet in both directions:
//! the crate builds, and the dependency gate reads an unlicensed package in the
//! graph rather than an unlicensed package in this tree, which is a different
//! sentence than the one a reader would draw from a green run.
//!
//! While the gate skipped this workspace's own crates the line was optional and
//! nothing could tell a member that had it from one that did not. `deny.toml`
//! stopped skipping them in the same change this case landed in, so the missing
//! line now reddens the dependency gate as well. This case is the cheaper of the
//! two: it names the member, it runs in the default suite, and it does not need
//! the advisory database.
//!
//! The subject is what git stores rather than the working directory, for the
//! same reason the guards beside this one give: the manifest somebody else
//! checks out is the one in the index.
//!
//! A checkout with no git available reds this case rather than passing it.

use std::path::{Path, PathBuf};
use std::process::Command;

use messlatte_tree::{takes_workspace_license, workspace_members};

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
fn every_workspace_member_takes_the_workspace_license() {
    let root = workspace_root();
    let members = workspace_members(&staged(&root, "Cargo.toml")).expect("the members list reads");

    let missing: Vec<&String> = members
        .iter()
        .filter(|member| !takes_workspace_license(&staged(&root, &format!("{member}/Cargo.toml"))))
        .collect();

    assert!(
        missing.is_empty(),
        "these members do not write `license.workspace = true`, so they carry no license and \
         the one named in the root manifest does not reach them: {missing:?}"
    );
}

#[test]
fn the_root_manifest_names_a_license_for_them_to_take() {
    // Without this, emptying the workspace field would leave every member
    // pointing at nothing and the case above still green: `license.workspace =
    // true` is a reference, and a reference to an absent key is not a license.
    let root = workspace_root();
    let manifest = staged(&root, "Cargo.toml");
    let named = manifest
        .lines()
        .map(str::trim)
        .any(|line| line.starts_with("license = \""));

    assert!(
        named,
        "the root manifest declares no `license`, so every member taking the workspace license \
         takes nothing"
    );
}

#[test]
fn a_member_with_no_package_table_does_not_take_the_license() {
    assert!(!takes_workspace_license("[lints]\nworkspace = true\n"));
}

#[test]
fn a_member_saying_nothing_about_a_license_does_not_take_it() {
    assert!(!takes_workspace_license(
        "[package]\nname = \"x\"\nversion.workspace = true\n"
    ));
}

#[test]
fn a_member_spelling_the_identifier_out_does_not_take_the_workspace_license() {
    // The near-miss, and the one somebody will actually write. It satisfies the
    // dependency gate and it is a second copy of an answer that lives in the
    // root manifest, so the two come apart the first time either is edited.
    assert!(!takes_workspace_license(
        "[package]\nname = \"x\"\nlicense = \"AGPL-3.0-or-later\"\n"
    ));
}

#[test]
fn a_member_declining_the_workspace_license_does_not_take_it() {
    assert!(!takes_workspace_license(
        "[package]\nlicense.workspace = false\n"
    ));
}

#[test]
fn a_member_pointing_at_a_license_file_does_not_take_the_workspace_license() {
    // `license-file` is the other key cargo accepts here, and it names a path
    // rather than an identifier. A member using it is outside the one-place rule
    // this case exists for, whatever that file turns out to hold.
    assert!(!takes_workspace_license(
        "[package]\nlicense-file = \"LICENSE\"\n"
    ));
}

#[test]
fn a_later_table_ends_the_package_table() {
    // A key of this name under another table is a different key, and reading it
    // as this one would pass a member whose `[package]` never said anything.
    assert!(!takes_workspace_license(
        "[package]\nname = \"x\"\n\n[dependencies]\nlicense.workspace = true\n"
    ));
}

#[test]
fn a_package_table_taking_the_workspace_license_takes_it() {
    assert!(takes_workspace_license(
        "[package]\nname = \"x\"\nlicense.workspace = true\n"
    ));
}

#[test]
fn a_trailing_comment_does_not_change_the_answer() {
    assert!(takes_workspace_license(
        "[package]\nlicense.workspace = true # the one in the root manifest\n"
    ));
}
