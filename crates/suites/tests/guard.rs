//! What the default-suite guard refuses, and what the tree it guards says
//! today.
//!
//! Every forbidden shape below is assembled from two pieces at run time. Written
//! whole, it would sit in a tracked file under a `tests/` directory, which is
//! exactly what the guard reads, and this file would become its own first
//! finding. The bytes handed to the guard are the same either way, and the
//! assembly is what keeps them out of the tree.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use messlatte_suites::{breaches, is_default_suite_test, Breach};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

/// A forbidden shape, in the two halves this file holds it as.
fn shape(head: &str, tail: &str) -> String {
    format!("{head}{tail}")
}

/// A case that binds a socket, as a source file rather than as a running test.
fn a_case_that_binds() -> String {
    format!(
        "#[test]\nfn it_talks() {{\n    let _ = {}(\"127.0.0.1:0\");\n}}\n",
        shape("TcpListener", "::bind")
    )
}

/// A case that writes through the standard entry point rather than through the
/// scratch directory.
fn a_case_that_writes() -> String {
    format!(
        "#[test]\nfn it_saves() {{\n    {}\"out.txt\", b\"x\").unwrap();\n}}\n",
        shape("fs::write", "(")
    )
}

#[test]
fn a_default_suite_case_that_binds_a_socket_is_refused() {
    let found = breaches("crates/x/tests/net.rs", &a_case_that_binds());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        matches!(found[0], Breach::BindsASocket { line: 3, .. }),
        "{found:?}"
    );
}

#[test]
fn a_default_suite_case_that_writes_through_the_standard_entry_point_is_refused() {
    let found = breaches("crates/x/tests/files.rs", &a_case_that_writes());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        matches!(
            found[0],
            Breach::WritesOutsideItsTemporaryDirectory { line: 3, .. }
        ),
        "{found:?}"
    );
}

#[test]
fn a_unit_test_inside_src_is_a_default_suite_case_too() {
    // The near-miss. A guard that judged only the `tests/` directories would be
    // green on this file, and a unit test is where somebody reaches for a
    // socket first, because it is the shortest thing to write.
    let source = format!(
        "{}\nmod cases {{\n{}}}\n",
        concat!("#[cfg(", "test)]"),
        a_case_that_binds()
    );
    assert!(is_default_suite_test("crates/x/src/lib.rs", &source));
    assert_eq!(breaches("crates/x/src/lib.rs", &source).len(), 1);
}

#[test]
fn a_source_file_that_is_not_a_case_is_not_judged() {
    // The product code is where these calls belong. A guard that refused them
    // everywhere would be a guard about the program rather than about the
    // suite, and it would be turned off.
    let source = a_case_that_binds();
    assert!(!is_default_suite_test("crates/x/src/net.rs", &source));
    assert!(breaches("crates/x/src/net.rs", &source).is_empty());
}

#[test]
fn a_file_that_is_not_rust_is_not_judged() {
    // A fixture, a document or a lock file under `tests/` is not a case, and
    // reading one as a case would make the guard fire on data it cannot judge.
    let source = a_case_that_binds();
    assert!(!is_default_suite_test(
        "crates/x/tests/expected.txt",
        &source
    ));
    assert!(breaches("crates/x/tests/expected.txt", &source).is_empty());
}

#[test]
fn the_word_alone_is_not_a_finding() {
    // A case is allowed to talk about writing. The trailing parenthesis is part
    // of every write token for this reason, and without it the sentence in this
    // very file would be a finding.
    let source =
        "#[test]\nfn it_reads() {\n    // fs::write is what a case must not call here.\n}\n";
    assert!(breaches("crates/x/tests/prose.rs", source).is_empty());
}

#[test]
fn every_default_suite_case_in_this_tree_passes_the_guard() {
    let root = workspace_root();
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "--", "crates/"])
        .output()
        .expect("git is on the path and the workspace root is a git checkout");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let listing = String::from_utf8_lossy(&output.stdout);
    let mut judged = 0usize;
    let mut found: Vec<Breach> = Vec::new();
    for path in listing.lines().map(str::trim_end).filter(|l| !l.is_empty()) {
        if !path.ends_with(".rs") {
            continue;
        }
        let source = fs::read_to_string(root.join(path))
            .unwrap_or_else(|e| panic!("{path} is tracked and readable: {e}"));
        if is_default_suite_test(path, &source) {
            judged += 1;
        }
        found.extend(breaches(path, &source));
    }

    assert!(
        found.is_empty(),
        "these default-suite cases do something the default suite forbids: {found:?}"
    );
    // Without this the case above would pass just as happily against a tree
    // holding no case at all, and a green result would mean the guard never had
    // a subject.
    assert!(
        judged > 1,
        "the guard found {judged} default-suite files in this tree, so it had nothing to judge"
    );
}
