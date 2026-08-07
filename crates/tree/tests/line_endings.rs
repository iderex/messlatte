//! No tracked text file carries a carriage return.
//!
//! The subject is what git stores, not what is in the working directory. On
//! Windows a working copy legitimately holds carriage returns, and the byte
//! that matters is the one in the index and in the tree, because that is what a
//! hash is taken over and what somebody else checks out.
//!
//! The judgement is `git grep --cached`, which reads the staged blobs and skips
//! what git detects as binary. It is a subprocess rather than a walk of the
//! filesystem because the filesystem is the wrong copy, and because the binary
//! detection is git's and should stay git's.
//!
//! A checkout with no git available reds this test rather than passing it. A
//! guard that goes quiet when it cannot run reports exactly like a clean tree.

use std::path::{Path, PathBuf};
use std::process::Command;

use messlatte_tree::offenders;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

#[test]
fn no_tracked_text_file_carries_a_carriage_return() {
    let root = workspace_root();
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["grep", "--cached", "-I", "-l", "-e", "\r", "--"])
        .arg(".")
        .output()
        .expect("git is on the path and the workspace root is a git checkout");

    let found = offenders(
        output.status.code(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
    .expect("git grep was understood");

    assert!(
        found.is_empty(),
        "these tracked text files carry a carriage return in the index: {found:?}"
    );
}

#[test]
fn the_tree_this_runs_against_is_not_empty() {
    // Without this, the case above would pass just as happily against a
    // directory git tracks nothing in, and a green result would mean the guard
    // never had a subject.
    let root = workspace_root();
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files"])
        .output()
        .expect("git is on the path and the workspace root is a git checkout");

    let tracked = String::from_utf8_lossy(&output.stdout).lines().count();
    assert!(
        tracked > 1,
        "git tracks {tracked} files here, so the carriage-return case has nothing to judge"
    );
}
