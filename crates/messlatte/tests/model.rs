//! The command that prints the forward model (#42).
//!
//! The clause this covers is that a command prints the expression, its symbol
//! table and the omissions. Reading the constant out of the library would not
//! cover it: what a reviewer runs is the binary, and a verb that stopped
//! reaching the constant would leave every case in the generator crate green.
//!
//! This is the whole of what the binary does today. What an operator runs is
//! #88 and is not built, so a run of this binary is not a run of this
//! repository.

use std::process::Command;

fn run(arguments: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_messlatte"))
        .args(arguments)
        .output()
        .expect("the binary this case was built beside is runnable");
    (
        String::from_utf8(output.stdout).expect("the output is text"),
        String::from_utf8(output.stderr).expect("the output is text"),
        output.status.success(),
    )
}

#[test]
fn the_model_verb_prints_the_form_the_generator_holds() {
    let (out, err, ok) = run(&["model"]);
    assert!(ok, "the verb failed: {err}");
    assert_eq!(out, messlatte_generator::operator::PRINTED_FORM);
}

#[test]
fn what_it_prints_carries_the_omissions_and_not_only_the_expression() {
    let (out, _, ok) = run(&["model"]);
    assert!(ok);
    for omission in [
        "no depletion of the ground state",
        "no space charge",
        "no propagation of either field through the target",
        "no vector character beyond the single declared polarisation direction",
        "one active electron",
        "one final state per target",
    ] {
        assert!(
            out.contains(omission),
            "the command printed the expression without {omission:?}, and the absence of a term \
             read as a claim that it is negligible is what printing them prevents"
        );
    }
}

#[test]
fn a_verb_this_binary_does_not_have_is_refused_rather_than_ignored() {
    let (out, err, ok) = run(&["score"]);
    assert!(!ok, "an unknown verb exited zero, having done nothing");
    assert!(out.is_empty(), "it printed {out:?} on the way out");
    assert!(err.contains("score"), "it did not name the verb: {err}");
}

#[test]
fn no_verb_at_all_still_prints_the_version() {
    let (out, _, ok) = run(&[]);
    assert!(ok);
    assert!(out.starts_with("messlatte "), "{out:?}");
}
