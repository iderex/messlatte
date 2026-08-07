//! What `offenders` makes of each thing `git grep -l` can do.
//!
//! These are fixture strings rather than real invocations. A case that ran git
//! would prove the state of the tree on the day it ran; these prove that a
//! status the caller did not expect cannot arrive as a clean result.

use messlatte_tree::offenders;

#[test]
fn nothing_found_is_an_empty_list() {
    assert_eq!(offenders(Some(1), "", ""), Ok(Vec::new()));
}

#[test]
fn a_match_is_the_paths_it_named() {
    assert_eq!(
        offenders(Some(0), "a.rs\nb/c.toml\n", ""),
        Ok(vec!["a.rs".to_string(), "b/c.toml".to_string()])
    );
}

#[test]
fn a_match_with_no_paths_is_an_error_and_not_a_clean_result() {
    // The near-miss. If this returned an empty list, an invocation whose output
    // format stopped being understood would read exactly like a clean tree.
    assert!(offenders(Some(0), "", "").is_err());
}

#[test]
fn a_status_the_caller_does_not_recognise_is_an_error() {
    // Exit 128 is what git gives outside a repository. Folding it into "nothing
    // found" is the shape this function exists to refuse.
    let result = offenders(Some(128), "", "fatal: not a git repository");
    assert!(result.is_err(), "{result:?}");
    assert!(result.unwrap_err().contains("128"));
}

#[test]
fn no_status_at_all_is_an_error() {
    assert!(offenders(None, "", "killed").is_err());
}

#[test]
fn trailing_blank_lines_are_not_paths() {
    assert_eq!(
        offenders(Some(0), "a.rs\n\n", ""),
        Ok(vec!["a.rs".to_string()])
    );
}
