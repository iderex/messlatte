//! The register of refusals, and the three legs each one ships with (#40).
//!
//! A validator is worth what its refusals are worth, so this file is arranged
//! around the refusals rather than around the clauses of the format. Every
//! refusal the trace validator can produce appears here with three legs: a
//! near-miss somebody would plausibly write, the same input with that one fault
//! repaired, and a one-change neighbour that must read. The second leg is what
//! shows the refusal fired for the fault rather than for the fixture, and the
//! third is what stops a refusal being broader than its reason.
//!
//! The register is held closed by [`name`], which matches on `Refusal`
//! exhaustively. A variant added to the validator stops this file compiling,
//! and the arm that repairs it sits beside the sample list and the cases that
//! have to name it.
//!
//! WHAT THIS DOES NOT CATCH, and it is one thing rather than a general
//! weakness. A variant added to `Refusal` and given an arm in [`name`] and an
//! entry in [`every`], but no case, is caught: the two sets are compared and
//! the missing case reddens this file. A variant given an arm in [`name`] and
//! left out of [`every`] is NOT caught, because Rust on the pinned toolchain
//! has no way to enumerate the variants of an enum, so the sample list is the
//! one edge held by hand. What reduces it is that the arm and the sample sit in
//! the same screen of this file and the compiler sends the author here.
//!
//! The scope is the trace validator alone. It is the only validator in the
//! tree; the truth, the declaration, the submission and the case index are
//! #36, #37, #38 and #39 and none of them exists yet, so #40 is not discharged
//! by this file and nothing here reaches them.
//!
//! `crates/formats/tests/trace.rs` reads the same format from the other end,
//! one case per clause of what a reader may assume. The fixtures here are
//! deliberately self-contained rather than shared with it, because a register
//! whose entries are proved by cases in another file is a register that goes
//! green when that file is thinned.

use std::collections::BTreeSet;

use messlatte_formats::npy::Array;
use messlatte_formats::trace::{Axis, Cells, Electron, Refusal, Trace, Version};

/// The fixture every case below is one edit away from: two electron samples,
/// three delays, and a delay axis with a gap in it so that no case here can
/// pass by assuming a uniform one.
fn base() -> Trace {
    Trace {
        case: "fixture-01".to_string(),
        electron_quantity: Electron::Energy,
        electron: Axis {
            unit: "eV".to_string(),
            values: vec![20.0, 20.5],
        },
        delay: Axis {
            unit: "fs".to_string(),
            values: vec![-1.0, -0.5, 1.5],
        },
        cells: Cells::Counts,
        values: Array::new(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).expect("six cells"),
    }
}

/// What the writer says about a trace, and nothing where it writes one.
fn writing(trace: &Trace) -> Vec<Refusal> {
    match trace.to_bytes() {
        Ok(_) => Vec::new(),
        Err(found) => found,
    }
}

/// What the pair of files says about a trace once it has been through both
/// ends. A refusal from the writer is returned as it stands, because a trace
/// the writer will not emit is not one the reader can be asked about.
fn round_trip(trace: &Trace) -> Vec<Refusal> {
    let (header, array) = match trace.to_bytes() {
        Ok(pair) => pair,
        Err(found) => return found,
    };
    reading(&header, &array)
}

/// What the reader says about two files.
fn reading(header: &[u8], array: &[u8]) -> Vec<Refusal> {
    match Trace::from_bytes(header, array) {
        Ok(_) => Vec::new(),
        Err(found) => found,
    }
}

/// The base fixture's header with substitutions applied, beside its own array.
fn edited(edits: &[(&str, &str)]) -> (Vec<u8>, Vec<u8>) {
    let (header, array) = base().to_bytes().expect("the base fixture is writable");
    let mut header = String::from_utf8(header).expect("the header is text");
    for (from, to) in edits {
        assert!(
            header.contains(from),
            "the base fixture's header does not carry {from:?}"
        );
        header = header.replace(from, to);
    }
    (header.into_bytes(), array)
}

/// The base fixture read back with substitutions applied to its header.
fn read_edited(edits: &[(&str, &str)]) -> Vec<Refusal> {
    let (header, array) = edited(edits);
    reading(&header, &array)
}

/// The base fixture with one field of the trace replaced.
fn changed(edit: impl FnOnce(&mut Trace)) -> Trace {
    let mut trace = base();
    edit(&mut trace);
    trace
}

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
///
/// Exhaustive on purpose. A refusal added to the validator stops this
/// compiling, and repairing it means naming the variant here, adding a sample
/// to [`every`] and writing a case for it below.
fn name(refusal: &Refusal) -> &'static str {
    match refusal {
        Refusal::Unreadable { .. } => "Unreadable",
        Refusal::NotATraceHeader { .. } => "NotATraceHeader",
        Refusal::UnknownMajorVersion { .. } => "UnknownMajorVersion",
        Refusal::Field { .. } => "Field",
        Refusal::EmptyAxis { .. } => "EmptyAxis",
        Refusal::AxisNotIncreasing { .. } => "AxisNotIncreasing",
        Refusal::AxisNotFinite { .. } => "AxisNotFinite",
        Refusal::UnknownUnit { .. } => "UnknownUnit",
        Refusal::Normalisation { .. } => "Normalisation",
        Refusal::ShapeDisagrees { .. } => "ShapeDisagrees",
        Refusal::CellNotFinite { .. } => "CellNotFinite",
        Refusal::EmptyCase => "EmptyCase",
    }
}

/// One value of every refusal the validator can produce.
fn every() -> Vec<Refusal> {
    vec![
        Refusal::Unreadable {
            what: String::new(),
            detail: String::new(),
        },
        Refusal::NotATraceHeader {
            found: String::new(),
        },
        Refusal::UnknownMajorVersion {
            found: Version { major: 0, minor: 0 },
        },
        Refusal::Field {
            path: String::new(),
            wanted: String::new(),
        },
        Refusal::EmptyAxis {
            axis: String::new(),
        },
        Refusal::AxisNotIncreasing {
            axis: String::new(),
            index: 0,
        },
        Refusal::AxisNotFinite {
            axis: String::new(),
            index: 0,
        },
        Refusal::UnknownUnit {
            axis: String::new(),
            unit: String::new(),
            admitted: String::new(),
        },
        Refusal::Normalisation {
            detail: String::new(),
        },
        Refusal::ShapeDisagrees {
            rows: 0,
            columns: 0,
            electron: 0,
            delay: 0,
        },
        Refusal::CellNotFinite { row: 0, column: 0 },
        Refusal::EmptyCase,
    ]
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            refusal: Refusal::Unreadable {
                what: String::new(),
                detail: String::new(),
            },
            why: "a header truncated by a write that ran out of room, which leaves a document \
                  that looks right until its last three bytes",
            near_miss: Box::new(|| {
                let (header, array) = edited(&[]);
                reading(&header[..header.len() - 3], &array)
            }),
            repaired: Box::new(|| {
                let (header, array) = edited(&[]);
                reading(&header, &array)
            }),
            // The writer ends the document with a newline. A file that lost it
            // on the way through an editor is still the same document, and a
            // reader that refused it would refuse traces for a reason the
            // format does not name.
            neighbour: Box::new(|| {
                let (header, array) = edited(&[]);
                reading(
                    header.strip_suffix(b"\n").expect("the writer ends it"),
                    &array,
                )
            }),
        },
        Case {
            refusal: Refusal::NotATraceHeader {
                found: String::new(),
            },
            why: "the truth header written into the trace's place in a case directory, which \
                  is two files one command apart",
            near_miss: Box::new(|| read_edited(&[("\"messlatte-trace\"", "\"messlatte-truth\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // The name of another format inside a field that is not the format
            // name. A reader matching the text anywhere in the document would
            // refuse this.
            neighbour: Box::new(|| read_edited(&[("\"fixture-01\"", "\"messlatte-truth\"")])),
        },
        Case {
            refusal: Refusal::UnknownMajorVersion {
                found: Version { major: 0, minor: 0 },
            },
            why: "a trace written by a later version of this workspace, which is what a \
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
            why: "a header assembled by hand with the delay unit left out, which is the field \
                  a writer forgets because the values look like a unit on their own",
            near_miss: Box::new(|| read_edited(&[("\"unit\":\"fs\",", "")])),
            repaired: Box::new(|| read_edited(&[])),
            // A field this reader does not know is ignored rather than refused,
            // which is the other half of the version rule. A reader refusing
            // every unexpected member would refuse this.
            neighbour: Box::new(|| {
                read_edited(&[("\"case\":", "\"acquired\":\"2026-01-01\",\"case\":")])
            }),
        },
        Case {
            refusal: Refusal::EmptyAxis {
                axis: String::new(),
            },
            why: "a momentum window applied to a trace until nothing was left inside it, which \
                  produces an axis with no samples and an array to match",
            near_miss: Box::new(|| {
                writing(&changed(|trace| {
                    trace.electron.values.clear();
                    trace.values = Array::new(0, 3, Vec::new()).expect("no cells");
                }))
            }),
            repaired: Box::new(|| round_trip(&base())),
            // One sample is a thin trace and not an empty one. A refusal
            // written against a length below two would delete it.
            neighbour: Box::new(|| {
                round_trip(&changed(|trace| {
                    trace.electron.values = vec![20.0];
                    trace.values = Array::new(1, 3, vec![1.0, 2.0, 3.0]).expect("three cells");
                }))
            }),
        },
        Case {
            refusal: Refusal::AxisNotIncreasing {
                axis: String::new(),
                index: 0,
            },
            why: "a delay axis whose last two samples repeat, which is what concatenating two \
                  scans that share an endpoint produces",
            near_miss: Box::new(|| read_edited(&[("[-1,-0.5,1.5]", "[-1,-0.5,-0.5]")])),
            repaired: Box::new(|| read_edited(&[])),
            // Sampling completeness is one of the axes of the study, so a gap
            // is a case. A refusal written against a step that is not the first
            // step would delete the incomplete-sampling half of it.
            neighbour: Box::new(|| read_edited(&[("[-1,-0.5,1.5]", "[-1,-0.5,40]")])),
        },
        Case {
            refusal: Refusal::AxisNotFinite {
                axis: String::new(),
                index: 0,
            },
            why: "a delay axis carrying the result of a division that had no samples under it, \
                  which arrives as a value rather than as an error",
            // Only the writer can be shown this. The header is JSON and JSON
            // has no spelling for a value that is not a number, so the reader
            // refuses these bytes before an axis exists to judge.
            near_miss: Box::new(|| writing(&changed(|trace| trace.delay.values[1] = f64::NAN))),
            repaired: Box::new(|| round_trip(&base())),
            // Zero delay is the sample a streaking scan is built around. A
            // check written against a falsy value rather than against
            // finiteness would refuse it.
            neighbour: Box::new(|| round_trip(&changed(|trace| trace.delay.values[1] = 0.0))),
        },
        Case {
            refusal: Refusal::UnknownUnit {
                axis: String::new(),
                unit: String::new(),
                admitted: String::new(),
            },
            why: "an electron axis left in the atomic units the numerics work in, which is a \
                  factor of twenty-seven away from the electronvolts the field admits",
            near_miss: Box::new(|| read_edited(&[("\"unit\":\"eV\"", "\"unit\":\"a.u.\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // The joule is the other unit the energy axis admits. A refusal
            // written against anything but the fixture's own unit would refuse
            // it.
            neighbour: Box::new(|| read_edited(&[("\"unit\":\"eV\"", "\"unit\":\"J\"")])),
        },
        Case {
            refusal: Refusal::Normalisation {
                detail: String::new(),
            },
            why: "a trace divided by something on the way out with the quantity updated and \
                  the sentence saying what never written",
            near_miss: Box::new(|| {
                read_edited(&[(
                    "{\"quantity\":\"counts\"}",
                    "{\"quantity\":\"normalised-counts\"}",
                )])
            }),
            repaired: Box::new(|| read_edited(&[])),
            // A normalised trace that carries its sentence is the case this
            // format exists to admit, so a refusal reaching every normalised
            // trace would remove it.
            neighbour: Box::new(|| {
                read_edited(&[(
                    "{\"quantity\":\"counts\"}",
                    "{\"normalisation\":\"the largest cell in the trace\",\"quantity\":\
                     \"normalised-counts\"}",
                )])
            }),
        },
        Case {
            refusal: Refusal::ShapeDisagrees {
                rows: 0,
                columns: 0,
                electron: 0,
                delay: 0,
            },
            why: "an array stored the other way round, which carries the same six values and \
                  the same product, so a reader counting cells accepts it",
            near_miss: Box::new(|| {
                writing(&changed(|trace| {
                    trace.values = Array::new(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
                        .expect("six cells the other way round");
                }))
            }),
            repaired: Box::new(|| round_trip(&base())),
            // A square trace agrees with its axes as much as a rectangular one
            // does. A check comparing the two axis lengths to each other rather
            // than to the array would pass the fixture and refuse this.
            neighbour: Box::new(|| {
                round_trip(&changed(|trace| {
                    trace.electron.values = vec![20.0, 20.5, 21.0];
                    trace.values =
                        Array::new(3, 3, (0..9).map(f64::from).collect()).expect("nine cells");
                }))
            }),
        },
        Case {
            refusal: Refusal::CellNotFinite { row: 0, column: 0 },
            why: "a cell standing in for a sample the detector never delivered, which is the \
                  sentinel this format refuses to carry",
            near_miss: Box::new(|| writing(&changed(|trace| trace.values.values[4] = f64::NAN))),
            repaired: Box::new(|| round_trip(&base())),
            // A bin that counted nothing is a measurement of zero and not a
            // missing sample. A check written against a falsy cell would delete
            // every empty bin in the trace.
            neighbour: Box::new(|| round_trip(&changed(|trace| trace.values.values[4] = 0.0))),
        },
        Case {
            refusal: Refusal::EmptyCase,
            why: "a header written from a template whose case identifier was never filled in",
            near_miss: Box::new(|| read_edited(&[("\"fixture-01\"", "\"\"")])),
            repaired: Box::new(|| read_edited(&[])),
            // The identifier is the case directory's name and this format says
            // nothing else about it. A refusal written against a shape rather
            // than against emptiness would refuse a short one.
            neighbour: Box::new(|| read_edited(&[("\"fixture-01\"", "\"a\"")])),
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
