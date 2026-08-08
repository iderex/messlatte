//! The trace file, and every assumption a reader is allowed to make about one
//! (#35).
//!
//! Each clause of "what a reader may assume" has a case here that trips it, and
//! each fixture is one edit away from the valid trace above it, so a case
//! proves the clause it names rather than proving that a document full of
//! mistakes is refused for some reason.

use messlatte_formats::npy::Array;
use messlatte_formats::trace::{Axis, Cells, Electron, Refusal, Trace, VERSION};

/// A small valid trace: two electron samples, three delays, and a delay axis
/// with a gap in it.
fn trace() -> Trace {
    Trace {
        case: "fixture-01".to_string(),
        electron_quantity: Electron::Energy,
        electron: Axis {
            unit: "eV".to_string(),
            values: vec![20.0, 20.5],
        },
        delay: Axis {
            unit: "fs".to_string(),
            // Not uniform, and that is the point. The gap between the second
            // and the third sample is four times the first step.
            values: vec![-1.0, -0.5, 1.5],
        },
        cells: Cells::Counts,
        values: Array::new(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).expect("six cells"),
    }
}

fn written() -> (Vec<u8>, Vec<u8>) {
    trace().to_bytes().expect("the fixture trace is writable")
}

#[test]
fn the_header_is_the_document_this_format_declares() {
    // Written out rather than described, because the schema and the order are
    // what an implementation in another language has to match, and a case that
    // only checked the fields would pass while the order moved underneath it.
    let (header, _) = written();
    assert_eq!(
        String::from_utf8(header).expect("the header is text"),
        "{\"case\":\"fixture-01\",\
         \"cells\":{\"quantity\":\"counts\"},\
         \"delay\":{\"unit\":\"fs\",\"values\":[-1,-0.5,1.5]},\
         \"electron\":{\"quantity\":\"energy\",\"unit\":\"eV\",\"values\":[20,20.5]},\
         \"format\":{\"name\":\"messlatte-trace\",\"version\":\"1.0\"}}\n"
    );
}

#[test]
fn a_trace_written_and_read_again_is_the_same_bytes() {
    let (header, array) = written();
    let read = Trace::from_bytes(&header, &array).expect("what this wrote is readable");
    let (again, array_again) = read.to_bytes().expect("writable");
    assert_eq!(again, header);
    assert_eq!(array_again, array);
    assert_eq!(read, trace());
}

#[test]
fn a_delay_axis_with_a_gap_in_it_is_a_case_rather_than_a_defect() {
    // Sampling completeness is one of the axes of the study, so this has to
    // read. A reader that took the step from the first two samples would place
    // the third one at zero instead of at 1.5.
    let (header, array) = written();
    let read = Trace::from_bytes(&header, &array).expect("a gapped delay axis reads");
    assert_eq!(read.delay.values.len(), 3);
}

#[test]
fn a_normalised_trace_carries_what_it_was_divided_by() {
    let mut normalised = trace();
    normalised.cells = Cells::Normalised {
        normalisation: "divided by the total counts in the trace".to_string(),
    };
    let (header, array) = normalised.to_bytes().expect("writable");
    assert_eq!(
        Trace::from_bytes(&header, &array).expect("readable").cells,
        normalised.cells
    );
}

#[test]
fn a_normalisation_nobody_stated_is_refused() {
    // The near-miss: the quantity says the cells were divided by something and
    // the sentence saying what is empty. Without this the file would say
    // "normalised" and mean an arbitrary scale, which is the one thing the
    // format says a cell never carries.
    let mut trace = trace();
    trace.cells = Cells::Normalised {
        normalisation: "  ".to_string(),
    };
    let refusals = trace.to_bytes().expect_err("an unstated normalisation");
    assert!(
        matches!(refusals.as_slice(), [Refusal::Normalisation { .. }]),
        "{refusals:?}"
    );
}

#[test]
fn counts_that_also_state_a_normalisation_are_refused() {
    let refusals = read_with(&[(
        "\"cells\":{\"quantity\":\"counts\"}",
        "\"cells\":{\"normalisation\":\"by the peak\",\"quantity\":\"counts\"}",
    )])
    .expect_err("counts and a normalisation");
    assert!(
        matches!(refusals.as_slice(), [Refusal::Normalisation { .. }]),
        "{refusals:?}"
    );
}

#[test]
fn an_axis_that_does_not_increase_is_refused() {
    let refusals = read_with(&[("[-1,-0.5,1.5]", "[-1,1.5,-0.5]")]).expect_err("out of order");
    assert!(
        refusals.contains(&Refusal::AxisNotIncreasing {
            axis: "delay".to_string(),
            index: 2
        }),
        "{refusals:?}"
    );
}

#[test]
fn an_axis_that_repeats_a_sample_is_refused() {
    // A repeated sample is the near-miss for "strictly": an axis that never
    // decreases would pass a check written with the wrong comparison, and two
    // cells at one delay are two measurements of one thing.
    let refusals = read_with(&[("[-1,-0.5,1.5]", "[-1,-0.5,-0.5]")]).expect_err("a repeat");
    assert!(
        refusals.contains(&Refusal::AxisNotIncreasing {
            axis: "delay".to_string(),
            index: 2
        }),
        "{refusals:?}"
    );
}

#[test]
fn an_axis_with_no_samples_is_refused() {
    let refusals = read_with(&[("[20,20.5]", "[]")]).expect_err("an empty axis");
    assert!(
        refusals.contains(&Refusal::EmptyAxis {
            axis: "electron".to_string()
        }),
        "{refusals:?}"
    );
}

#[test]
fn a_unit_outside_the_set_the_format_admits_is_refused() {
    // Atomic units are what the numerics work in and are not admitted in a
    // file, which is #13's boundary. A reader that accepted the name would take
    // a number in atomic units for one in electronvolts and be wrong by a
    // factor of twenty-seven.
    let refusals =
        read_with(&[("\"unit\":\"eV\"", "\"unit\":\"a.u.\"")]).expect_err("atomic units");
    assert!(
        refusals.contains(&Refusal::UnknownUnit {
            axis: "electron".to_string(),
            unit: "a.u.".to_string(),
            admitted: "J, eV".to_string()
        }),
        "{refusals:?}"
    );
}

#[test]
fn an_array_that_is_not_the_shape_of_the_axes_is_refused() {
    let (header, _) = written();
    let array = Array::new(3, 2, vec![1.0; 6])
        .expect("six cells the other way round")
        .to_bytes()
        .expect("writable");
    let refusals = Trace::from_bytes(&header, &array).expect_err("a transposed array");
    // The near-miss: the same six values, so a reader comparing only the count
    // would accept a trace whose axes are swapped.
    assert!(
        refusals.contains(&Refusal::ShapeDisagrees {
            rows: 3,
            columns: 2,
            electron: 2,
            delay: 3
        }),
        "{refusals:?}"
    );
}

#[test]
fn a_cell_that_is_not_a_number_is_refused() {
    // The sentinel this format does not have. A missing sample is absent from
    // the axis, so a value standing for one cannot get in through the array
    // either.
    let mut trace = trace();
    trace.values.values[4] = f64::NAN;
    let refusals = trace.to_bytes().expect_err("a missing sample as a value");
    assert!(
        refusals.contains(&Refusal::CellNotFinite { row: 1, column: 1 }),
        "{refusals:?}"
    );
}

#[test]
fn a_trace_with_no_case_identifier_is_refused() {
    let refusals = read_with(&[("\"fixture-01\"", "\"\"")]).expect_err("no identifier");
    assert!(refusals.contains(&Refusal::EmptyCase), "{refusals:?}");
}

#[test]
fn a_document_that_is_not_a_trace_header_is_refused() {
    let refusals =
        read_with(&[("\"messlatte-trace\"", "\"messlatte-truth\"")]).expect_err("another format");
    assert!(
        matches!(
            refusals.as_slice(),
            [Refusal::NotATraceHeader { found }] if found == "messlatte-truth"
        ),
        "{refusals:?}"
    );
}

#[test]
fn a_major_version_this_reader_does_not_know_is_refused() {
    let refusals = read_with(&[("\"1.0\"", "\"2.0\"")]).expect_err("a later major version");
    assert!(
        matches!(refusals.as_slice(), [Refusal::UnknownMajorVersion { .. }]),
        "{refusals:?}"
    );
}

#[test]
fn a_higher_minor_version_reads_with_its_unknown_fields_ignored() {
    // The other half of the version rule. A field added under a minor version
    // does not change what the fields here mean, so this reads and drops it,
    // and what it writes back is a version 1.0 trace rather than a claim to
    // have understood the field.
    let read = read_with(&[
        ("\"1.0\"", "\"1.7\""),
        ("\"case\":", "\"acquired\":\"2026-01-01\",\"case\":"),
    ])
    .expect("a higher minor version reads");
    assert_eq!(read, trace());
    let (header, _) = read.to_bytes().expect("writable");
    let header = String::from_utf8(header).expect("text");
    assert!(
        header.contains(&format!("\"version\":\"{VERSION}\"")),
        "{header}"
    );
    assert!(!header.contains("acquired"), "{header}");
}

#[test]
fn a_header_that_is_missing_a_field_says_which_one() {
    let refusals = read_with(&[("\"unit\":\"fs\",", "")]).expect_err("no delay unit");
    assert!(
        matches!(
            refusals.as_slice(),
            [Refusal::Field { path, .. }] if path == "delay.unit"
        ),
        "{refusals:?}"
    );
}

#[test]
fn an_electron_axis_of_neither_quantity_is_refused() {
    let refusals = read_with(&[("\"quantity\":\"energy\"", "\"quantity\":\"wavelength\"")])
        .expect_err("neither");
    assert!(
        matches!(
            refusals.as_slice(),
            [Refusal::Field { path, .. }] if path == "electron.quantity"
        ),
        "{refusals:?}"
    );
}

#[test]
fn a_header_that_is_not_a_document_is_refused() {
    let (_, array) = written();
    let refusals = Trace::from_bytes(b"{not json", &array).expect_err("not a document");
    assert!(
        matches!(refusals.as_slice(), [Refusal::Unreadable { what, .. }] if what == "header"),
        "{refusals:?}"
    );
}

#[test]
fn an_array_that_is_not_one_is_refused() {
    let (header, _) = written();
    let refusals = Trace::from_bytes(&header, b"not an array").expect_err("not an array");
    assert!(
        matches!(refusals.as_slice(), [Refusal::Unreadable { what, .. }] if what == "array"),
        "{refusals:?}"
    );
}

#[test]
fn everything_wrong_with_a_trace_is_reported_at_once() {
    // A file with two mistakes is one file somebody fixes once. A reader that
    // stopped at the first would send them round the loop twice.
    let refusals = read_with(&[
        ("[-1,-0.5,1.5]", "[-1,-0.5,-0.5]"),
        ("\"unit\":\"eV\"", "\"unit\":\"a.u.\""),
    ])
    .expect_err("two mistakes");
    assert_eq!(refusals.len(), 2, "{refusals:?}");
}

/// The fixture trace's header with substitutions applied, read back against the
/// fixture's own array.
fn read_with(edits: &[(&str, &str)]) -> Result<Trace, Vec<Refusal>> {
    let (header, array) = written();
    let mut header = String::from_utf8(header).expect("the header is text");
    for (from, to) in edits {
        assert!(
            header.contains(from),
            "the fixture header does not carry {from:?}"
        );
        header = header.replace(from, to);
    }
    Trace::from_bytes(header.as_bytes(), &array)
}
