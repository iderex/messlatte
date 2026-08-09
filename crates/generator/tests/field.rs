//! The streaking field, its potential and the phase (#43).
//!
//! The worked example in `docs/format/streaking-field.md` is the subject of the
//! first case. The document is the authority for the conventions and this
//! reproduces its numbers, so a convention that moved in the code and not in the
//! document reddens here.
//!
//! The quadrature is not compared with a second implementation of itself.
//! Everything integrated below is checked against the integrand it came from,
//! by differencing the accumulated integral and requiring minus the potential,
//! which is a relation between two things this crate computes by different
//! routes.

use core::f64::consts::PI;

use messlatte_generator::field::{Refusal, SquaredTerm, StreakingField, VolkovPhase};

/// The worked example's field.
fn worked_example() -> StreakingField {
    StreakingField::new(1.0, 0.05, 4.0, 0.0).expect("the worked example is a field")
}

#[track_caller]
fn close(left: f64, right: f64, tolerance: f64) {
    let scale = left.abs().max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "{left:e} and {right:e} differ by more than {tolerance:e} of the larger"
    );
}

#[test]
fn the_worked_example_is_the_field_the_document_describes() {
    let field = worked_example();
    close(field.period(), 40.0 * PI, 4.0 * f64::EPSILON);
    close(field.half_duration(), 80.0 * PI, 4.0 * f64::EPSILON);
}

#[test]
fn the_three_instants_of_the_worked_example_are_the_documented_numbers() {
    // The numbers are the document's, written here as the document writes them
    // rather than read back out of the code. A convention that changed in the
    // code and not in the document fails this case.
    let field = worked_example();

    // The envelope's peak. Both terms of the field vanish, from both sides at
    // once, so the field is zero for two reasons and not one.
    close(field.envelope(0.0), 1.0, 4.0 * f64::EPSILON);
    close(field.potential(0.0), 1.0, 4.0 * f64::EPSILON);
    assert!(
        field.electric_field(0.0).abs() < 1e-15,
        "{}",
        field.electric_field(0.0)
    );

    // A quarter period later. The carrier's cosine is zero, so the potential is
    // zero and the whole field is the carrier term. The zero is a zero of exact
    // arithmetic and not of a double, so it is bounded absolutely.
    let quarter = 10.0 * PI;
    close(
        field.envelope(quarter),
        0.961_939_766_255_643_4,
        4.0 * f64::EPSILON,
    );
    assert!(
        field.potential(quarter).abs() < 1e-15,
        "{}",
        field.potential(quarter)
    );
    close(
        field.electric_field(quarter),
        0.048_096_988_312_782_17,
        4.0 * f64::EPSILON,
    );

    // Half a period later. The carrier's sine is zero, so the whole field is the
    // envelope's slope, and its sign is the one a wrong convention flips.
    let half = 20.0 * PI;
    close(
        field.envelope(half),
        0.853_553_390_593_273_7,
        4.0 * f64::EPSILON,
    );
    close(
        field.potential(half),
        -0.853_553_390_593_273_7,
        4.0 * f64::EPSILON,
    );
    close(
        field.electric_field(half),
        -0.004_419_417_382_415_92,
        16.0 * f64::EPSILON,
    );
}

#[test]
fn the_documented_values_are_the_closed_forms_the_document_gives() {
    // The decimals above are checkable by hand only because the document says
    // what they are in closed form. This is that claim, executed.
    let field = worked_example();
    close(
        field.envelope(20.0 * PI),
        (2.0 + 2.0_f64.sqrt()) / 4.0,
        4.0 * f64::EPSILON,
    );
    close(
        field.envelope(10.0 * PI),
        (2.0 + (2.0 + 2.0_f64.sqrt()).sqrt()) / 4.0,
        4.0 * f64::EPSILON,
    );
    close(
        field.electric_field(20.0 * PI),
        -2.0_f64.sqrt() / 320.0,
        16.0 * f64::EPSILON,
    );
}

#[test]
fn the_electric_field_is_minus_the_derivative_of_the_potential() {
    // The sign convention, checked against the potential rather than against a
    // second copy of the same formula. A central difference, so the error falls
    // as the square of the step and a step of this size leaves room to spare.
    let field = worked_example();
    let step = 1e-6;
    for index in -30..=30_i32 {
        let time = f64::from(index) * 8.0;
        let differenced =
            (field.potential(time + step) - field.potential(time - step)) / (2.0 * step);
        let stated = field.electric_field(time);
        assert!(
            (stated + differenced).abs() < 1e-9,
            "at t={time}: the field is {stated:e} and minus the derivative is {:e}",
            -differenced
        );
    }
}

#[test]
fn a_carrier_envelope_phase_of_a_quarter_turn_moves_the_carrier_and_not_the_envelope() {
    // The convention that the carrier is referred to the envelope's peak. With
    // the phase set to a quarter turn the potential at the peak is zero while
    // the envelope there is untouched, which is what "referred to the peak"
    // means and is not true of a phase applied as a time shift.
    let shifted = StreakingField::new(1.0, 0.05, 4.0, PI / 2.0).expect("a field");
    close(shifted.envelope(0.0), 1.0, 4.0 * f64::EPSILON);
    assert!(
        shifted.potential(0.0).abs() < 1e-15,
        "{}",
        shifted.potential(0.0)
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exactly zero is the claim. Outside the support the field is not small, it is the \
              value an early return wrote, and a tolerance here would pass a taper this envelope \
              does not have"
)]
fn the_field_is_exactly_zero_outside_its_support() {
    let field = worked_example();
    let half = field.half_duration();
    for time in [half * 1.000_001, -half * 1.000_001, half * 10.0, f64::MAX] {
        assert_eq!(field.envelope(time), 0.0, "at {time:e}");
        assert_eq!(field.potential(time), 0.0, "at {time:e}");
        assert_eq!(field.electric_field(time), 0.0, "at {time:e}");
    }
}

#[test]
fn a_parameter_that_describes_no_field_is_refused() {
    assert_eq!(
        StreakingField::new(f64::NAN, 0.05, 4.0, 0.0),
        Err(Refusal::AmplitudeNotFinite)
    );
    assert_eq!(
        StreakingField::new(1.0, 0.0, 4.0, 0.0),
        Err(Refusal::FrequencyNotPositive)
    );
    assert_eq!(
        StreakingField::new(1.0, -0.05, 4.0, 0.0),
        Err(Refusal::FrequencyNotPositive)
    );
    assert_eq!(
        StreakingField::new(1.0, 0.05, 0.0, 0.0),
        Err(Refusal::CyclesNotPositive)
    );
    assert_eq!(
        StreakingField::new(1.0, 0.05, 4.0, f64::INFINITY),
        Err(Refusal::PhaseNotFinite)
    );
    for refusal in [
        Refusal::AmplitudeNotFinite,
        Refusal::FrequencyNotPositive,
        Refusal::CyclesNotPositive,
        Refusal::PhaseNotFinite,
    ] {
        assert!(!refusal.to_string().is_empty());
    }
}

#[test]
fn the_accumulated_potential_differentiates_back_to_the_potential() {
    // The quadrature judged against its own integrand. The derivative of the
    // integral from t to the end is minus the value at t, so this is the
    // integral checked against the thing it integrated and not against a second
    // integrator.
    let phase = VolkovPhase::new(worked_example());
    let step = 1e-4;
    for index in -25..=25_i32 {
        let time = f64::from(index) * 9.0;
        let differenced =
            (phase.displacement(time + step) - phase.displacement(time - step)) / (2.0 * step);
        let expected = -worked_example().potential(time);
        assert!(
            (differenced - expected).abs() < 1e-7,
            "at t={time}: the difference is {differenced:e} and minus the potential is {expected:e}"
        );
    }
}

#[test]
fn the_accumulated_squared_potential_differentiates_back_to_it() {
    let phase = VolkovPhase::new(worked_example());
    let step = 1e-4;
    for index in -25..=25_i32 {
        let time = f64::from(index) * 9.0;
        let differenced = (phase.squared_integral(time + step)
            - phase.squared_integral(time - step))
            / (2.0 * step);
        let potential = worked_example().potential(time);
        let expected = -potential * potential / 2.0;
        assert!(
            (differenced - expected).abs() < 1e-7,
            "at t={time}: the difference is {differenced:e} and the integrand is {expected:e}"
        );
    }
}

#[test]
fn the_phase_keeps_the_full_momentum_dependence() {
    // The near-miss this repository is built around. A phase expanded about a
    // central momentum is affine in the momentum with a coefficient fixed at
    // that centre, so the difference between successive momenta would be the
    // same wherever they sat. Here the coefficient is the displacement at the
    // birth time and nothing else, so the differences are equal for a reason
    // that survives any momentum, and a phase that had dropped the momentum
    // term would make every difference zero.
    let field = worked_example();
    let phase = VolkovPhase::new(field);
    let birth = -30.0;
    let displacement = phase.displacement(birth);
    assert!(
        displacement.abs() > 1e-3,
        "the example field has to displace something for this case to bite: {displacement:e}"
    );
    for momentum in [0.5, 1.0, 2.0, 5.0] {
        close(
            phase.at(momentum, birth) - phase.at(0.0, birth),
            momentum * displacement,
            1e-12,
        );
    }
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exactly zero is the claim. With the squared term dropped and no momentum there is \
              nothing left to accumulate, so a phase that is merely small would mean the term \
              was still being integrated"
)]
fn the_squared_term_is_kept_unless_a_caller_names_the_other_choice() {
    let field = worked_example();
    let birth = -30.0;
    let kept = VolkovPhase::new(field);
    let dropped = VolkovPhase::new(field).without_squared_term();
    let squared = kept.squared_integral(birth);
    assert!(
        squared.abs() > 1e-3,
        "the term has to be worth something for this case to bite: {squared:e}"
    );
    close(kept.at(0.0, birth), squared, 1e-12);
    assert_eq!(dropped.at(0.0, birth), 0.0);
    close(kept.at(1.0, birth) - dropped.at(1.0, birth), squared, 1e-12);
    assert_ne!(SquaredTerm::Kept, SquaredTerm::Dropped);
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exactly zero is the claim. The upper limit is the end of the support rather than \
              infinity, which is exact because the potential is identically zero beyond it, and \
              a tolerance would hide a quadrature that ran past the end"
)]
fn the_phase_is_zero_once_the_pulse_is_over() {
    let phase = VolkovPhase::new(worked_example());
    let end = worked_example().half_duration();
    for birth in [end, end * 1.000_001, end * 10.0] {
        assert_eq!(phase.at(3.0, birth), 0.0, "at {birth:e}");
    }
}

#[test]
fn a_coarser_quadrature_agrees_with_a_finer_one() {
    // What the default resolution buys, measured rather than asserted. Simpson
    // on a smooth integrand, so halving the count should not move the answer at
    // this bound; a count too coarse for the carrier would.
    let field = worked_example();
    let birth = -100.0;
    let fine = VolkovPhase::new(field).with_intervals_per_cycle(4096);
    let default = VolkovPhase::new(field);
    let coarse = VolkovPhase::new(field).with_intervals_per_cycle(64);

    let reference = fine.at(2.0, birth);
    let from_default = ((default.at(2.0, birth) - reference) / reference).abs();
    let from_coarse = ((coarse.at(2.0, birth) - reference) / reference).abs();
    println!("default {from_default:e} coarse {from_coarse:e} against {reference:e}");

    // The bounds are above what was measured rather than at it, so a run on
    // another target does not redden for a last-place difference. The numbers
    // the run prints are what the pull request quotes.
    assert!(from_default < 1e-8, "{from_default:e}");
    assert!(from_coarse < 1e-3, "{from_coarse:e}");

    // The knob does something, which is the near-miss. A resolution argument
    // that was read and discarded would leave these two equal, and both bounds
    // above would still pass.
    assert!(
        from_coarse > from_default * 100.0,
        "coarse {from_coarse:e} is not meaningfully worse than default {from_default:e}, so the \
         resolution argument may not be reaching the quadrature"
    );
}
