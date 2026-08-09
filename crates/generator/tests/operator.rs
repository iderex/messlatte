//! The streaking operator, the form it prints and the one limit checked here
//! (#42).
//!
//! `docs/format/streaking-operator.md` is the authority for the model. The
//! first case below reads that document and requires it to carry the printed
//! form byte for byte, so a model that moves in the code and not in the
//! document reddens rather than being noticed by a reader later.
//!
//! What is checked about the numbers, and what is not. With the streaking field
//! off the trace is the pulse's spectrum shifted by the ionisation potential,
//! and that limit is checked here against a closed form the operator does not
//! compute: a Gaussian pulse has a Gaussian spectrum, and the ratios between
//! cells are what a normalisation cannot move. The four limits #46 asks for,
//! their tolerances and the second slower quadrature are not here, and this
//! file must not be read as covering them.

use core::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

use messlatte_formats::dipole::Table;
use messlatte_formats::trace::{Cells, Electron};
use messlatte_generator::field::StreakingField;
use messlatte_generator::operator::{self, Grids, Pulse, Refusal, Streaking, Target};

/// The carrier of the fixture pulse, in atomic units.
const CARRIER: f64 = 1.5;

/// Its Gaussian width in time, in atomic units.
const WIDTH: f64 = 20.0;

/// The ionisation potential of the fixture target, in atomic units.
const IONISATION_POTENTIAL: f64 = 0.5;

/// One count as a double. The lint set refuses the cast because a `usize` is
/// wider than a double's mantissa, and every count in this file is a grid size.
fn whole(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("a grid this file builds fits in a u32"))
}

#[track_caller]
fn close(left: f64, right: f64, tolerance: f64) {
    let scale = left.abs().max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "{left:e} and {right:e} differ by more than {tolerance:e} of the larger"
    );
}

/// The flat table this repository ships, read from the tree rather than built
/// here, so this exercises the same bytes an operator would.
fn flat_dipole() -> Table {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(messlatte_formats::dipole::FLAT);
    let bytes = fs::read(path).expect("the flat table is tracked");
    Table::from_bytes(&bytes).expect("the flat table reads")
}

fn target() -> Target {
    Target {
        ionisation_potential: IONISATION_POTENTIAL,
        dipole: flat_dipole(),
    }
}

/// A Gaussian pulse with a carrier, on a grid six widths either side of its
/// peak.
///
/// Six rather than a round number, and that is where the tolerance below comes
/// from. A Gaussian truncated at n widths leaves about exp(-n^2 / 2) of its
/// amplitude outside the window, which the spectrum carries as a floor: at five
/// widths that is four parts in a million and the closed form below disagreed
/// with the operator by three, and at six it is under two parts in a hundred
/// million.
///
/// The carrier is `exp(-i * CARRIER * s)`, which is the sign that makes the
/// amplitude's own `exp(+i * (p^2 / 2 + Ip) * s)` beat against it. The
/// photoelectron energy the pulse is centred on is therefore the carrier less
/// the ionisation potential.
fn gaussian_pulse(samples: usize) -> Pulse {
    gaussian_pulse_of(samples, WIDTH)
}

/// The same, at a width of the caller's choosing.
fn gaussian_pulse_of(samples: usize, width: f64) -> Pulse {
    let span = 6.0 * width;
    let step = 2.0 * span / whole(samples - 1);
    let mut real = Vec::with_capacity(samples);
    let mut imaginary = Vec::with_capacity(samples);
    for index in 0..samples {
        let time = -span + step * whole(index);
        let envelope = (-time * time / (2.0 * width * width)).exp();
        real.push(envelope * (CARRIER * time).cos());
        imaginary.push(-envelope * (CARRIER * time).sin());
    }
    Pulse {
        first: -span,
        step,
        real,
        imaginary,
    }
}

/// A grid of momenta about the one the pulse is centred on.
fn momenta(count: usize, spread: f64) -> Vec<f64> {
    let centre = (2.0 * (CARRIER - IONISATION_POTENTIAL)).sqrt();
    (0..count)
        .map(|index| {
            let fraction = whole(index) / whole(count - 1) - 0.5;
            centre + spread * fraction
        })
        .collect()
}

/// The field that is not there. An amplitude of zero is a field, and the phase
/// accumulated in it is exactly zero, so this is the limit rather than an
/// approximation to it.
fn no_field() -> Streaking {
    let mut streaking = Streaking::new(
        StreakingField::new(0.0, 0.05, 4.0, 0.0).expect("a zero amplitude is a field"),
    );
    streaking.intervals_per_cycle = 8;
    streaking
}

fn document() -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("format")
        .join("streaking-operator.md");
    fs::read_to_string(path).expect("the operator document is tracked")
}

#[test]
fn the_document_holds_the_printed_form_byte_for_byte() {
    assert!(
        document().contains(operator::PRINTED_FORM),
        "docs/format/streaking-operator.md no longer carries the form the operator prints, so \
         one of the two has moved without the other"
    );
}

#[test]
fn the_printed_form_names_every_omission_and_every_symbol() {
    let printed = operator::PRINTED_FORM;
    for omission in [
        "no depletion of the ground state",
        "no space charge",
        "no propagation of either field through the target",
        "no vector character beyond the single declared polarisation direction",
        "one active electron",
        "one final state per target",
    ] {
        assert!(
            printed.contains(omission),
            "the form does not print {omission:?}"
        );
    }
    for symbol in ["d_amp", "d_phase", "phi(p, t)", "A(t)", "E(s)", "Ip"] {
        assert!(
            printed.contains(symbol),
            "the form names {symbol} in its expression and not in its symbol table"
        );
    }
}

#[test]
fn with_the_field_off_every_delay_carries_the_same_column() {
    // Nothing in the model depends on the delay once the potential is zero, so
    // this is bit equality and not a tolerance. A column that moved would mean
    // the delay had reached the trace through something other than the field.
    let trace = operator::trace(
        "fixture-no-field",
        &gaussian_pulse(401),
        no_field(),
        &target(),
        &Grids {
            momenta: momenta(9, 1.0),
            delays: vec![-40.0, 0.0, 55.0],
        },
    )
    .expect("the zero-field case is a trace");

    for row in 0..trace.values.rows {
        let first = trace.values.values[row * trace.values.columns];
        for column in 1..trace.values.columns {
            let cell = trace.values.values[row * trace.values.columns + column];
            assert!(
                cell.to_bits() == first.to_bits(),
                "row {row} column {column} is {cell:e} against {first:e} in column zero"
            );
        }
    }
}

#[test]
fn with_the_field_off_the_trace_is_the_pulse_spectrum_shifted_by_the_ionisation_potential() {
    // The closed form the operator does not compute. A Gaussian of width WIDTH
    // in time has a spectral intensity exp(-WIDTH^2 * (w - CARRIER)^2), and the
    // photoelectron at momentum p sits at w = p^2 / 2 + Ip. Ratios between
    // cells are compared rather than cells, because the trace carries no
    // absolute scale and the normalisation divides by whichever cell happened
    // to be largest.
    // The spread is a few spectral widths and not more. A Gaussian five widths
    // out is below what a window five widths wide can carry, so a grid reaching
    // further would compare the closed form against this fixture's truncation
    // rather than against the operator.
    let grid = momenta(11, 0.2);
    let trace = operator::trace(
        "fixture-no-field",
        &gaussian_pulse(1601),
        no_field(),
        &target(),
        &Grids {
            momenta: grid.clone(),
            delays: vec![0.0],
        },
    )
    .expect("the zero-field case is a trace");

    let expected = |momentum: f64| {
        let offset = momentum * momentum / 2.0 + IONISATION_POTENTIAL - CARRIER;
        (-WIDTH * WIDTH * offset * offset).exp()
    };
    let reference = grid.len() / 2;
    for (index, momentum) in grid.iter().enumerate() {
        let found = trace.values.values[index] / trace.values.values[reference];
        // A part in a million, two orders above the window's own floor.
        close(found, expected(*momentum) / expected(grid[reference]), 1e-6);
    }
}

#[test]
fn the_first_moment_follows_minus_the_vector_potential() {
    // The streaking relation. An electron born at t with speed v0 leaves with a
    // canonical momentum of v0 - A(t), so the first moment of the trace in
    // momentum has to move against the potential and not with it. Which of the
    // two it does is the whole content of the sign the accumulated phase enters
    // with, and it is #123: the operator was first written with the other one,
    // and the trace still looked like a streaking trace.
    //
    // The two delays are half a period apart, where the potential of a pulse
    // with a carrier-envelope phase of zero is equal and opposite. Their
    // DIFFERENCE is what is compared, because the squared-potential term
    // contributes a shift that does not depend on the momentum and is the same
    // at both, so it cancels there and does not in either one alone. What is
    // left is the relation above.
    //
    // The pulse is short against the streaking period, which is what the
    // streaking picture describes. The long fixture used elsewhere in this file
    // averages the potential over a good fraction of a cycle and is not one.
    let field = StreakingField::new(0.15, 0.05, 6.0, 0.0).expect("a field");
    let mut streaking = Streaking::new(field);
    streaking.intervals_per_cycle = 256;
    let grid = momenta(201, 1.2);
    let half = field.period() / 2.0;

    let moments = |streaking: Streaking| {
        let trace = operator::trace(
            "fixture-streaked",
            &gaussian_pulse_of(801, 4.0),
            streaking,
            &target(),
            &Grids {
                momenta: grid.clone(),
                delays: vec![0.0, half],
            },
        )
        .expect("the streaked case is a trace");
        let mut found = Vec::new();
        for column in 0..trace.values.columns {
            let mut weight = 0.0;
            let mut total = 0.0;
            for (index, momentum) in grid.iter().enumerate() {
                let cell = trace.values.values[index * trace.values.columns + column];
                weight += cell;
                total += cell * momentum;
            }
            found.push(total / weight);
        }
        found
    };

    let streaked = moments(streaking);
    let quiet = moments(no_field());

    // The field-free case is the same at both delays, which is the other file's
    // case restated here so that what follows is a difference of differences
    // rather than a difference against a number this case assumes.
    close(quiet[0], quiet[1], 1e-12);

    let measured = streaked[0] - streaked[1];
    let wanted = -(field.potential(0.0) - field.potential(half));
    assert!(
        measured < 0.0,
        "the first moment moved with the potential rather than against it: {measured:e}"
    );
    // Five parts in a hundred. The residual is the pulse's own width, which
    // samples the potential over a finite stretch of a cycle rather than at one
    // instant, and it shrinks with the pulse rather than with the quadrature.
    close(measured, wanted, 5e-2);
}

#[test]
fn the_trace_it_builds_is_one_the_trace_format_admits() {
    let trace = operator::trace(
        "fixture-no-field",
        &gaussian_pulse(201),
        no_field(),
        &target(),
        &Grids {
            momenta: momenta(5, 0.6),
            delays: vec![-20.0, 20.0],
        },
    )
    .expect("the case is a trace");
    assert_eq!(trace.case, "fixture-no-field");
    assert_eq!(trace.electron_quantity, Electron::Momentum);
    assert_eq!(trace.electron.unit, "kg m/s");
    assert_eq!(trace.delay.unit, "as");
    assert!(matches!(trace.cells, Cells::Normalised { .. }));
    assert!(trace.to_bytes().is_ok(), "the writer refuses what it built");
}

#[test]
fn a_pulse_the_quadrature_cannot_be_applied_to_is_refused() {
    let refused = |pulse: Pulse| {
        operator::trace(
            "fixture",
            &pulse,
            no_field(),
            &target(),
            &Grids {
                momenta: momenta(3, 0.2),
                delays: vec![0.0],
            },
        )
        .expect_err("this pulse is refused")
    };

    // An even sample count. Composite Simpson pairs its intervals, and a rule
    // the caller did not choose would otherwise be applied silently.
    let mut even = gaussian_pulse(201);
    even.real.pop();
    even.imaginary.pop();
    assert!(matches!(refused(even), Refusal::PulseGrid { .. }));

    // The two parts of one sample, pasted to different lengths.
    let mut ragged = gaussian_pulse(201);
    ragged.imaginary.pop();
    assert!(matches!(refused(ragged), Refusal::PulseGrid { .. }));

    // A step that is not a step.
    let mut still = gaussian_pulse(201);
    still.step = 0.0;
    assert!(matches!(refused(still), Refusal::PulseGrid { .. }));

    // An odd count of three is the shortest grid Simpson applies to, so it is
    // not refused. A refusal written against a length below some larger number
    // would take it.
    let short = Pulse {
        first: -1.0,
        step: 1.0,
        real: vec![0.0, 1.0, 0.0],
        imaginary: vec![0.0, 0.0, 0.0],
    };
    assert!(operator::trace(
        "fixture",
        &short,
        no_field(),
        &target(),
        &Grids {
            momenta: momenta(3, 0.2),
            delays: vec![0.0],
        },
    )
    .is_ok());
}

#[test]
fn an_empty_grid_and_a_value_that_is_not_a_number_are_refused() {
    let run = |grids: Grids| {
        operator::trace(
            "fixture",
            &gaussian_pulse(201),
            no_field(),
            &target(),
            &grids,
        )
    };
    assert!(matches!(
        run(Grids {
            momenta: Vec::new(),
            delays: vec![0.0]
        }),
        Err(Refusal::EmptyGrid { .. })
    ));
    assert!(matches!(
        run(Grids {
            momenta: momenta(3, 0.2),
            delays: Vec::new()
        }),
        Err(Refusal::EmptyGrid { .. })
    ));
    assert!(matches!(
        run(Grids {
            momenta: vec![1.0, f64::NAN, 2.0],
            delays: vec![0.0]
        }),
        Err(Refusal::NotFinite { .. })
    ));
}

#[test]
fn a_momentum_window_reaching_past_the_dipole_table_is_refused_and_never_extrapolated() {
    // The flat table covers nought to a thousand electronvolts, which is about
    // thirty-seven hartree, so a momentum of nine carries forty and leaves it.
    // The alternative to this refusal is a trace with an invented tail in it.
    let found = operator::trace(
        "fixture",
        &gaussian_pulse(201),
        no_field(),
        &target(),
        &Grids {
            momenta: vec![1.0, 9.0],
            delays: vec![0.0],
        },
    );
    assert!(
        matches!(found, Err(Refusal::DipoleOutsideItsRange { .. })),
        "{found:?}"
    );
}

#[test]
fn the_pulse_grid_is_the_one_the_operator_integrates_over() {
    // The delay is not restricted to a whole number of steps. Half a step is
    // the case that would redden if the pulse were shifted onto a fixed grid by
    // interpolation instead.
    let pulse = gaussian_pulse(401);
    let quarter = pulse.step / 4.0;
    assert!(quarter > 0.0 && quarter < PI);
    let trace = operator::trace(
        "fixture",
        &pulse,
        no_field(),
        &target(),
        &Grids {
            momenta: momenta(5, 0.6),
            delays: vec![quarter],
        },
    )
    .expect("a delay that is a fraction of a step is a delay");
    assert_eq!(trace.values.columns, 1);
}

/// A table built here rather than shipped, so a case can put a slope or an edge
/// where it needs one. The shipped table is flat by definition and cannot.
fn table(unit_top: f64, amplitude: Vec<f64>, phase: Vec<f64>) -> Table {
    Table {
        target: "fixture".to_string(),
        source: "a fixture rather than a table of anything real".to_string(),
        unit: "eV".to_string(),
        energy: vec![0.0, unit_top],
        amplitude,
        normalisation: "one, by construction".to_string(),
        phase,
    }
}

#[test]
fn the_dipole_is_read_at_the_shifted_momentum_and_not_at_the_final_one() {
    // Exact rather than a tolerance, and it needs no non-flat table. The table
    // stops at thirty-five electronvolts. Every momentum on the grid below sits
    // under that on its own, and the largest of them shifted by the potential
    // does not, so the field-free case reads and the streaked case leaves the
    // table. A model reading the dipole at the final momentum would produce a
    // trace for both.
    let grid = momenta(5, 0.2);
    let target = Target {
        ionisation_potential: IONISATION_POTENTIAL,
        dipole: table(35.0, vec![1.0, 1.0], vec![0.0, 0.0]),
    };
    let grids = Grids {
        momenta: grid,
        delays: vec![0.0],
    };

    assert!(
        operator::trace("fixture", &gaussian_pulse(201), no_field(), &target, &grids).is_ok(),
        "every momentum on this grid is inside the table on its own"
    );

    let mut streaking = Streaking::new(StreakingField::new(0.3, 0.05, 4.0, 0.0).expect("a field"));
    streaking.intervals_per_cycle = 32;
    let found = operator::trace("fixture", &gaussian_pulse(201), streaking, &target, &grids);
    assert!(
        matches!(found, Err(Refusal::DipoleOutsideItsRange { .. })),
        "the potential shifts the largest momentum past the table's last sample: {found:?}"
    );
}

#[test]
fn the_dipole_amplitude_and_its_phase_both_reach_the_trace() {
    // Three targets that differ only in the dipole, on one set of parameters.
    // The shipped table cannot show this, because it is flat by definition, so
    // the slopes are built here.
    let grid = momenta(15, 0.4);
    let grids = Grids {
        momenta: grid,
        delays: vec![0.0],
    };
    let mut streaking = Streaking::new(StreakingField::new(0.08, 0.05, 4.0, 0.0).expect("a field"));
    streaking.intervals_per_cycle = 64;

    let run = |dipole: Table| {
        operator::trace(
            "fixture",
            &gaussian_pulse(401),
            streaking,
            &Target {
                ionisation_potential: IONISATION_POTENTIAL,
                dipole,
            },
            &grids,
        )
        .expect("each of these is a trace")
        .values
        .values
    };

    let flat = run(table(1000.0, vec![1.0, 1.0], vec![0.0, 0.0]));
    let sloped_amplitude = run(table(1000.0, vec![1.0, 20.0], vec![0.0, 0.0]));
    let sloped_phase = run(table(1000.0, vec![1.0, 1.0], vec![0.0, 400.0]));

    let differs = |left: &[f64], right: &[f64]| {
        left.iter()
            .zip(right)
            .any(|(one, other)| (one - other).abs() > 1e-6)
    };
    assert!(
        differs(&flat, &sloped_amplitude),
        "the amplitude column does not reach the trace"
    );
    assert!(
        differs(&flat, &sloped_phase),
        "the phase column does not reach the trace"
    );
}

#[test]
fn the_squared_potential_term_is_in_the_phase_and_reaches_the_trace() {
    // The term a convenient forward model drops. It is momentum-independent, so
    // it moves no first moment, and its time dependence is what a streaking
    // measurement reads. A case that dropped it silently would produce a trace
    // that looks right, which is why the choice is a variant with a name rather
    // than a default.
    let grids = Grids {
        momenta: momenta(15, 0.6),
        delays: vec![0.0],
    };
    let field = StreakingField::new(0.5, 0.05, 4.0, 0.0).expect("a field");
    let mut kept = Streaking::new(field);
    kept.intervals_per_cycle = 128;
    let mut dropped = kept;
    dropped.squared_term = messlatte_generator::field::SquaredTerm::Dropped;

    let run = |streaking: Streaking| {
        operator::trace(
            "fixture",
            &gaussian_pulse(401),
            streaking,
            &target(),
            &grids,
        )
        .expect("both are traces")
        .values
        .values
    };

    let with = run(kept);
    let without = run(dropped);
    assert!(
        with.iter()
            .zip(&without)
            .any(|(one, other)| (one - other).abs() > 1e-6),
        "the trace is the same with the squared term and without it"
    );
}
