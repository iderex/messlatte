//! The directory a default-suite case is allowed to write in.
//!
//! Every case here writes, and none of them names a shape the guard refuses.
//! That is the property being demonstrated as much as it is the way the cases
//! are written: the rule forbids a shape and this is what to write instead.

use std::path::Path;

use messlatte_suites::Scratch;

#[test]
fn it_is_under_the_temporary_directory_and_it_holds_what_was_written() {
    let scratch = Scratch::new("case").expect("a scratch directory can be made");
    assert!(
        scratch.path().starts_with(std::env::temp_dir()),
        "{:?} is not under {:?}",
        scratch.path(),
        std::env::temp_dir()
    );

    let written = scratch
        .write("trace.txt", b"one line")
        .expect("the write lands");
    assert_eq!(
        std::fs::read(&written).expect("it is readable"),
        b"one line"
    );
    assert!(written.starts_with(scratch.path()));
}

#[test]
fn two_at_once_do_not_collide() {
    // Cases in one target run as threads, so two of them asking for a scratch
    // directory at the same moment is the ordinary case rather than the corner.
    let first = Scratch::new("case").expect("a scratch directory can be made");
    let second = Scratch::new("case").expect("a second one can be made");
    assert_ne!(first.path(), second.path());
}

#[test]
fn it_is_gone_when_the_case_ends() {
    let path = {
        let scratch = Scratch::new("ends").expect("a scratch directory can be made");
        scratch.write("a.txt", b"x").expect("the write lands");
        scratch.path().to_path_buf()
    };
    assert!(!Path::new(&path).exists(), "{path:?} outlived its case");
}

#[test]
fn a_label_that_would_leave_the_temporary_directory_is_refused() {
    // The near-miss. A label carrying a separator or a parent segment would put
    // the directory somewhere else, and the removal on drop would then remove
    // something nobody asked it to.
    for label in ["..", ".", "../elsewhere", "a/b", "a\\b", "", "a b"] {
        assert!(
            Scratch::new(label).is_err(),
            "the label {label:?} was accepted"
        );
    }
}

#[test]
fn a_relative_path_is_written_with_the_directories_above_it() {
    // A case that builds a small tree on disk is what this is for, so a path
    // with segments in it is the ordinary use rather than a corner. The
    // separator is a forward slash on every platform.
    let scratch = Scratch::new("nested").expect("a scratch directory can be made");
    let written = scratch
        .write("g/src/lib.rs", b"// nothing")
        .expect("the write lands");
    assert_eq!(
        std::fs::read(&written).expect("it is readable"),
        b"// nothing"
    );
    assert!(written.starts_with(scratch.path()));
}

#[test]
fn a_path_that_would_leave_the_directory_is_refused() {
    // The near-miss. A parent segment is one segment among several, so a rule
    // that looked at the path as a whole rather than at every segment would let
    // the second of these through and write outside the directory this type
    // promises to keep everything inside and then remove.
    let scratch = Scratch::new("names").expect("a scratch directory can be made");
    for name in [
        "../escape.txt",
        "g/../../escape.txt",
        "",
        "..",
        "a\\b.txt",
        "/abs.txt",
        "g//lib.rs",
    ] {
        assert!(
            scratch.write(name, b"x").is_err(),
            "the name {name:?} was accepted"
        );
    }
}
