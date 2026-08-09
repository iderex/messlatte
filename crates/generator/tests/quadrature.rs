//! The two evaluations of the operator's integral, compared (#46).
//!
//! This case is the one route in this repository that could notice a wrong
//! factor in the forward model. Every limit in
//! `crates/generator/tests/limits.rs` and in
//! `crates/generator/tests/operator.rs` is a statement about the shape of what
//! comes out, so none of them can see an error that multiplies the whole of it.
//! Two implementations of the same expression, built out of different parts, can.
//!
//! Two implementations agreeing is not a proof. A disagreement is a finding, and
//! the alternative to having this is that nothing here could ever notice.
//!
//! It is compared on the AMPLITUDE and not on the trace, which is the point of
//! [`messlatte_generator::operator::amplitudes`] existing. A trace is divided by
//! its own largest cell, so a comparison made after that division would report
//! agreement for the reason that both sides had been divided by their own
//! largest cell. That was measured rather than supposed, and the measurement is
//! in #46.
//!
//! ## Why this is opt-in, and how it is asked for
//!
//! The slower route integrates the accumulated phase again for every cell rather
//! than once per delay and sample, and it needs a grid many times finer to reach
//! a tolerance the faster route reaches on a coarse one, because the trapezoid's
//! error falls with the square of the step where Simpson's falls with the fourth
//! power. Both are deliberate. Together they put this well outside what the
//! default suite can afford, so it belongs in the suite named for what it costs,
//! which is the one asked for with `MESSLATTE_SUITE_MINUTES`.
//!
//! What that variable means is decided in `crates/suites`, and this file reads
//! it rather than calling that crate. `layout.toml` gives the generator role
//! edges to `units` and `formats` and to nothing else, and its rule covers
//! development dependencies as well as ordinary ones, so no case in this crate
//! can call the suite split. The name of the variable is therefore written here
//! as well as there, which is a second place one fact lives and nothing in this
//! tree refuses a drift between them. That is a gap rather than a decision, and
//! it is #46's to report rather than to close by adding an edge to a declaration
//! it does not own.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use messlatte_formats::dipole::Table;
use messlatte_generator::field::StreakingField;
use messlatte_generator::operator::{self, Grids, Pulse, Streaking, Target};
use messlatte_generator::quadrature;

/// The variable that asks for the suite this case is in.
const ASKED_WITH: &str = "MESSLATTE_SUITE_MINUTES";

/// Whether this run asked for the suite.
///
/// The three readings are the ones `crates/suites` defines: unset is nobody
/// asked, `1` is run it, `0` is do not, and anything else is refused rather than
/// read as off. Somebody who wrote `true` asked for this suite, and a run that
/// treated that as a decline would report the case as skipped to the person
/// watching for it to pass.
fn asked_for() -> bool {
    match env::var(ASKED_WITH) {
        Err(env::VarError::NotPresent) => false,
        Ok(value) if value == "1" => true,
        Ok(value) if value == "0" => false,
        Ok(value) => panic!(
            "{ASKED_WITH} holds {value:?}, which is neither 1 nor 0, so what was asked for cannot \
             be read"
        ),
        Err(env::VarError::NotUnicode(_)) => panic!(
            "{ASKED_WITH} holds bytes that are not text, so what was asked for cannot be read"
        ),
    }
}

/// The carrier of the fixture pulse, in atomic units.
const CARRIER: f64 = 1.5;

/// Its Gaussian width in time, in atomic units.
const WIDTH: f64 = 5.0;

/// The ionisation potential of the fixture target, in atomic units.
const IONISATION_POTENTIAL: f64 = 0.5;

/// The samples the faster route integrates over.
const SAMPLES: usize = 401;

/// How many times finer the slower route's grid is.
///
/// The trapezoid's error falls with the square of the step and Simpson's with
/// the fourth power, so the slower rule only reaches the faster one's accuracy
/// on a grid several times finer. Thirty-two is what puts both estimates of this
/// fixture's integral below the tolerance asserted at the bottom of this file,
/// and it is where most of the cost is.
const REFINEMENT: usize = 32;

/// One count as a double. The lint set refuses the cast because a `usize` is
/// wider than a double's mantissa, and every count in this file is a grid size.
fn whole(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("a grid this file builds fits in a u32"))
}

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
/// peak, at whatever resolution the caller asks for.
///
/// Evaluated again rather than interpolated. The slower route needs a finer grid
/// than the faster one, and a pulse interpolated onto that grid would be a
/// different pulse, so the two routes would be compared on two integrands.
fn gaussian_pulse(samples: usize) -> Pulse {
    let span = 6.0 * WIDTH;
    let step = 2.0 * span / whole(samples - 1);
    let mut real = Vec::with_capacity(samples);
    let mut imaginary = Vec::with_capacity(samples);
    for index in 0..samples {
        let time = -span + step * whole(index);
        let envelope = (-time * time / (2.0 * WIDTH * WIDTH)).exp();
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

/// The fixture field: strong enough that the phase is doing work, and short
/// enough that its support holds the pulse at both delays.
fn field() -> StreakingField {
    StreakingField::new(0.1, 0.5, 8.0, 0.0).expect("a field")
}

/// Momenta about the one the pulse is centred on.
fn momenta(count: usize, spread: f64) -> Vec<f64> {
    let centre = (2.0 * (CARRIER - IONISATION_POTENTIAL)).sqrt();
    (0..count)
        .map(|index| centre + spread * (whole(index) / whole(count - 1) - 0.5))
        .collect()
}

#[test]
fn the_two_quadratures_agree_on_the_amplitude_before_any_normalisation() {
    if !asked_for() {
        println!(
            "NOT RUN: {ASKED_WITH} is unset, so the second quadrature did not run and this \
             run cannot be read as one that compared the two implementations of the operator."
        );
        return;
    }

    let grids = Grids {
        momenta: momenta(3, 0.3),
        delays: vec![0.0, field().period() / 4.0],
    };

    // Each route is given the resolution its own rule needs to put its own
    // truncation below the tolerance, which is what makes the comparison a
    // statement about the two expressions rather than about two step sizes. The
    // field is the same field in both.
    let mut fast = Streaking::new(field());
    fast.intervals_per_cycle = 512;
    let mut slow = Streaking::new(field());
    slow.intervals_per_cycle = 2048;

    let coarse = gaussian_pulse(SAMPLES);
    let fine = gaussian_pulse((SAMPLES - 1) * REFINEMENT + 1);

    let left = operator::amplitudes(&coarse, fast, &target(), &grids)
        .expect("the fixture is an amplitude on the faster route");
    let right = quadrature::amplitudes(&fine, slow, &target(), &grids)
        .expect("the fixture is an amplitude on the slower route");

    let worst = quadrature::largest_difference(&left, &right);
    println!("the two routes differ by {worst:e} of the largest amplitude in the pair");

    // Five parts in ten million of the largest amplitude in the pair. What is
    // left at this resolution is the two rules' own truncation, and the residual
    // measured when this landed was six and eight tenths parts in a hundred
    // million, quoted with the run that produced it in #46. The tolerance is
    // seven times that rather than a number chosen to make the comparison pass,
    // and the measurement is repeatable by asking for this suite.
    assert!(
        worst < 5e-7,
        "the two evaluations of the operator's integral differ by {worst:e} of the largest \
         amplitude in the pair, which is a disagreement about the model rather than about a \
         step size"
    );

    // The pair carries something to disagree about. Without this, a fixture that
    // produced nothing anywhere would agree perfectly and this case would pass
    // on an empty comparison.
    let largest = (0..left.momenta)
        .flat_map(|row| (0..left.delays).map(move |column| (row, column)))
        .map(|(row, column)| left.intensity(row, column))
        .fold(0.0, f64::max);
    assert!(
        largest > 0.0,
        "every cell of the fixture is zero, so the comparison above compared nothing"
    );

    // And the delays are not two names for one column. The field is at a maximum
    // at the first and at zero at the second, so a model that ignored the delay
    // would pass the comparison above and fail here.
    let moved = (0..left.momenta)
        .any(|row| (left.intensity(row, 0) - left.intensity(row, 1)).abs() > largest * 1e-6);
    assert!(
        moved,
        "the two delays carry the same column, so the fixture does not exercise the phase"
    );
}

#[test]
fn the_slower_route_refuses_what_the_faster_one_refuses() {
    // Cheap enough for the default suite, and it is the half of the pair that
    // has nothing to do with wall clock: the two routes share one validation, so
    // a caller cannot be told by one of them that an input is fine and by the
    // other that it is not. The near-miss is an even sample count, which the
    // trapezoid would happily integrate and composite Simpson cannot.
    let mut even = gaussian_pulse(101);
    even.real.pop();
    even.imaginary.pop();
    let grids = Grids {
        momenta: momenta(2, 0.2),
        delays: vec![0.0],
    };
    let streaking = Streaking::new(field());

    let refused_by_the_faster = operator::amplitudes(&even, streaking, &target(), &grids);
    let refused_by_the_slower = quadrature::amplitudes(&even, streaking, &target(), &grids);
    assert_eq!(
        format!("{refused_by_the_faster:?}"),
        format!("{refused_by_the_slower:?}"),
        "the two routes disagree about which inputs they are comparable on"
    );
    assert!(refused_by_the_faster.is_err());

    // And the one-change neighbour: the same pulse with the sample put back is
    // taken by both. A refusal that took every pulse would pass the assertion
    // above.
    let odd = gaussian_pulse(101);
    assert!(operator::amplitudes(&odd, streaking, &target(), &grids).is_ok());
    assert!(quadrature::amplitudes(&odd, streaking, &target(), &grids).is_ok());
}

#[test]
fn the_period_of_the_fixture_field_holds_the_pulse() {
    // The fixture is only a fixture if the pulse sits inside the field's
    // support at both delays. A pulse reaching past the end would see a field
    // that stops, and the comparison above would then be a comparison of two
    // routes through a discontinuity rather than through the operator.
    let field = field();
    let reach = 6.0 * WIDTH + field.period() / 4.0;
    assert!(
        reach < field.half_duration(),
        "the pulse reaches {reach} and the field's support ends at {}",
        field.half_duration()
    );
}
