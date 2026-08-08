//! What a value of a suite variable is read as, and what the report says.
//!
//! These are fixture values rather than the real environment. A case that read
//! the environment would prove what this machine happened to have set on the
//! day it ran; these prove that a value nobody understood cannot arrive as a
//! decision.

use messlatte_suites::suite::{report_from_env, ALL};
use messlatte_suites::{enrol, report, Enrolment, Suite};

#[test]
fn an_unset_variable_is_nobody_asking() {
    assert_eq!(enrol(None), Ok(Enrolment::NotAsked));
}

#[test]
fn one_is_asking_for_it() {
    assert_eq!(enrol(Some("1")), Ok(Enrolment::Ran));
}

#[test]
fn zero_is_asking_for_it_not_to_run() {
    assert_eq!(enrol(Some("0")), Ok(Enrolment::Declined));
}

#[test]
fn a_value_nobody_understood_is_an_error_and_not_a_decline() {
    // The near-miss, and the one somebody will actually make. `true` is what a
    // person types when they mean 1. If it read as a decline, the run would
    // report the suite as not asked for while the person who asked watched the
    // default suite pass and read it as everything.
    for value in ["true", "yes", "on", "1 ", "", "01"] {
        let verdict = enrol(Some(value));
        assert!(
            verdict.is_err(),
            "the value {value:?} was read as {verdict:?} rather than refused"
        );
    }
}

#[test]
fn every_suite_has_its_own_variable_and_its_own_name() {
    // Two suites sharing a variable would mean asking for one silently ran the
    // other, and two sharing a name would make the report unreadable at the
    // moment it matters.
    let mut variables: Vec<&str> = ALL.iter().map(|suite| suite.variable()).collect();
    let mut names: Vec<&str> = ALL.iter().map(|suite| suite.name()).collect();
    variables.sort_unstable();
    names.sort_unstable();
    let before = (variables.len(), names.len());
    variables.dedup();
    names.dedup();
    assert_eq!(
        before,
        (variables.len(), names.len()),
        "{variables:?} {names:?}"
    );
}

#[test]
fn a_suite_that_did_not_run_is_printed_as_not_run_with_its_reason_and_its_cost() {
    let text = report(&[
        (Suite::Minutes, Enrolment::Ran),
        (Suite::Accelerator, Enrolment::NotAsked),
        (Suite::OutsideFile, Enrolment::Declined),
    ]);

    assert!(text.contains("minutes        RAN"), "{text}");
    assert!(text.contains("accelerator    NOT RUN"), "{text}");
    assert!(text.contains("outside-file   NOT RUN"), "{text}");

    for suite in ALL {
        assert!(text.contains(suite.variable()), "{text}");
        assert!(text.contains(suite.cost()), "{text}");
    }

    assert!(text.contains("did not pass"), "{text}");
}

#[test]
fn the_default_suite_is_in_the_report_even_though_it_is_not_opt_in() {
    // Without this line the report would list only what did not run, and a
    // reader would have nothing saying which suite the passes came from.
    assert!(report(&[]).contains("default"));
}

#[test]
fn the_report_from_the_environment_covers_every_suite() {
    // The only case here that reads the environment. It asserts coverage rather
    // than a verdict, because the verdict is a fact about whoever started this
    // process.
    let text = report_from_env().expect("this process holds no unreadable suite variable");
    for suite in ALL {
        assert!(text.contains(suite.name()), "{text}");
    }
}
