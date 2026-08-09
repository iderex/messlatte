//! The limits the forward model has to reproduce (#46).
//!
//! Nothing checks the generator by construction, so it is checked against cases
//! whose answer is known without it. Every number this repository will report
//! rests on this model, and a wrong factor or a wrong sign inside it would
//! propagate into every result while every trace still looked like a trace.
//!
//! #46 names four limits and this file holds two of them. The other two are in
//! `crates/generator/tests/operator.rs`, where they landed with the operator
//! itself and with #123, and they are named here rather than moved, so that a
//! reader of either file can find all four:
//!
//! - with the streaking field off, the trace is the pulse's spectrum shifted by
//!   the ionisation potential, at every delay identically. That is
//!   `with_the_field_off_the_trace_is_the_pulse_spectrum_shifted_by_the_ionisation_potential`
//!   and the case beside it, at a tolerance of a part in a million.
//! - with a strong field and a short pulse, the trace's first moment in momentum
//!   follows minus the vector potential. That is
//!   `the_first_moment_follows_minus_the_vector_potential`, at five parts in a
//!   hundred.
//!
//! Here: the weak-field dependence of a sideband on the field strength, and the
//! oscillation of a sideband at twice the driving frequency for a train, with
//! the phase relation the interferometric methods read.
//!
//! Both cases below work on the amplitude rather than on the trace, through
//! [`messlatte_generator::operator::amplitudes`]. A trace is divided by its own
//! largest cell, so a statement about a trace is a statement about ratios
//! between cells, and every error that multiplies the whole of it survives one.
//! That is measured rather than supposed, and the measurement is in #46: the
//! fours and the twos of the quadrature's weights swapped, and no case in this
//! tree reddened.
//!
//! What these two limits cannot see, said here rather than left to be inferred.
//! Each is a statement about where amplitude sits on the momentum axis and how
//! it moves with a parameter, so neither catches a factor multiplying the whole
//! amplitude. The route that reaches that is the second quadrature in
//! `crates/generator/tests/quadrature.rs`, which is opt-in and did not run in a
//! default run.

use core::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

use messlatte_formats::dipole::Table;
use messlatte_generator::field::StreakingField;
use messlatte_generator::operator::{self, Grids, Pulse, Streaking, Target};

/// The carrier of the extreme-ultraviolet pulse, in atomic units.
const CARRIER: f64 = 1.5;

/// Its Gaussian width in time, in atomic units.
const WIDTH: f64 = 20.0;

/// The ionisation potential of the fixture target, in atomic units.
const IONISATION_POTENTIAL: f64 = 0.5;

/// The angular frequency of the driving field, in atomic units.
///
/// Ten times the frequency of the eight-hundred-nanometre light this field
/// actually uses, and that is deliberate rather than sloppy. A sideband is a
/// statement about a spectrum, and it is only a statement where the sideband is
/// resolved from the line it came from, which needs the driving frequency well
/// above the pulse's own spectral width of one over [`WIDTH`]. The alternative
/// is a pulse a hundred times longer, which costs the same limit a hundred times
/// the arithmetic and says nothing more. The relations checked below hold for
/// any driving frequency; the numbers here are a fixture vocabulary and no
/// result is quoted from them.
const DRIVING: f64 = 0.5;

/// How many cycles the driving envelope spans.
///
/// Two things set this and the second is the one that decided the number. The
/// envelope has to hold every sample of the pulse at every delay either case
/// scans, because a pulse reaching past the end of the support would see a field
/// that stops and neither limit below is about that. And the delay scan in the
/// second case moves the pulse through one driving period, over which the
/// envelope must not change much: what changes with the envelope rather than
/// with the interference appears in the scan at the driving frequency itself,
/// which is the term that case asserts is absent. Measured rather than chosen:
/// at twenty-four cycles that term stood at sixteen parts in a thousand of the
/// mean and at forty-eight it stands at four, which is where the tolerance below
/// comes from.
const DRIVING_CYCLES: f64 = 48.0;

/// The quadrature resolution the accumulated phase is integrated at.
///
/// Simpson on a smooth periodic integrand, so the error falls with the fourth
/// power of the step: sixteen intervals per cycle leaves a residual in the phase
/// far below the parts in a thousand these cases assert at, and the default of
/// five hundred and twelve buys nothing here except time.
const INTERVALS_PER_CYCLE: u32 = 16;

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
/// here.
///
/// Flat on purpose in both cases below. A dipole with structure over the
/// momentum window would put its own amplitude and its own phase into a
/// sideband, and the sideband is what is being measured; separating the two is
/// what the tabulated cases are for and is not this file's subject.
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

/// The momentum at which a photoelectron carries a given photon energy.
fn momentum_at(photon_energy: f64) -> f64 {
    (2.0 * (photon_energy - IONISATION_POTENTIAL)).sqrt()
}

/// A driving field of a given peak vector potential.
fn driving(amplitude: f64) -> Streaking {
    let mut streaking = Streaking::new(
        StreakingField::new(amplitude, DRIVING, DRIVING_CYCLES, 0.0).expect("a field"),
    );
    streaking.intervals_per_cycle = INTERVALS_PER_CYCLE;
    streaking
}

/// A pulse built from a set of spectral lines under one Gaussian envelope.
///
/// Each line is a photon energy and a phase, and the field is
/// `exp(-s^2 / (2 W^2)) * sum over lines of exp(i * phase) * exp(-i * energy *
/// s)`. The sign of the carrier is the one the fixture in
/// `crates/generator/tests/operator.rs` uses and the one the amplitude's own
/// `exp(+i * (p^2 / 2 + Ip) * s)` beats against, so a line at energy `w`
/// produces photoelectrons at `p^2 / 2 = w - Ip`.
///
/// One line is a transform-limited pulse. Two lines an equal distance either
/// side of a photoelectron energy are the smallest thing that is a train for the
/// purposes of the second case below: the sideband between them is reachable
/// from both, and the interference of those two routes is what oscillates.
fn pulse(samples: usize, lines: &[(f64, f64)]) -> Pulse {
    let span = 6.0 * WIDTH;
    let step = 2.0 * span / whole(samples - 1);
    let mut real = Vec::with_capacity(samples);
    let mut imaginary = Vec::with_capacity(samples);
    for index in 0..samples {
        let time = -span + step * whole(index);
        let envelope = (-time * time / (2.0 * WIDTH * WIDTH)).exp();
        let mut here = (0.0, 0.0);
        for (energy, phase) in lines {
            let angle = phase - energy * time;
            here.0 += envelope * angle.cos();
            here.1 += envelope * angle.sin();
        }
        real.push(here.0);
        imaginary.push(here.1);
    }
    Pulse {
        first: -span,
        step,
        real,
        imaginary,
    }
}

/// How many samples the pulse carries in both cases.
///
/// The integrand either case evaluates is the pulse against
/// `exp(i * (p^2 / 2 + Ip) * s)` at a momentum on a sideband, so what the
/// quadrature has to resolve is the beat between the two, at the driving
/// frequency, and not the carrier itself. One beat is about twelve and a half
/// atomic units against a span of two hundred and forty, so this is above sixty
/// samples per oscillation. It is a resolution for the sideband cells these
/// cases read and not for the whole momentum axis.
const SAMPLES: usize = 801;

#[test]
fn a_weak_field_puts_a_sideband_where_the_amplitude_grows_with_the_field() {
    // The second limit. Expand the amplitude in the driving field: the phase
    // accumulated in it is `p * D(t) + Q(t)`, where D is the integral of the
    // vector potential and is linear in the field's amplitude, and Q is the
    // integral of half its square and is quadratic. To first order the amplitude
    // therefore carries one power of the field, in a term proportional to
    // `p * D(s + tau)`, and D oscillates at the driving frequency. That puts
    // amplitude at a photoelectron energy one driving photon either side of the
    // line, and its size is proportional to the peak vector potential.
    //
    // So the trace at a sideband grows with the SQUARE of the field's amplitude,
    // and doubling the field multiplies the sideband by four. That is a
    // statement nothing in the operator computes, and it is a different one from
    // the fourth limit: this is where amplitude appears and how much, not where
    // the centre of mass moves.
    let below = momentum_at(CARRIER - DRIVING);
    let above = momentum_at(CARRIER + DRIVING);
    let grids = Grids {
        momenta: vec![below, above],
        delays: vec![0.0],
    };
    let transform_limited = pulse(SAMPLES, &[(CARRIER, 0.0)]);

    // Weak, and the three are a factor of two apart so that the prediction is a
    // ratio of four and not a fitted slope. The largest of them shifts the
    // photoelectron's momentum by under one per cent of itself.
    let weakest = 0.002;
    let yields = |amplitude: f64| {
        let found = operator::amplitudes(&transform_limited, driving(amplitude), &target(), &grids)
            .expect("the weak-field case is an amplitude");
        [found.intensity(0, 0), found.intensity(1, 0)]
    };

    let one = yields(weakest);
    let two = yields(2.0 * weakest);
    let four = yields(4.0 * weakest);

    for (index, side) in ["below", "above"].iter().enumerate() {
        assert!(
            one[index] > 0.0,
            "the sideband {side} the line carries nothing at all"
        );
        // Five parts in ten thousand. What is neglected is the next order in the
        // field, which reaches this sideband at two more powers of `p * A / w`,
        // and that ratio is under two parts in a hundred at the largest
        // amplitude here. The residual measured at this commit is one and three
        // tenths parts in ten thousand, on the ratio of sixteen at the upper
        // sideband, and the tolerance is four times it. It is a truncation of
        // the model rather than a floating-point residual, so it does not move
        // with the machine.
        close(two[index] / one[index], 4.0, 5e-4);
        close(four[index] / two[index], 4.0, 5e-4);
        close(four[index] / one[index], 16.0, 5e-4);
    }
}

/// The delays one driving period is scanned at.
///
/// Twelve resolves everything up to five times the driving frequency, which is
/// more than the third harmonic of it the case below asserts is absent. A count
/// that only just resolved the term being measured could not say whether what it
/// found was that term or something above it folded down.
const DELAYS: usize = 12;

/// The amplitude and the phase of one harmonic of a delay scan.
///
/// The scan covers exactly one driving period, so the harmonics of the scan are
/// harmonics of the driving frequency and this is a sum rather than a fit.
/// Returned as the pair (`cosine`, `sine`) so a caller can read a phase off it
/// in whichever convention its own prediction is written in.
fn harmonic(values: &[f64], order: usize) -> (f64, f64) {
    let mut cosine = 0.0;
    let mut sine = 0.0;
    for (index, value) in values.iter().enumerate() {
        let angle = 2.0 * PI * whole(order) * whole(index) / whole(values.len());
        cosine += value * angle.cos();
        sine += value * angle.sin();
    }
    let scale = 2.0 / whole(values.len());
    (cosine * scale, sine * scale)
}

#[test]
fn a_train_in_a_weak_driving_field_gives_a_sideband_oscillating_at_twice_that_frequency() {
    // The third limit, and the relation every interferometric method is derived
    // from. Two lines a driving photon either side of one photoelectron energy
    // are the smallest train that has a sideband: the electron reaches that
    // energy from the lower line by taking a driving photon and from the upper
    // line by giving one back, and the two routes interfere.
    //
    // Working it through to first order in the field gives the two routes an
    // amplitude of `exp(i * (delta + w * tau))` and `-exp(-i * w * tau)` apart
    // from a common factor, where delta is the phase between the two lines and w
    // is the driving frequency. Their squared sum is
    //
    //     ( 1 - cos( 2 * w * tau + delta ) ) / 2
    //
    // so the sideband oscillates at TWICE the driving frequency, is fully
    // modulated, and sits at a minimum at zero delay when the two lines are in
    // phase. The phase of that oscillation is what a RABBITT measurement reads,
    // and it moves with the phase between the lines one for one: that last part
    // is the whole content of the technique, and it is asserted here as a second
    // scan rather than argued.
    let sideband = momentum_at(CARRIER);
    let period = 2.0 * PI / DRIVING;
    let delays: Vec<f64> = (0..DELAYS)
        .map(|index| period * whole(index) / whole(DELAYS))
        .collect();
    let grids = Grids {
        momenta: vec![sideband],
        delays,
    };
    // Weak enough that a second driving photon is a correction rather than a
    // route: `p * A / w` is under three parts in a hundred here.
    let field = driving(0.01);

    let scan = |between_the_lines: f64| {
        let train = pulse(
            SAMPLES,
            &[
                (CARRIER - DRIVING, 0.0),
                (CARRIER + DRIVING, between_the_lines),
            ],
        );
        let found = operator::amplitudes(&train, field, &target(), &grids)
            .expect("the train case is an amplitude");
        (0..grids.delays.len())
            .map(|column| found.intensity(0, column))
            .collect::<Vec<f64>>()
    };

    for between_the_lines in [0.0, PI / 2.0, -PI / 3.0] {
        let yields = scan(between_the_lines);
        let mean = yields.iter().sum::<f64>() / whole(yields.len());
        assert!(mean > 0.0, "the sideband carries nothing at all");

        let (first_cosine, first_sine) = harmonic(&yields, 1);
        let (second_cosine, second_sine) = harmonic(&yields, 2);
        let (third_cosine, third_sine) = harmonic(&yields, 3);
        let first = first_cosine.hypot(first_sine) / mean;
        let second = second_cosine.hypot(second_sine) / mean;
        let third = third_cosine.hypot(third_sine) / mean;

        // The driving frequency itself and three times it are absent. A model
        // that put a one-photon oscillation on this sideband would be a model
        // where a single driving photon connects the sideband to itself, which
        // is the parity this measurement rests on. One part in a hundred of the
        // against a residual measured at this commit of four parts in a thousand
        // at the driving frequency and one at three times it. That residual is
        // the driving envelope changing across the scan, and it shrinks with
        // [`DRIVING_CYCLES`] rather than with anything about the quadrature.
        assert!(
            first < 1e-2 && third < 1e-2,
            "the scan carries {first:e} at the driving frequency and {third:e} at three times \
             it, against {second:e} at twice it"
        );

        // Fully modulated: the prediction is ( 1 - cos ) / 2, so the amplitude
        // of the oscillation equals its mean. Three parts in a thousand, against
        // a measured five parts in ten thousand.
        close(second, 1.0, 3e-3);

        // The phase relation. Writing the scan as `mean - amplitude * cos(2 w
        // tau + delta)`, the cosine coefficient is minus the amplitude times the
        // cosine of delta and the sine coefficient is plus the amplitude times
        // its sine.
        let measured = second_sine.atan2(-second_cosine);
        let difference = (measured - between_the_lines).rem_euclid(2.0 * PI);
        let difference = difference.min(2.0 * PI - difference);
        // Four parts in a thousand of a radian, against a measured residual of
        // seven parts in ten thousand at the largest of the three separations
        // below. This is the assertion the technique rests on: a shift in the
        // phase between the lines arrives in the oscillation one for one, which
        // is why a measured oscillation phase is read as a spectral phase.
        assert!(
            difference < 4e-3,
            "the oscillation sits at {measured} where the lines are {between_the_lines} apart"
        );
    }
}
