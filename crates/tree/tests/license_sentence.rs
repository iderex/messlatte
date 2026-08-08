//! No tracked file denies the license while the license is tracked.
//!
//! The sentence in question is the one built in [`stale_sentence`] below, and it
//! is not written out anywhere in this file for the reason given there. It sat in
//! the header of the sign-off gate, it was true when it was written, it stopped
//! being true when the license landed, and nothing noticed: no check here reads a
//! comment, so a header describing a tree the repository has left behind stays
//! green on every route. It was found by reading, which is the route this case
//! replaces.
//!
//! The subject is what git stores rather than the working directory, for the
//! same reason the carriage-return case beside it gives: the bytes somebody else
//! checks out are the ones in the index.
//!
//! The condition is deliberately two-sided. A repository with no license file
//! may say so, and this case says nothing about one. It refuses the combination:
//! `LICENSE` tracked and the sentence still in the tree.
//!
//! A checkout with no git available reds this case rather than passing it.

use std::path::{Path, PathBuf};
use std::process::Command;

use messlatte_tree::offenders;

/// The sentence this case refuses, held in pieces rather than as one literal.
///
/// Written whole, this file would be the first thing the search below found, and
/// the guard would refuse its own source. Two halves joined at run time reach the
/// reader unchanged and reach the index as something the pattern does not match.
fn stale_sentence() -> String {
    ["this repository has no ", "license file"].concat()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

fn git(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git is on the path and the workspace root is a git checkout")
}

fn license_is_tracked(root: &Path) -> bool {
    let output = git(root, &["ls-files", "--error-unmatch", "--", "LICENSE"]);
    output.status.success()
}

#[test]
fn no_tracked_file_denies_the_license_that_is_tracked() {
    let root = workspace_root();
    assert!(
        license_is_tracked(&root),
        "LICENSE is not tracked, so this case has nothing to judge and must not pass quietly"
    );

    let needle = stale_sentence();
    let output = git(
        &root,
        &[
            "grep", "--cached", "-I", "-l", "-F", "-e", &needle, "--", ".",
        ],
    );

    let found = offenders(
        output.status.code(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
    .expect("git grep was understood");

    assert!(
        found.is_empty(),
        "LICENSE is tracked and these tracked files still say it is not: {found:?}"
    );
}
