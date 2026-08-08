//! Constructed pairs for the quotient (#8).
//!
//! Every case builds a field, applies a known transformation to it, and
//! requires the quotient to give the distance back as zero and to name the
//! transformation it undid. The direction that matters as much is the last two
//! cases: a freedom the case did not admit is not removed, and two genuinely
//! different fields do not align, which is what catches an alignment so
//! generous that everything matches.

use messlatte_scoring::{quotient, Amplitude, Freedoms};

/// The whole-sample grid every case here works on. Wide enough that a field of
/// width `WIDTH` centred in it is below the double-precision floor at both
/// ends, so a shift with zero fill loses nothing that the comparison could see.
const SAMPLES: usize = 96;
const START: f64 = -24.0;
const STEP: f64 = 0.5;
const WIDTH: f64 = 2.0;

/// What counts as zero here, and why that number.
///
/// The residual is a sum of a few hundred double-precision products, so its
/// floor is a small multiple of the machine epsilon times the squared field
/// norm, and the relative distance derived from it lands near 1e-16. A
/// thousand times that leaves room for the accumulation order to change
/// without moving a verdict, and it is far below any error this board reports.
const ZERO: f64 = 1e-12;

/// A chirped Gaussian with a satellite after it.
///
/// The chirp is what makes the field complex rather than real. The satellite is
/// what makes it asymmetric in time, and it is load bearing rather than
/// decorative: a field that is even about its centre is carried onto a shift of
/// itself by a reversal, so on one of those the case below asking for a
/// reversal is answered with a shift and the case asking for a reversal to be
/// refused is answered with a distance of zero. Both did, before the satellite
/// was here.
fn field(centre: f64, width: f64, chirp: f64) -> Vec<Amplitude> {
    let mut out = Vec::with_capacity(SAMPLES);
    let mut time = START;
    for _ in 0..SAMPLES {
        let offset = time - centre;
        let satellite = offset - 2.5 * width;
        let envelope = (-(offset * offset) / (2.0 * width * width)).exp()
            + 0.35 * (-2.0 * satellite * satellite / (width * width)).exp();
        let phase = chirp * offset * offset;
        out.push(Amplitude::new(
            envelope * phase.cos(),
            envelope * phase.sin(),
        ));
        time += STEP;
    }
    out
}

fn phased(field: &[Amplitude], phase: f64) -> Vec<Amplitude> {
    let turn = Amplitude::new(phase.cos(), phase.sin());
    field.iter().map(|value| value.times(turn)).collect()
}

fn resized(field: &[Amplitude], factor: f64) -> Vec<Amplitude> {
    field.iter().map(|value| value.scaled(factor)).collect()
}

/// The field moved to later times, with zeros where it moved in from, which is
/// what the quotient's own displacement does and what it has to undo.
fn later(field: &[Amplitude], samples: usize) -> Vec<Amplitude> {
    let mut out = vec![Amplitude::default(); field.len()];
    out[samples..].copy_from_slice(&field[..field.len() - samples]);
    out
}

fn backwards(field: &[Amplitude]) -> Vec<Amplitude> {
    field.iter().copied().rev().collect()
}

/// The difference between two angles, in the half turn either side of zero, so
/// that a phase recovered as -3.14 and one recovered as 3.14 agree.
fn angle_gap(left: f64, right: f64) -> f64 {
    let raw = (left - right).rem_euclid(std::f64::consts::TAU);
    (raw - std::f64::consts::TAU).abs().min(raw)
}

fn admitting_everything() -> Freedoms {
    Freedoms {
        amplitude_scale: true,
        time_reversal: true,
    }
}

#[test]
fn a_constant_phase_offset_is_removed_and_reported() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = phased(&reference, 0.9);

    let aligned = quotient(&reference, &candidate, admitting_everything()).unwrap();

    assert!(aligned.distance < ZERO, "distance {}", aligned.distance);
    assert_eq!(aligned.transformation.shift, 0);
    assert!(!aligned.transformation.reversed);
    assert!(angle_gap(aligned.transformation.scale.arg(), -0.9) < 1e-9);
}

#[test]
fn a_whole_sample_shift_is_removed_and_reported() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = later(&reference, 7);

    let aligned = quotient(&reference, &candidate, admitting_everything()).unwrap();

    assert!(aligned.distance < ZERO, "distance {}", aligned.distance);
    assert_eq!(aligned.transformation.shift, -7);
    assert!(!aligned.transformation.reversed);
}

#[test]
fn the_sign_of_the_time_axis_is_removed_where_the_case_admits_it() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = backwards(&reference);

    let aligned = quotient(&reference, &candidate, admitting_everything()).unwrap();

    assert!(aligned.distance < ZERO, "distance {}", aligned.distance);
    assert!(aligned.transformation.reversed);
}

#[test]
fn the_three_freedoms_are_removed_together() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = later(&phased(&backwards(&reference), -2.4), 5);

    let aligned = quotient(&reference, &candidate, admitting_everything()).unwrap();

    assert!(aligned.distance < ZERO, "distance {}", aligned.distance);
    assert!(aligned.transformation.reversed);
    // The reversal is applied first, so the five samples the candidate was
    // moved later on its own axis are five samples earlier on the reversed one,
    // and undoing them moves it later.
    assert_eq!(aligned.transformation.shift, 5);
    assert!(angle_gap(aligned.transformation.scale.arg(), 2.4) < 1e-9);
}

#[test]
fn the_aligned_candidate_is_the_reference_to_the_same_tolerance() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = resized(&later(&phased(&reference, 1.7), 3), 4.0);

    let aligned = quotient(&reference, &candidate, admitting_everything()).unwrap();

    let worst = aligned
        .candidate
        .iter()
        .zip(reference.iter())
        .map(|(left, right)| Amplitude::new(left.re - right.re, left.im - right.im).abs())
        .fold(0.0_f64, f64::max);
    assert!(worst < ZERO, "worst sample difference {worst}");
}

#[test]
fn an_amplitude_scale_is_removed_only_where_the_case_admits_it() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = resized(&reference, 3.0);

    let removed = quotient(&reference, &candidate, admitting_everything()).unwrap();
    assert!(removed.distance < ZERO, "distance {}", removed.distance);
    assert!((removed.transformation.scale.abs() - 1.0 / 3.0).abs() < 1e-9);

    let kept = quotient(
        &reference,
        &candidate,
        Freedoms {
            amplitude_scale: false,
            time_reversal: true,
        },
    )
    .unwrap();
    // Three times the reference, with the size held, is two references away
    // from it. That number matters more than its size: the shift cannot be used
    // to push the candidate off the grid and score the one an empty grid would.
    assert!(
        (kept.distance - 2.0).abs() < 1e-9,
        "distance {}",
        kept.distance
    );
    assert!((kept.transformation.scale.abs() - 1.0).abs() < 1e-9);
}

#[test]
fn a_reversal_the_case_does_not_admit_is_not_removed() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = backwards(&reference);

    let aligned = quotient(
        &reference,
        &candidate,
        Freedoms {
            amplitude_scale: true,
            time_reversal: false,
        },
    )
    .unwrap();

    assert!(aligned.distance > 0.1, "distance {}", aligned.distance);
    assert!(!aligned.transformation.reversed);
}

#[test]
fn two_genuinely_different_fields_do_not_align() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = field(0.0, WIDTH * 2.5, -0.3);

    let aligned = quotient(&reference, &candidate, admitting_everything()).unwrap();

    assert!(aligned.distance > 0.1, "distance {}", aligned.distance);
}

#[test]
fn one_input_gives_one_answer_bit_for_bit() {
    let reference = field(0.0, WIDTH, 0.05);
    let candidate = later(&phased(&reference, 0.4), 2);

    let first = quotient(&reference, &candidate, admitting_everything()).unwrap();
    let second = quotient(&reference, &candidate, admitting_everything()).unwrap();

    assert_eq!(first.distance.to_bits(), second.distance.to_bits());
    assert_eq!(first.transformation.shift, second.transformation.shift);
    assert_eq!(
        first.transformation.scale.re.to_bits(),
        second.transformation.scale.re.to_bits()
    );
}

#[test]
fn what_cannot_be_compared_is_refused_rather_than_repaired() {
    let reference = field(0.0, WIDTH, 0.05);
    let freedoms = admitting_everything();

    let short = quotient(&reference, &reference[..SAMPLES - 1], freedoms).unwrap_err();
    assert!(short.contains("not on one grid"), "{short}");

    let empty = quotient(&[], &[], freedoms).unwrap_err();
    assert!(empty.contains("empty grid"), "{empty}");

    let mut broken = reference.clone();
    broken[3] = Amplitude::new(f64::NAN, 0.0);
    let infinite = quotient(&reference, &broken, freedoms).unwrap_err();
    assert!(
        infinite.contains("non-finite value at sample 3"),
        "{infinite}"
    );

    let zeros = vec![Amplitude::default(); SAMPLES];
    let flat = quotient(&zeros, &reference, freedoms).unwrap_err();
    assert!(flat.contains("zero everywhere"), "{flat}");

    let nothing = quotient(&reference, &zeros, freedoms).unwrap_err();
    assert!(nothing.contains("no alignment"), "{nothing}");
}
