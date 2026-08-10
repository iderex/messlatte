//! Every module that declares one of this project's formats carries the version
//! rule.
//!
//! The rule is #41: every file this project writes carries a format version, a
//! reader refuses a major version it does not know rather than guessing, and a
//! higher minor version reads with the fields the reader does not recognise
//! ignored. Three modules apply it today, each with its own copy, and until this
//! case there was nothing between them. A fourth format could declare a name, no
//! version and no refusal for a major version, and every case in the workspace
//! would stay green while a colleague's file scored against a misread document.
//!
//! Three more formats are already on the tracker, in #36, #37 and #39, which is
//! what makes the gap worth closing before they are written rather than after.
//!
//! The subject is what git stores rather than the working directory, for the
//! reason the two cases beside this one give: the bytes somebody else checks out
//! are the ones in the index, and a rule about what the tree may contain that
//! read the working copy would be about a different tree.
//!
//! A checkout with no git available reds this case rather than passing it.

use std::path::{Path, PathBuf};
use std::process::Command;

use messlatte_tree::{format_module_marker, offenders, version_rule_gaps};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

/// A format module's source, with each part of the rule present or absent.
///
/// The format constant is joined at run time rather than written out. It is the
/// pattern the search below uses, and spelled whole here this file would be the
/// first module that search named and the case would judge its own fixtures. The
/// other three parts are not search patterns and are written as they are written
/// in a real module.
///
/// The fixture is text rather than a module that compiles, which is the shape
/// the thing under test wants: it reads source as text and cannot do otherwise.
fn module(version: bool, unknown_major: bool, major_comparison: bool) -> String {
    let mut source = String::from("//! A format, as this repository writes one.\n\n");
    source.push_str(&format!(
        "{}: &str = \"messlatte-example\";\n",
        format_module_marker()
    ));
    if version {
        source.push_str("pub const VERSION: Version = Version { major: 1, minor: 0 };\n");
    }
    if unknown_major {
        source.push_str("\npub enum Refusal {\n    UnknownMajorVersion { found: Version },\n}\n");
    }
    source.push_str("\npub fn read(bytes: &[u8]) -> Result<Example, Vec<Refusal>> {\n");
    if major_comparison {
        source.push_str("    if version.major != VERSION.major {\n");
    } else {
        // The near miss, and the one worth spending the effort on. Two
        // characters either side: the whole version compared instead of its
        // major part. It refuses nothing a reviewer is looking for, it reads
        // like the check it is not, and what it breaks is the other half of the
        // rule, where a higher minor version has to read.
        source.push_str("    if version != VERSION {\n");
    }
    source.push_str("        return Err(refusal(version));\n    }\n    read_fields(bytes)\n}\n");
    source
}

/// A module that reads a format somebody else specified.
///
/// The real one is `crates/formats/src/npy.rs`. It refuses a container version
/// it cannot read and declares no format name of this project's, because it is
/// not one of this project's formats. It is here so the two-sided condition has
/// a case on the other side.
fn container() -> String {
    String::from(
        "//! The array container: NPY version 1.0.\n\n\
         const MAGIC: &[u8] = b\"\\x93NUMPY\";\n\n\
         pub fn read(bytes: &[u8]) -> Result<Array, String> {\n\
         \x20   let (major, minor) = version_bytes(bytes)?;\n\
         }\n",
    )
}

/// The tracked modules that declare one of this project's formats.
fn format_modules(root: &Path) -> Vec<String> {
    let marker = format_module_marker();
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["grep", "--cached", "-F", "-l", "-e"])
        .arg(&marker)
        .args(["--", "crates"])
        .output()
        .expect("git is on the path and the workspace root is a git checkout");

    offenders(
        output.status.code(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
    .expect("git grep was understood")
}

/// What git stores for one path, which is not what the working directory holds.
fn staged(root: &Path, path: &str) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("show")
        .arg(format!(":{path}"))
        .output()
        .expect("git is on the path and the workspace root is a git checkout");

    assert!(
        output.status.success(),
        "git show :{path} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    String::from_utf8(output.stdout).expect("a tracked Rust source is UTF-8")
}

#[test]
fn a_format_carrying_the_whole_rule_is_admitted() {
    assert!(version_rule_gaps(&module(true, true, true)).is_empty());
}

#[test]
fn a_format_that_does_not_say_which_version_it_writes_is_named() {
    let gaps = version_rule_gaps(&module(false, true, true));
    assert_eq!(gaps.len(), 1, "{gaps:?}");
    assert!(gaps[0].contains("pub const VERSION"), "{gaps:?}");
}

#[test]
fn a_format_with_no_refusal_for_a_major_version_it_does_not_know_is_named() {
    let gaps = version_rule_gaps(&module(true, false, true));
    assert_eq!(gaps.len(), 1, "{gaps:?}");
    assert!(gaps[0].contains("UnknownMajorVersion"), "{gaps:?}");
}

#[test]
fn a_reader_judging_the_whole_version_rather_than_its_major_part_is_named() {
    let gaps = version_rule_gaps(&module(true, true, false));
    assert_eq!(gaps.len(), 1, "{gaps:?}");
    assert!(gaps[0].contains(".major"), "{gaps:?}");
}

#[test]
fn the_comparison_is_read_in_both_operators_and_both_orders() {
    // Two people write this decision two ways and neither is a mistake, so
    // neither is refused. A case reading one spelling would send the second
    // author to rewrite working code.
    let written = |comparison: &str| {
        module(true, true, false).replace("if version != VERSION {", &format!("if {comparison} {{"))
    };
    for comparison in [
        "version.major != VERSION.major",
        "VERSION.major == version.major",
    ] {
        assert!(
            version_rule_gaps(&written(comparison)).is_empty(),
            "{comparison}"
        );
    }
}

#[test]
fn a_format_that_skipped_the_rule_entirely_is_named_part_by_part() {
    // Three sentences rather than one, because a module that lost all three
    // needs all three back and a report naming only the first would send
    // somebody round this loop twice.
    let gaps = version_rule_gaps(&module(false, false, false));
    assert_eq!(gaps.len(), 3, "{gaps:?}");
}

#[test]
fn a_module_that_declares_no_format_is_not_this_cases_subject() {
    // The other side of the condition. The container declares no format of this
    // project's, and this says nothing about it rather than demanding the rule
    // of a format this project did not define.
    assert!(version_rule_gaps(&container()).is_empty());
}

#[test]
fn every_tracked_format_module_carries_the_version_rule() {
    let root = workspace_root();
    for path in format_modules(&root) {
        let gaps = version_rule_gaps(&staged(&root, &path));
        assert!(
            gaps.is_empty(),
            "{path} declares one of this project's formats and misses part of the version rule: \
             {gaps:?}"
        );
    }
}

#[test]
fn the_search_this_runs_against_found_something_to_judge() {
    // Without this, the case above would pass just as happily against a tree
    // holding no formats at all, or against a search whose pathspec had stopped
    // matching, and a green result would mean the guard never had a subject.
    let modules = format_modules(&workspace_root());
    assert!(
        !modules.is_empty(),
        "no tracked module under crates/ declares one of this project's formats, so the version \
         rule was judged against nothing"
    );
}
