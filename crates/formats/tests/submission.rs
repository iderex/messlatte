//! The submission file: what it refuses, and what it must not (#38).
//!
//! The first half is the register #40 asks for, arranged around the refusals
//! rather than around the clauses of the format. Every refusal the submission
//! validator can produce appears with three legs: a near-miss somebody would
//! plausibly write, the same input with that one fault repaired, and a
//! one-change neighbour that must read. The second leg shows the refusal fired
//! for the fault rather than for the fixture, and the third stops a refusal
//! being broader than its reason.
//!
//! The register is held closed by [`name`], which matches on `Refusal`
//! exhaustively, so a variant added to the validator stops this file compiling.
//! It has the same one gap the trace and dipole registers have and for the same
//! reason: a variant given an arm in [`name`] and left out of [`every`] is not
//! caught, because there is no way on the pinned toolchain to enumerate the
//! variants of an enum, so the sample list is the one edge held by hand.
//!
//! The scope is the submission validator alone. #40 is not discharged by this
//! file: the truth file, the case declaration and the case index are #36, #37
//! and #39, and none of them exists.
//!
//! The second half reads the format from the other end, one case per clause of
//! what a reader may assume, and checks the worked example in
//! `docs/format/submission.md` against the bytes this writer produces. The
//! fixtures here are self-contained rather than shared with that half, because
//! a register whose entries are proved by cases in another file is a register
//! that goes green when that file is thinned.

use std::collections::BTreeSet;

use messlatte_formats::submission::{
    Refusal, Retrieved, Rule, Scoring, Stopping, Streaking, Submission, Version, FORMAT,
};
use messlatte_formats::trace::Axis;

/// The fixture every case below is one edit away from.
///
/// Three samples on a time grid with a gap in it, so that no case here can pass
/// by assuming a uniform one, and no streaking field, so that every edit to a
/// time grid or a unit below reaches exactly one place in the document.
fn base() -> Submission {
    Submission {
        case: "fixture-01".to_string(),
        start: "start-000".to_string(),
        seed: 0x0123_4567_89ab_cdef,
        knowns: vec!["delay-axis".to_string(), "ionisation-potential".to_string()],
        stopping: Stopping {
            rule: Rule::FixedCount { iterations: 200.0 },
            stopped_at: 200.0,
            merit: 0.0125,
            why: "the declared iteration count was reached".to_string(),
        },
        retrieved: Retrieved::Field {
            time: Axis {
                unit: "as".to_string(),
                values: vec![-100.0, -50.0, 50.0],
            },
            real: vec![0.25, 1.0, 0.25],
            imaginary: vec![0.0, 0.5, -0.25],
        },
        streaking: None,
    }
}

/// The same submission in the other domain, for the one refusal that only a
/// spectrum can trip.
fn spectrum() -> Submission {
    Submission {
        retrieved: Retrieved::Spectrum {
            energy: Axis {
                unit: "eV".to_string(),
                values: vec![20.0, 20.5, 22.0],
            },
            amplitude: vec![0.25, 1.0, 0.25],
            phase: vec![0.0, 0.5, -0.25],
        },
        ..base()
    }
}

/// The case the base fixture claims to be about, offering exactly what it read.
fn scored() -> Vec<String> {
    vec!["delay-axis".to_string(), "ionisation-potential".to_string()]
}

/// What the writer says about a submission, and nothing where it writes one.
fn writing(submission: &Submission) -> Vec<Refusal> {
    match submission.to_bytes() {
        Ok(_) => Vec::new(),
        Err(found) => found,
    }
}

/// What the reader says about a document.
fn reading(bytes: &[u8]) -> Vec<Refusal> {
    match Submission::from_bytes(bytes) {
        Ok(_) => Vec::new(),
        Err(found) => found,
    }
}

/// What the pair says about a submission once it has been through both ends. A
/// refusal from the writer is returned as it stands, because a document the
/// writer will not emit is not one the reader can be asked about.
fn round_trip(submission: &Submission) -> Vec<Refusal> {
    match submission.to_bytes() {
        Ok(bytes) => reading(&bytes),
        Err(found) => found,
    }
}

/// The bytes of a submission with substitutions applied.
fn edited(submission: &Submission, edits: &[(&str, &str)]) -> Vec<u8> {
    let bytes = submission.to_bytes().expect("the fixture is writable");
    let mut text = String::from_utf8(bytes).expect("the document is text");
    for (from, to) in edits {
        assert!(
            text.contains(from),
            "the fixture's document does not carry {from:?}"
        );
        assert_eq!(
            text.matches(from).count(),
            1,
            "{from:?} appears more than once, so this edit reaches more than it names"
        );
        text = text.replace(from, to);
    }
    text.into_bytes()
}

/// The base fixture read back with substitutions applied.
fn read_edited(edits: &[(&str, &str)]) -> Vec<Refusal> {
    reading(&edited(&base(), edits))
}

/// The base fixture with one part replaced.
fn changed(edit: impl FnOnce(&mut Submission)) -> Submission {
    let mut submission = base();
    edit(&mut submission);
    submission
}

/// The spectrum fixture with one part replaced.
fn changed_spectrum(edit: impl FnOnce(&mut Vec<f64>, &mut Vec<f64>)) -> Submission {
    let mut submission = spectrum();
    if let Retrieved::Spectrum {
        amplitude, phase, ..
    } = &mut submission.retrieved
    {
        edit(amplitude, phase);
    }
    submission
}

/// The base fixture's real column, for the cases that edit it in place.
fn with_real(edit: impl FnOnce(&mut Vec<f64>)) -> Submission {
    changed(|submission| {
        if let Retrieved::Field { real, .. } = &mut submission.retrieved {
            edit(real);
        }
    })
}

/// The base fixture's time grid and both its columns, replaced together.
fn on_grid(times: Vec<f64>, real: Vec<f64>, imaginary: Vec<f64>) -> Submission {
    changed(|submission| {
        submission.retrieved = Retrieved::Field {
            time: Axis {
                unit: "as".to_string(),
                values: times,
            },
            real,
            imaginary,
        };
    })
}

/// The stopping record of the base fixture, exactly as the writer emits it.
///
/// Held as a literal so that a change to the writer's order or spelling reddens
/// the case that removes it rather than silently editing nothing: [`edited`]
/// asserts the text is present and appears once.
const STOPPING: &str = ",\"stopping\":{\"merit\":0.0125,\"rule\":{\"form\":\"fixed-count\",\
                        \"iterations\":200},\"stopped-at\":200,\"why\":\"the declared iteration \
                        count was reached\"}";

/// One refusal and the three legs that prove it.
struct Case {
    /// A value of the variant this case is about. Only its shape is read.
    refusal: Refusal,
    /// Why this near-miss is one somebody would actually produce.
    why: &'static str,
    /// The near-miss. Must refuse, and one of its refusals must be this
    /// variant.
    near_miss: Box<dyn Fn() -> Vec<Refusal>>,
    /// The same input with that one fault repaired. Must read.
    repaired: Box<dyn Fn() -> Vec<Refusal>>,
    /// A different single change, close enough to the fault to be caught by a
    /// refusal written too broadly. Must read.
    neighbour: Box<dyn Fn() -> Vec<Refusal>>,
}

/// The name of a refusal, and the register that keeps the set closed.
fn name(refusal: &Refusal) -> &'static str {
    match refusal {
        Refusal::Unreadable { .. } => "Unreadable",
        Refusal::NotASubmission { .. } => "NotASubmission",
        Refusal::UnknownMajorVersion { .. } => "UnknownMajorVersion",
        Refusal::Field { .. } => "Field",
        Refusal::Blank { .. } => "Blank",
        Refusal::UnknownUnit { .. } => "UnknownUnit",
        Refusal::EmptyGrid { .. } => "EmptyGrid",
        Refusal::GridNotIncreasing { .. } => "GridNotIncreasing",
        Refusal::LengthDisagrees { .. } => "LengthDisagrees",
        Refusal::NotFinite { .. } => "NotFinite",
        Refusal::AmplitudeNegative { .. } => "AmplitudeNegative",
        Refusal::NoStoppingRecord => "NoStoppingRecord",
        Refusal::KnownNotOffered { .. } => "KnownNotOffered",
        Refusal::CaseMismatch { .. } => "CaseMismatch",
    }
}

/// One value of every refusal the validator can produce.
fn every() -> Vec<Refusal> {
    vec![
        Refusal::Unreadable {
            detail: String::new(),
        },
        Refusal::NotASubmission {
            found: String::new(),
        },
        Refusal::UnknownMajorVersion {
            found: Version { major: 0, minor: 0 },
        },
        Refusal::Field {
            path: String::new(),
            wanted: String::new(),
        },
        Refusal::Blank {
            field: String::new(),
        },
        Refusal::UnknownUnit {
            grid: String::new(),
            unit: String::new(),
            admitted: String::new(),
        },
        Refusal::EmptyGrid {
            grid: String::new(),
        },
        Refusal::GridNotIncreasing {
            grid: String::new(),
            index: 0,
        },
        Refusal::LengthDisagrees {
            column: String::new(),
            grid: String::new(),
            samples: 0,
            values: 0,
        },
        Refusal::NotFinite {
            column: String::new(),
            index: 0,
        },
        Refusal::AmplitudeNegative { index: 0 },
        Refusal::NoStoppingRecord,
        Refusal::KnownNotOffered {
            known: String::new(),
            offered: String::new(),
        },
        Refusal::CaseMismatch {
            found: String::new(),
            scoring: String::new(),
        },
    ]
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            refusal: Refusal::Unreadable {
                detail: String::new(),
            },
            why: "a document truncated by a write that ran out of room, which looks right until \
                  its last three bytes",
            near_miss: Box::new(|| {
                let bytes = edited(&base(), &[]);
                reading(&bytes[..bytes.len() - 3])
            }),
            repaired: Box::new(|| reading(&edited(&base(), &[]))),
            // The writer ends the document with a newline. A file that lost it
            // on the way through an editor is the same document, and a reader
            // that refused it would refuse submissions for a reason the format
            // does not name.
            neighbour: Box::new(|| {
                let bytes = edited(&base(), &[]);
                reading(bytes.strip_suffix(b"\n").expect("the writer ends it"))
            }),
        },
        Case {
            refusal: Refusal::NotASubmission {
                found: String::new(),
            },
            why: "a trace header written into the submission's place in a case directory, which \
                  is two files one command apart",
            near_miss: Box::new(|| {
                read_edited(&[("\"messlatte-submission\"", "\"messlatte-trace\"")])
            }),
            repaired: Box::new(|| read_edited(&[])),
            // The name of another format inside a field that is not the format
            // name. A reader matching the text anywhere in the document would
            // refuse this.
            neighbour: Box::new(|| read_edited(&[("\"fixture-01\"", "\"messlatte-trace\"")])),
        },
        Case {
            refusal: Refusal::UnknownMajorVersion {
                found: Version { major: 0, minor: 0 },
            },
            why: "a submission written by a later version of this workspace, which is what a \
                  colleague's checkout produces",
            near_miss: Box::new(|| read_edited(&[("\"1.0\"", "\"2.0\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // A higher minor version means the same fields mean the same
            // things, so it reads. A refusal written against the version string
            // rather than against its major part would refuse this too.
            neighbour: Box::new(|| read_edited(&[("\"1.0\"", "\"1.7\"")])),
        },
        Case {
            refusal: Refusal::Field {
                path: String::new(),
                wanted: String::new(),
            },
            why: "a stopping record assembled by hand with the sentence left out, which is the \
                  field an author drops because the rule beside it already reads like a reason",
            near_miss: Box::new(|| {
                read_edited(&[(",\"why\":\"the declared iteration count was reached\"", "")])
            }),
            repaired: Box::new(|| read_edited(&[])),
            // A member this reader does not know is ignored rather than
            // refused, which is the other half of the version rule. A reader
            // refusing every unexpected member would refuse this.
            neighbour: Box::new(|| {
                read_edited(&[("\"case\":", "\"acquired\":\"2026-01-01\",\"case\":")])
            }),
        },
        Case {
            refusal: Refusal::Blank {
                field: String::new(),
            },
            why: "a start identifier from a template that was never filled in, which is what a \
                  run that wrote its first start before naming them produces",
            near_miss: Box::new(|| read_edited(&[("\"start-000\"", "\"\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // The identifier is a name and this format says nothing else about
            // it. A refusal written against a shape rather than against
            // emptiness would refuse a short one.
            neighbour: Box::new(|| read_edited(&[("\"start-000\"", "\"a\"")])),
        },
        Case {
            refusal: Refusal::UnknownUnit {
                grid: String::new(),
                unit: String::new(),
                admitted: String::new(),
            },
            why: "a time grid left in the atomic units the numerics work in, which is a factor \
                  of twenty-four orders of magnitude away from the seconds a file admits",
            near_miss: Box::new(|| read_edited(&[("\"unit\":\"as\"", "\"unit\":\"a.u.\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // The femtosecond is another unit the time grid admits. A refusal
            // written against anything but the fixture's own unit would refuse
            // it.
            neighbour: Box::new(|| read_edited(&[("\"unit\":\"as\"", "\"unit\":\"fs\"")])),
        },
        Case {
            refusal: Refusal::EmptyGrid {
                grid: String::new(),
            },
            why: "a support window applied to a retrieved field until nothing was left inside \
                  it, which produces a grid with no samples and columns to match",
            near_miss: Box::new(|| writing(&on_grid(Vec::new(), Vec::new(), Vec::new()))),
            repaired: Box::new(|| round_trip(&base())),
            // One sample is a thin retrieval and not an empty one. A refusal
            // written against a length below two would delete it.
            neighbour: Box::new(|| round_trip(&on_grid(vec![0.0], vec![1.0], vec![0.0]))),
        },
        Case {
            refusal: Refusal::GridNotIncreasing {
                grid: String::new(),
                index: 0,
            },
            why: "a time grid whose last two samples repeat, which is what concatenating two \
                  windows that share an endpoint produces",
            near_miss: Box::new(|| read_edited(&[("[-100,-50,50]", "[-100,-50,-50]")])),
            repaired: Box::new(|| read_edited(&[])),
            // A retrieval on a grid with a gap in it is a submission and not a
            // fault, because a method may return the support it is confident
            // about. A refusal written against a step that is not the first
            // step would delete it.
            neighbour: Box::new(|| read_edited(&[("[-100,-50,50]", "[-100,-50,400]")])),
        },
        Case {
            refusal: Refusal::LengthDisagrees {
                column: String::new(),
                grid: String::new(),
                samples: 0,
                values: 0,
            },
            why: "a column written out one sample short by a loop that stopped at the last \
                  index rather than past it",
            near_miss: Box::new(|| read_edited(&[("\"real\":[0.25,1,0.25]", "\"real\":[0.25,1]")])),
            repaired: Box::new(|| read_edited(&[])),
            // A submission on a grid of another length is the ordinary case. A
            // refusal written against the fixture's own three samples rather
            // than against the grid it is on would refuse this.
            neighbour: Box::new(|| {
                round_trip(&on_grid(
                    vec![-100.0, -50.0, 50.0, 100.0],
                    vec![0.25, 1.0, 0.25, 0.125],
                    vec![0.0, 0.5, -0.25, 0.0],
                ))
            }),
        },
        Case {
            refusal: Refusal::NotFinite {
                column: String::new(),
                index: 0,
            },
            why: "a sample standing in for one the method did not retrieve, which is the \
                  sentinel this format refuses to carry",
            // Only the writer can be shown this. The document is JSON and JSON
            // has no spelling for a value that is not a number, so the reader
            // refuses those bytes before a column exists to judge.
            near_miss: Box::new(|| writing(&with_real(|real| real[1] = f64::NAN))),
            repaired: Box::new(|| round_trip(&base())),
            // A sample a method retrieved as zero is a value and not a missing
            // one. A check written against a falsy sample would delete every
            // zero in the retrieval.
            neighbour: Box::new(|| round_trip(&with_real(|real| real[1] = 0.0))),
        },
        Case {
            refusal: Refusal::AmplitudeNegative { index: 0 },
            why: "a spectral amplitude carrying the sign of its own real part, which is what \
                  submitting the real spectrum in the amplitude column produces",
            near_miss: Box::new(|| writing(&changed_spectrum(|amplitude, _| amplitude[1] = -1.0))),
            repaired: Box::new(|| round_trip(&spectrum())),
            // A negative phase is the ordinary case, since a phase runs either
            // way from zero. A refusal written against a negative number
            // anywhere in the retrieval would delete half of every spectrum.
            neighbour: Box::new(|| round_trip(&changed_spectrum(|_, phase| phase[1] = -1.5))),
        },
        Case {
            refusal: Refusal::NoStoppingRecord,
            why: "a submission from a method whose stopping rule lives in its operator's head, \
                  which is the practice this board exists to measure",
            near_miss: Box::new(|| read_edited(&[(STOPPING, "")])),
            repaired: Box::new(|| read_edited(&[])),
            // A wall-clock rule is a stopping record, admitted for the cost
            // comparison. A refusal written against the absence of an iteration
            // count rather than against the absence of the record would refuse
            // it.
            neighbour: Box::new(|| {
                round_trip(&changed(|submission| {
                    submission.stopping.rule = Rule::WallClock { seconds: 30.0 };
                    submission.stopping.why = "the wall-clock budget ran out".to_string();
                }))
            }),
        },
        Case {
            refusal: Refusal::KnownNotOffered {
                known: String::new(),
                offered: String::new(),
            },
            why: "a method carrying a known from the case family it was developed on into one \
                  that withholds it, which is the shape a comparison cannot see from a score",
            near_miss: Box::new(|| {
                base().against(&Scoring {
                    case: "fixture-01",
                    offered: &["delay-axis".to_string()],
                })
            }),
            repaired: Box::new(|| {
                base().against(&Scoring {
                    case: "fixture-01",
                    offered: &scored(),
                })
            }),
            // A case may offer more than a method chose to read, and a method
            // that read less than it was allowed to is not cheating. A refusal
            // written against the two lists differing would refuse this.
            neighbour: Box::new(|| {
                let mut offered = scored();
                offered.push("streaking-field".to_string());
                base().against(&Scoring {
                    case: "fixture-01",
                    offered: &offered,
                })
            }),
        },
        Case {
            refusal: Refusal::CaseMismatch {
                found: String::new(),
                scoring: String::new(),
            },
            why: "a submission copied from the neighbouring case directory and rerun without \
                  its case identifier being changed, which scores one case against another's \
                  truth",
            near_miss: Box::new(|| {
                base().against(&Scoring {
                    case: "fixture-02",
                    offered: &scored(),
                })
            }),
            repaired: Box::new(|| {
                base().against(&Scoring {
                    case: "fixture-01",
                    offered: &scored(),
                })
            }),
            // A method that read none of what the case offered is scored like
            // any other. A refusal written over both halves of this comparison
            // at once, or one that required either list to be non-empty, would
            // refuse it.
            neighbour: Box::new(|| {
                changed(|submission| submission.knowns.clear()).against(&Scoring {
                    case: "fixture-01",
                    offered: &[],
                })
            }),
        },
    ]
}

#[test]
fn every_refusal_the_validator_can_produce_has_a_case() {
    let named: BTreeSet<&str> = every().iter().map(name).collect();
    assert_eq!(
        named.len(),
        every().len(),
        "two samples name one refusal, so the register is short by one"
    );
    let proved: BTreeSet<&str> = cases().iter().map(|case| name(&case.refusal)).collect();
    let missing: Vec<&str> = named.difference(&proved).copied().collect();
    assert!(
        missing.is_empty(),
        "these refusals ship without a case that trips them: {missing:?}"
    );
    let stray: Vec<&str> = proved.difference(&named).copied().collect();
    assert!(
        stray.is_empty(),
        "these cases name a refusal absent from the register: {stray:?}"
    );
}

#[test]
fn the_near_miss_of_every_case_is_refused_for_the_reason_it_names() {
    for case in cases() {
        let wanted = name(&case.refusal);
        let found = (case.near_miss)();
        let names: Vec<&str> = found.iter().map(name).collect();
        assert!(
            names.contains(&wanted),
            "the near-miss for {wanted} was refused as {names:?}, and the fixture is {}",
            case.why
        );
    }
}

#[test]
fn every_case_reads_once_its_one_fault_is_repaired() {
    for case in cases() {
        let wanted = name(&case.refusal);
        let found = (case.repaired)();
        assert!(
            found.is_empty(),
            "the repaired input for {wanted} is still refused: {found:?}"
        );
    }
}

#[test]
fn no_refusal_reaches_its_one_change_neighbour() {
    for case in cases() {
        let wanted = name(&case.refusal);
        let found = (case.neighbour)();
        assert!(
            found.is_empty(),
            "the neighbour of {wanted} is refused, so that refusal is broader than its \
             reason: {found:?}"
        );
    }
}

// The second half: the format from the other end.

/// The worked example of `docs/format/submission.md`, as that document holds
/// it. A change to either side reddens here.
const WORKED_EXAMPLE: &str = concat!(
    "{\"case\":\"fixture-01\",",
    "\"format\":{\"name\":\"messlatte-submission\",\"version\":\"1.0\"},",
    "\"knowns\":[\"delay-axis\",\"ionisation-potential\"],",
    "\"retrieved\":{\"imaginary\":[0,0.5,-0.25],\"quantity\":\"field\",",
    "\"real\":[0.25,1,0.25],\"time\":{\"unit\":\"as\",\"values\":[-100,-50,50]}},",
    "\"seed\":\"0123456789abcdef\",",
    "\"start\":\"start-000\",",
    "\"stopping\":{\"merit\":0.0125,\"rule\":{\"form\":\"fixed-count\",\"iterations\":200},",
    "\"stopped-at\":200,\"why\":\"the declared iteration count was reached\"}}\n"
);

#[test]
fn the_writer_produces_the_document_the_format_document_prints() {
    let bytes = base().to_bytes().expect("the fixture is writable");
    assert_eq!(
        String::from_utf8(bytes).expect("the document is text"),
        WORKED_EXAMPLE
    );
}

#[test]
fn a_submission_survives_the_round_trip_unchanged() {
    for submission in [base(), spectrum()] {
        let bytes = submission.to_bytes().expect("the fixture is writable");
        let read = Submission::from_bytes(&bytes).expect("the fixture reads");
        assert_eq!(read, submission);
        assert_eq!(
            read.to_bytes().expect("what was read is writable"),
            bytes,
            "a document read and written again is not the same bytes"
        );
    }
}

/// The base fixture with a streaking field on the same grid.
fn streaked() -> Submission {
    changed(|submission| {
        submission.streaking = Some(Streaking {
            time: Axis {
                unit: "as".to_string(),
                values: vec![-100.0, -50.0, 50.0],
            },
            unit: "kg m/s".to_string(),
            values: vec![-0.5, 0.25, 1.5],
        });
    })
}

#[test]
fn a_retrieved_streaking_field_is_carried_and_is_optional() {
    assert!(base().streaking.is_none());
    let bytes = streaked().to_bytes().expect("the fixture is writable");
    let read = Submission::from_bytes(&bytes).expect("the fixture reads");
    assert_eq!(read, streaked());
}

#[test]
fn a_streaking_field_in_a_unit_no_file_states_is_refused() {
    let found = reading(&edited(
        &streaked(),
        &[("\"unit\":\"kg m/s\"", "\"unit\":\"a.u.\"")],
    ));
    let names: Vec<&str> = found.iter().map(name).collect();
    assert!(
        names.contains(&"UnknownUnit"),
        "a vector potential left in atomic units read as {names:?}"
    );
}

#[test]
fn the_seed_is_sixteen_hexadecimal_digits_and_survives_the_round_trip() {
    // The largest seed there is, which is the one a number would round.
    let submission = changed(|submission| submission.seed = u64::MAX);
    let bytes = submission.to_bytes().expect("the fixture is writable");
    assert!(String::from_utf8_lossy(&bytes).contains("\"seed\":\"ffffffffffffffff\""));
    let read = Submission::from_bytes(&bytes).expect("the fixture reads");
    assert_eq!(read.seed, u64::MAX);
}

#[test]
fn a_seed_written_as_a_number_or_in_the_wrong_case_is_refused() {
    for spelling in [
        "\"seed\":81985529216486895",
        "\"seed\":\"123456789ABCDEF\"",
        "\"seed\":\"0123456789ABCDEF\"",
        "\"seed\":\"abc\"",
    ] {
        let found = read_edited(&[("\"seed\":\"0123456789abcdef\"", spelling)]);
        assert!(
            !found.is_empty(),
            "the seed spelled {spelling} was accepted, and a seed that does not read back \
             exactly reproduces a different start"
        );
    }
}

#[test]
fn a_stopping_rule_this_format_does_not_name_is_refused() {
    let found = read_edited(&[("\"fixed-count\"", "\"until-it-looks-right\"")]);
    let names: Vec<&str> = found.iter().map(name).collect();
    assert_eq!(
        names,
        vec!["Field"],
        "a stopping rule outside the three forms was read as {names:?}"
    );
}

#[test]
fn an_iteration_count_that_is_not_a_whole_number_is_refused() {
    for spelling in ["200.5", "-1", "10000000000000000000"] {
        let found = read_edited(&[("\"iterations\":200", &format!("\"iterations\":{spelling}"))]);
        assert!(
            !found.is_empty(),
            "an iteration count of {spelling} was accepted"
        );
    }
}

#[test]
fn the_document_says_which_format_it_is_before_it_says_anything_else() {
    // A document of another format with a member this one requires missing is
    // told which format it is, and not which member. A reader that checked the
    // members first would send its author after the wrong file.
    let found = read_edited(&[
        ("\"messlatte-submission\"", "\"messlatte-truth\""),
        (STOPPING, ""),
    ]);
    let names: Vec<&str> = found.iter().map(name).collect();
    assert_eq!(names, vec!["NotASubmission"]);
    assert_eq!(FORMAT, "messlatte-submission");
}
