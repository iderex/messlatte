//! The dipole table: what it refuses, what it interpolates, and the sign (#44).
//!
//! The first half of this file is the register #40 asks for, arranged around
//! the refusals rather than around the clauses of the format. Every refusal the
//! dipole validator can produce appears with three legs: a near-miss somebody
//! would plausibly write, the same input with that one fault repaired, and a
//! one-change neighbour that must read. The second leg shows the refusal fired
//! for the fault rather than for the fixture, and the third stops a refusal
//! being broader than its reason.
//!
//! The register is held closed by [`name`], which matches on `Refusal`
//! exhaustively, so a variant added to the validator stops this file compiling.
//! It has the same one gap the trace register has and for the same reason: a
//! variant given an arm in [`name`] and left out of [`every`] is not caught,
//! because there is no way on the pinned toolchain to enumerate the variants of
//! an enum, so the sample list is the one edge held by hand.
//!
//! The scope is the dipole validator alone. #40 is not discharged by this file:
//! the truth file, the case declaration, the submission and the case index are
//! #36, #37, #38 and #39 and none of them exists.
//!
//! The second half reads `docs/format/dipole.md` from the other end. That
//! document is the authority for the conventions and the numbers here are its
//! numbers, so a convention that moved in the code and not in the document
//! reddens here.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use messlatte_formats::dipole::{Convention, Lookup, Refusal, Table, Version, FLAT};
use messlatte_units::{Energy, Time};

/// Two roundings and no more, the tolerance the units tests use beside the same
/// reasoning.
const TOLERANCE: f64 = 4.0 * f64::EPSILON;

#[track_caller]
fn close(left: f64, right: f64) {
    let scale = left.abs().max(right.abs());
    assert!(
        (left - right).abs() <= TOLERANCE * scale,
        "{left:e} and {right:e} differ by more than {TOLERANCE:e} of the larger"
    );
}

fn ev(value: f64) -> Energy {
    Energy::from_si(value, "eV").expect("the electronvolt is admitted")
}

/// The worked example of `docs/format/dipole.md`, in this repository's own
/// convention, which is what the writer emits.
fn base() -> Table {
    Table {
        target: "worked-example".to_string(),
        source: "a fixture rather than a table of anything real".to_string(),
        unit: "eV".to_string(),
        energy: vec![20.0, 30.0, 40.0],
        amplitude: vec![1.0, 2.0, 1.5],
        normalisation: "the largest amplitude in this table".to_string(),
        phase: vec![-0.25, -0.75, -0.5],
    }
}

/// The same three samples as the document writes them, in the source's own
/// convention, so that the conversion is visible in the bytes.
fn worked_example_as_the_source_writes_it() -> Vec<u8> {
    document(
        "exp(-i phase)",
        "[20,30,40]",
        "[1,2,1.5]",
        "[0.25,0.75,0.5]",
    )
}

/// One table document, assembled rather than written by the writer, because the
/// writer only ever emits this repository's own convention.
fn document(convention: &str, energy: &str, amplitude: &str, phase: &str) -> Vec<u8> {
    format!(
        "{{\"amplitude\":{{\"normalisation\":\"the largest amplitude in this table\",\
         \"values\":{amplitude}}},\"energy\":{{\"unit\":\"eV\",\"values\":{energy}}},\
         \"format\":{{\"name\":\"messlatte-dipole\",\"version\":\"1.0\"}},\
         \"phase\":{{\"convention\":\"{convention}\",\"values\":{phase}}},\
         \"source\":\"a fixture rather than a table of anything real\",\
         \"target\":\"worked-example\"}}\n"
    )
    .into_bytes()
}

/// What the writer says about a table, and nothing where it writes one.
fn writing(table: &Table) -> Vec<Refusal> {
    match table.to_bytes() {
        Ok(_) => Vec::new(),
        Err(found) => found,
    }
}

/// What the reader says about bytes.
fn reading(bytes: &[u8]) -> Vec<Refusal> {
    match Table::from_bytes(bytes) {
        Ok(_) => Vec::new(),
        Err(found) => found,
    }
}

/// What the document says about a table once it has been through both ends.
fn round_trip(table: &Table) -> Vec<Refusal> {
    match table.to_bytes() {
        Ok(bytes) => reading(&bytes),
        Err(found) => found,
    }
}

/// The base fixture's document with substitutions applied.
fn edited(edits: &[(&str, &str)]) -> Vec<u8> {
    let bytes = base().to_bytes().expect("the base fixture is writable");
    let mut text = String::from_utf8(bytes).expect("the document is text");
    for (from, to) in edits {
        assert!(
            text.contains(from),
            "the base fixture's document does not carry {from:?}"
        );
        text = text.replace(from, to);
    }
    text.into_bytes()
}

/// The base fixture read back with substitutions applied.
fn read_edited(edits: &[(&str, &str)]) -> Vec<Refusal> {
    reading(&edited(edits))
}

/// The base fixture with one field of the table replaced.
fn changed(edit: impl FnOnce(&mut Table)) -> Table {
    let mut table = base();
    edit(&mut table);
    table
}

/// One refusal and the three legs that prove it.
struct Case {
    /// A value of the variant this case is about. Only its shape is read.
    refusal: Refusal,
    /// Why this near-miss is one somebody would actually produce.
    why: &'static str,
    near_miss: Box<dyn Fn() -> Vec<Refusal>>,
    repaired: Box<dyn Fn() -> Vec<Refusal>>,
    neighbour: Box<dyn Fn() -> Vec<Refusal>>,
}

/// The name of a refusal, and the register that keeps the set closed.
fn name(refusal: &Refusal) -> &'static str {
    match refusal {
        Refusal::Unreadable { .. } => "Unreadable",
        Refusal::NotADipoleTable { .. } => "NotADipoleTable",
        Refusal::UnknownMajorVersion { .. } => "UnknownMajorVersion",
        Refusal::Field { .. } => "Field",
        Refusal::Blank { .. } => "Blank",
        Refusal::UnknownUnit { .. } => "UnknownUnit",
        Refusal::UnknownConvention { .. } => "UnknownConvention",
        Refusal::TooFewSamples { .. } => "TooFewSamples",
        Refusal::LengthDisagrees { .. } => "LengthDisagrees",
        Refusal::ColumnNotFinite { .. } => "ColumnNotFinite",
        Refusal::EnergyNotIncreasing { .. } => "EnergyNotIncreasing",
        Refusal::AmplitudeNegative { .. } => "AmplitudeNegative",
    }
}

/// One value of every refusal the validator can produce.
fn every() -> Vec<Refusal> {
    vec![
        Refusal::Unreadable {
            detail: String::new(),
        },
        Refusal::NotADipoleTable {
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
            unit: String::new(),
            admitted: String::new(),
        },
        Refusal::UnknownConvention {
            found: String::new(),
            admitted: String::new(),
        },
        Refusal::TooFewSamples { found: 0 },
        Refusal::LengthDisagrees {
            energy: 0,
            amplitude: 0,
            phase: 0,
        },
        Refusal::ColumnNotFinite {
            column: String::new(),
            index: 0,
        },
        Refusal::EnergyNotIncreasing { index: 0 },
        Refusal::AmplitudeNegative { index: 0 },
    ]
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            refusal: Refusal::Unreadable {
                detail: String::new(),
            },
            why: "a table truncated by a write that ran out of room, which leaves a document \
                  that looks right until its last three bytes",
            near_miss: Box::new(|| {
                let bytes = edited(&[]);
                reading(&bytes[..bytes.len() - 3])
            }),
            repaired: Box::new(|| reading(&edited(&[]))),
            // The writer ends the document with a newline. A file that lost it
            // on the way through an editor is the same document, and a reader
            // that refused it would refuse tables for a reason the format does
            // not name.
            neighbour: Box::new(|| {
                let bytes = edited(&[]);
                reading(bytes.strip_suffix(b"\n").expect("the writer ends it"))
            }),
        },
        Case {
            refusal: Refusal::NotADipoleTable {
                found: String::new(),
            },
            why: "a trace header written into the place a case names its dipole table, which \
                  is two files one path apart",
            near_miss: Box::new(|| read_edited(&[("\"messlatte-dipole\"", "\"messlatte-trace\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // The name of another format inside a field that is not the format
            // name. A reader matching the text anywhere in the document would
            // refuse this.
            neighbour: Box::new(|| read_edited(&[("\"worked-example\"", "\"messlatte-trace\"")])),
        },
        Case {
            refusal: Refusal::UnknownMajorVersion {
                found: Version { major: 0, minor: 0 },
            },
            why: "a table written by a later version of this workspace, which is what a \
                  colleague's checkout produces",
            near_miss: Box::new(|| read_edited(&[("\"version\":\"1.0\"", "\"version\":\"2.0\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // A higher minor version means the same fields mean the same
            // things, so it reads. A refusal written against the version string
            // rather than against its major part would refuse this too.
            neighbour: Box::new(|| read_edited(&[("\"version\":\"1.0\"", "\"version\":\"1.7\"")])),
        },
        Case {
            refusal: Refusal::Field {
                path: String::new(),
                wanted: String::new(),
            },
            why: "a document assembled by hand with the energy unit left out, which is the \
                  field a writer forgets because the values look like a unit on their own",
            near_miss: Box::new(|| read_edited(&[("\"unit\":\"eV\",", "")])),
            repaired: Box::new(|| read_edited(&[])),
            // A field this reader does not know is ignored rather than refused,
            // which is the other half of the version rule. A reader refusing
            // every unexpected member would refuse this.
            neighbour: Box::new(|| {
                read_edited(&[("\"format\":", "\"computed\":\"2026-01-01\",\"format\":")])
            }),
        },
        Case {
            refusal: Refusal::Blank {
                field: String::new(),
            },
            why: "a table written from a template whose source line was never filled in, which \
                  is the field nobody misses until somebody asks where the numbers came from",
            near_miss: Box::new(|| {
                read_edited(&[(
                    "\"source\":\"a fixture rather than a table of anything real\"",
                    "\"source\":\"\"",
                )])
            }),
            repaired: Box::new(|| read_edited(&[])),
            // The format says nothing about how long a source has to be. A
            // refusal written against a shape rather than against emptiness
            // would refuse a short one.
            neighbour: Box::new(|| {
                read_edited(&[(
                    "\"source\":\"a fixture rather than a table of anything real\"",
                    "\"source\":\"a\"",
                )])
            }),
        },
        Case {
            refusal: Refusal::UnknownUnit {
                unit: String::new(),
                admitted: String::new(),
            },
            why: "an energy column left in the atomic units the numerics work in, which is a \
                  factor of twenty-seven away from the electronvolts a file admits",
            near_miss: Box::new(|| read_edited(&[("\"unit\":\"eV\"", "\"unit\":\"a.u.\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // The joule is the other unit an energy in a file may be stated in.
            // A refusal written against anything but the fixture's own unit
            // would refuse it.
            neighbour: Box::new(|| read_edited(&[("\"unit\":\"eV\"", "\"unit\":\"J\"")])),
        },
        Case {
            refusal: Refusal::UnknownConvention {
                found: String::new(),
                admitted: String::new(),
            },
            why: "a convention written without its sign, which is what somebody types when \
                  their own source only ever used one of the two",
            near_miss: Box::new(|| read_edited(&[("\"exp(+i phase)\"", "\"exp(i phase)\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // The other spelling is the one this format exists to convert, so a
            // refusal reaching it would delete every table written the other
            // way round.
            neighbour: Box::new(|| read_edited(&[("\"exp(+i phase)\"", "\"exp(-i phase)\"")])),
        },
        Case {
            refusal: Refusal::TooFewSamples { found: 0 },
            why: "a table cut down to the one energy a case needed, which leaves a file that \
                  looks complete and can interpolate nowhere",
            near_miss: Box::new(|| {
                writing(&changed(|table| {
                    table.energy = vec![20.0];
                    table.amplitude = vec![1.0];
                    table.phase = vec![-0.25];
                }))
            }),
            repaired: Box::new(|| round_trip(&base())),
            // Two samples are an interval, and the flat table that ships has
            // exactly two. A refusal written against a length below three would
            // delete it.
            neighbour: Box::new(|| {
                round_trip(&changed(|table| {
                    table.energy = vec![20.0, 40.0];
                    table.amplitude = vec![1.0, 1.5];
                    table.phase = vec![-0.25, -0.5];
                }))
            }),
        },
        Case {
            refusal: Refusal::LengthDisagrees {
                energy: 0,
                amplitude: 0,
                phase: 0,
            },
            why: "a phase column pasted one row short, which is what a spreadsheet selection \
                  that missed the last line produces",
            near_miss: Box::new(|| {
                writing(&changed(|table| {
                    table.phase.pop();
                }))
            }),
            repaired: Box::new(|| round_trip(&base())),
            // A table of a different length is not a table with a fault. A
            // check comparing against a fixed number of samples rather than
            // comparing the columns with each other would refuse this.
            neighbour: Box::new(|| {
                round_trip(&changed(|table| {
                    table.energy = vec![20.0, 30.0, 40.0, 50.0];
                    table.amplitude = vec![1.0, 2.0, 1.5, 1.25];
                    table.phase = vec![-0.25, -0.75, -0.5, -0.375];
                }))
            }),
        },
        Case {
            refusal: Refusal::ColumnNotFinite {
                column: String::new(),
                index: 0,
            },
            why: "an amplitude that is the result of a normalisation with nothing under it, \
                  which arrives as a value rather than as an error",
            // Only the writer can be shown this. JSON has no spelling for a
            // value that is not a number, so the reader refuses those bytes
            // before a column exists to judge.
            near_miss: Box::new(|| writing(&changed(|table| table.amplitude[1] = f64::NAN))),
            repaired: Box::new(|| round_trip(&base())),
            // An amplitude of exactly zero is a Cooper minimum and is the
            // feature a table with one in it exists to carry. A check written
            // against a falsy value rather than against finiteness would delete
            // it.
            neighbour: Box::new(|| round_trip(&changed(|table| table.amplitude[1] = 0.0))),
        },
        Case {
            refusal: Refusal::EnergyNotIncreasing { index: 0 },
            why: "two tables concatenated at a shared endpoint, which leaves one energy \
                  carrying two amplitudes and nothing saying which is meant",
            near_miss: Box::new(|| {
                read_edited(&[("\"values\":[20,30,40]", "\"values\":[20,30,30]")])
            }),
            repaired: Box::new(|| read_edited(&[])),
            // A coarse table is a table. A refusal written against a step that
            // is not the first step would delete every table sampled unevenly,
            // which is most published ones.
            neighbour: Box::new(|| {
                read_edited(&[("\"values\":[20,30,40]", "\"values\":[20,30,400]")])
            }),
        },
        Case {
            refusal: Refusal::AmplitudeNegative { index: 0 },
            why: "a real matrix element whose sign was left in the amplitude column instead of \
                  being written as a phase of pi, which is the same number to a plot and a \
                  different one to everything that reads the phase",
            near_miss: Box::new(|| {
                read_edited(&[("\"values\":[1,2,1.5]", "\"values\":[1,-2,1.5]")])
            }),
            repaired: Box::new(|| read_edited(&[])),
            // The phase column is signed and most of this fixture's phases are
            // negative already. A refusal written against a negative number
            // anywhere in the table would refuse a phase that ran below minus
            // pi.
            neighbour: Box::new(|| {
                read_edited(&[(
                    "\"values\":[-0.25,-0.75,-0.5]",
                    "\"values\":[-0.25,-3,-0.5]",
                )])
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

#[test]
fn the_worked_example_reads_as_the_document_says_it_does() {
    let table = Table::from_bytes(&worked_example_as_the_source_writes_it())
        .expect("the worked example is a table");
    assert_eq!(table.target, "worked-example");
    assert_eq!(table.unit, "eV");
    assert_eq!(table.energy, vec![20.0, 30.0, 40.0]);
    assert_eq!(table.amplitude, vec![1.0, 2.0, 1.5]);
    // The document's fourth column. The source wrote 0.25, 0.75 and 0.5 in the
    // other convention, and this is what they mean here.
    assert_eq!(table.phase, vec![-0.25, -0.75, -0.5]);
}

#[test]
fn the_two_midpoints_of_the_worked_example_are_the_documented_numbers() {
    let table = Table::from_bytes(&worked_example_as_the_source_writes_it())
        .expect("the worked example is a table");
    let at_25 = table.at(ev(25.0)).expect("25 eV is inside the range");
    close(at_25.amplitude, 1.5);
    close(at_25.phase, -0.5);
    let at_35 = table.at(ev(35.0)).expect("35 eV is inside the range");
    close(at_35.amplitude, 1.75);
    close(at_35.phase, -0.625);
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "the sample's own numbers are the claim. The interpolation is a weighted sum of \
              the two bracketing samples, so an energy landing on a sample carries weight one \
              and weight zero, and a tolerance here would pass a form that returns the sample \
              plus a rounding"
)]
fn an_energy_landing_on_a_sample_returns_that_sample_and_nothing_a_rounding_away() {
    // The energies are the ones a query built through the conversion layer
    // reproduces exactly, so what this measures is the interpolation and not
    // the round trip through atomic units.
    let stated: Vec<f64> = [20.0, 30.0, 40.0]
        .into_iter()
        .map(|value| ev(value).in_si("eV").expect("the electronvolt is admitted"))
        .collect();
    // The columns are not the base fixture's, and the reason is the mistake
    // this case exists to catch: an interpolation written as the lower sample
    // plus the difference times the weight. On values within a factor of two of
    // each other that form is exact at the upper end as well, so a fixture of
    // round numbers passes it and proves nothing. These three are pairs where
    // the difference rounds, so the two forms disagree in the last bit and the
    // wrong one reddens.
    let table = Table {
        energy: stated.clone(),
        amplitude: vec![0.2, 0.9, 0.3],
        phase: vec![-30.0, -13.9, -5.8],
        ..base()
    };
    for (index, value) in [20.0, 30.0, 40.0].into_iter().enumerate() {
        let found = table.at(ev(value)).expect("a tabulated energy is in range");
        assert_eq!(found.amplitude, table.amplitude[index], "at {value} eV");
        assert_eq!(found.phase, table.phase[index], "at {value} eV");
    }
}

#[test]
fn an_energy_outside_the_range_is_refused_at_both_ends_and_never_extrapolated() {
    let table = base();
    for outside in [19.9, 40.1, -1.0, 4000.0] {
        let found = table.at(ev(outside));
        assert!(
            matches!(found, Err(Lookup::OutsideRange { .. })),
            "{outside} eV is outside the table and gave {found:?}"
        );
    }
    for inside in [20.0, 21.0, 39.0, 40.0] {
        assert!(
            table.at(ev(inside)).is_ok(),
            "{inside} eV is inside the table"
        );
    }
}

#[test]
fn a_table_with_no_interval_answers_for_no_energy() {
    // Not reachable through the reader, which refuses all three of these. It is
    // reachable by building a table by hand, which the public fields allow.
    let one = Table {
        energy: vec![20.0],
        amplitude: vec![1.0],
        phase: vec![-0.25],
        ..base()
    };
    assert_eq!(one.at(ev(20.0)), Err(Lookup::Unusable));
    let ragged = Table {
        phase: vec![-0.25, -0.75],
        ..base()
    };
    assert_eq!(ragged.at(ev(25.0)), Err(Lookup::Unusable));
    let unconvertible = Table {
        unit: "a.u.".to_string(),
        ..base()
    };
    assert_eq!(unconvertible.at(ev(25.0)), Err(Lookup::Unusable));
}

/// A table describing one target as a pure group delay about its first sample,
/// written in whichever convention is asked for.
///
/// The delay goes in as a time and the phase values are derived from it, so the
/// loader is never handed the number the test is about.
fn delayed(delay: Time, convention: Convention) -> Vec<u8> {
    let low = ev(20.0).in_hartree();
    let high = ev(40.0).in_hartree();
    let ours = delay.in_atomic() * (high - low);
    let stated = ours
        * if convention == Convention::OURS {
            1.0
        } else {
            -1.0
        };
    document(
        convention.spelling(),
        "[20,40]",
        "[1,1]",
        &format!("[0,{stated}]"),
    )
}

/// The group delay a loaded table carries, in atomic units: the derivative of
/// the phase with respect to energy, which is exact for a two-sample table.
fn group_delay(table: &Table) -> Time {
    let low = Energy::from_si(table.energy[0], &table.unit).expect("the unit is admitted");
    let high = Energy::from_si(table.energy[1], &table.unit).expect("the unit is admitted");
    Time::from_atomic((table.phase[1] - table.phase[0]) / (high.in_hartree() - low.in_hartree()))
}

#[test]
fn a_delay_written_in_either_convention_loads_as_the_same_delay() {
    let delay = Time::from_si(100.0, "as").expect("the attosecond is admitted");

    for convention in [Convention::Positive, Convention::Negative] {
        let table = Table::from_bytes(&delayed(delay, convention))
            .expect("a delayed target is a table in either convention");
        let found = group_delay(&table)
            .in_si("as")
            .expect("the attosecond is admitted");
        close(found, 100.0);
    }
}

#[test]
fn a_table_read_in_the_wrong_convention_reports_the_opposite_delay() {
    // The mutation the conversion exists to prevent, and the reason it is worth
    // a field in the format. The bytes are one target delaying the photoelectron
    // by a hundred attoseconds; mislabelled, they read as one advancing it by
    // the same amount, and nothing about the amplitude moves.
    let delay = Time::from_si(100.0, "as").expect("the attosecond is admitted");
    let bytes = delayed(delay, Convention::Negative);
    let mislabelled = String::from_utf8(bytes)
        .expect("the document is text")
        .replace("exp(-i phase)", "exp(+i phase)")
        .into_bytes();

    let table = Table::from_bytes(&mislabelled).expect("the mislabelled table still reads");
    let found = group_delay(&table)
        .in_si("as")
        .expect("the attosecond is admitted");
    close(found, -100.0);
}

fn flat_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FLAT)
}

fn flat_table() -> Table {
    let bytes = fs::read(flat_path()).expect("the flat table is tracked at the path FLAT names");
    Table::from_bytes(&bytes).expect("the flat table this repository ships is a table")
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "one and zero are the claim. The flat table is a definition rather than a \
              measurement, so a value near one would mean the file had been edited into \
              something that is not the trivial case any more"
)]
fn the_table_that_ships_is_flat_across_its_whole_range() {
    let table = flat_table();
    assert_eq!(table.target, "flat");
    for energy in [0.0, 1.0, 137.5, 500.0, 999.0, 1000.0] {
        let found = table
            .at(ev(energy))
            .expect("the shipped range covers this energy");
        assert_eq!(found.amplitude, 1.0, "at {energy} eV");
        assert_eq!(found.phase, 0.0, "at {energy} eV");
    }
}

#[test]
fn the_table_that_ships_names_its_source_and_refuses_outside_its_range() {
    let table = flat_table();
    assert!(
        table.source.len() > 20,
        "a shipped table names where its numbers came from: {:?}",
        table.source
    );
    assert!(!table.normalisation.trim().is_empty());
    for outside in [-0.1, 1000.1] {
        assert!(
            matches!(table.at(ev(outside)), Err(Lookup::OutsideRange { .. })),
            "{outside} eV is outside the shipped range and has to be refused"
        );
    }
}

#[test]
fn the_table_that_ships_is_byte_for_byte_what_this_writer_produces() {
    // What makes a hash over a table mean something. A file whose bytes are not
    // the canonical form of its own content would hash differently after any
    // reader had been through it.
    let tracked = fs::read(flat_path()).expect("the flat table is tracked");
    let written = flat_table()
        .to_bytes()
        .expect("the flat table is one this crate would read");
    assert_eq!(
        String::from_utf8(written).expect("the document is text"),
        String::from_utf8(tracked).expect("the document is text")
    );
}
