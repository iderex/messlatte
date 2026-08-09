//! A second, slower evaluation of the same integral, written to disagree (#46).
//!
//! Nothing checks the forward model by construction. Every number this
//! repository will report rests on the expression in
//! `docs/format/streaking-operator.md`, and the limits in
//! `crates/generator/tests/limits.rs` check the shape of what comes out of it
//! against answers that are known without it. This is the other half: one more
//! implementation of the same expression, deliberately built out of different
//! parts, so that a disagreement is a finding.
//!
//! Two implementations agreeing is not a proof and this module does not claim to
//! be one. What it buys is that a wrong factor in one of them has to be matched
//! by the same wrong factor in the other before the pair goes quiet, and the
//! alternative is that nothing here could ever notice.
//!
//! Where the two differ, and each difference is chosen rather than inherited.
//!
//! The rule over the pulse is the trapezoid rather than composite Simpson, so
//! the two carry different truncation errors and converge to the same integral
//! from different sides. That is also why the comparison belongs in an opt-in
//! suite: the trapezoid's error falls with the square of the step where
//! Simpson's falls with the fourth power, so agreement at a useful tolerance is
//! bought with a grid several times finer, and the case that buys it takes
//! minutes rather than the seconds a default case is allowed.
//!
//! The accumulated phase is integrated here, by the trapezoid, in one pass over
//! `momentum * A + A^2 / 2`, rather than as the two separate integrals
//! [`crate::field::VolkovPhase`] accumulates by Simpson and combines afterwards.
//! So the momentum enters before the quadrature here and after it there, and an
//! error in how the two terms are combined cannot be common to both.
//!
//! Nothing is precomputed. The operator evaluates the phase once per delay and
//! sample and reuses it across the momentum axis, which is correct because its
//! phase is linear in the momentum; here the phase is integrated again for every
//! cell. That is the whole of the cost, and it is the point: a shared table is a
//! shared assumption.
//!
//! What is NOT independent, and a reader should not read this as covering it.
//! Both implementations call the same field, the same dipole table and the same
//! refusals, so a wrong vector potential, a wrong table reading or a wrong
//! convention in either is invisible to this comparison. Those are checked
//! against closed forms in `crates/generator/tests/field.rs` and against the
//! limits, not here.

use messlatte_formats::dipole::Lookup;
use messlatte_units::Energy;

use crate::field::{SquaredTerm, StreakingField};
use crate::operator::{check, Amplitudes, Grids, Pulse, Refusal, Streaking, Target};

/// One, as a double, built from an integer.
///
/// The reasoning is the one at the same helper in [`crate::field`]: the
/// invariants rule refuses a floating-point literal in this crate's source.
fn one() -> f64 {
    f64::from(1_u8)
}

/// Two, on the same reasoning.
fn two() -> f64 {
    f64::from(2_u8)
}

/// Zero, on the same reasoning.
fn zero() -> f64 {
    f64::from(0_u8)
}

/// The largest number of quadrature intervals one phase integral is cut into.
///
/// A ceiling rather than a guess. The interval count is derived from the span
/// left to integrate and the resolution the caller asked for, and both come from
/// a caller; without a bound a field with a very short period turns one cell into
/// an unbounded loop. Where the bound bites the phase is integrated more coarsely
/// than the caller asked for, which the comparison sees as a disagreement rather
/// than as silence.
const MOST_INTERVALS: u32 = 1 << 22;

/// The same amplitudes [`crate::operator::amplitudes`] returns, by the other
/// route.
///
/// The grids, the pulse and the target mean what they mean there, and the
/// refusals are that module's, shared rather than restated so that the two
/// implementations cannot disagree about which inputs they are comparable on.
///
/// `streaking.intervals_per_cycle` is read as trapezoid intervals per optical
/// cycle here and as Simpson intervals there. The same number therefore buys
/// less accuracy on this route, which is the difference the comparison is made
/// of and not an oversight.
///
/// # Errors
///
/// The set [`crate::operator::trace`] refuses, for the same reasons.
pub fn amplitudes(
    pulse: &Pulse,
    streaking: Streaking,
    target: &Target,
    grids: &Grids,
) -> Result<Amplitudes, Refusal> {
    check(pulse, target, grids)?;

    let field = streaking.field;
    let samples = pulse.samples();
    let cells = grids.momenta.len() * grids.delays.len();
    let mut real_part = vec![zero(); cells];
    let mut imaginary_part = vec![zero(); cells];

    for (row, momentum) in grids.momenta.iter().enumerate() {
        let drift = momentum * momentum / two() + target.ionisation_potential;
        for (column, delay) in grids.delays.iter().enumerate() {
            let mut real = zero();
            let mut imaginary = zero();
            for index in 0..samples {
                let sample_time = pulse.time(index);
                let time = sample_time + delay;
                let shifted = momentum + field.potential(time);
                let energy = shifted * shifted / two();
                let dipole = target
                    .dipole
                    .at(Energy::from_hartree(energy))
                    .map_err(|lookup| outside(*momentum, &lookup))?;

                let accumulated = phase(
                    field,
                    streaking.squared_term,
                    *momentum,
                    time,
                    streaking.intervals_per_cycle,
                );
                let angle = dipole.phase - accumulated + drift * sample_time;
                // The trapezoid: the two ends carry half a step each and every
                // interior sample carries one.
                let weight = if index == 0 || index + 1 == samples {
                    one() / two()
                } else {
                    one()
                };
                let scale = weight * dipole.amplitude;
                real += scale
                    * (pulse.real[index] * angle.cos() - pulse.imaginary[index] * angle.sin());
                imaginary += scale
                    * (pulse.real[index] * angle.sin() + pulse.imaginary[index] * angle.cos());
            }
            let at = row * grids.delays.len() + column;
            real_part[at] = real * pulse.step;
            imaginary_part[at] = imaginary * pulse.step;
        }
    }

    Ok(Amplitudes {
        momenta: grids.momenta.len(),
        delays: grids.delays.len(),
        real: real_part,
        imaginary: imaginary_part,
    })
}

fn outside(momentum: f64, lookup: &Lookup) -> Refusal {
    Refusal::DipoleOutsideItsRange {
        momentum,
        detail: lookup.to_string(),
    }
}

/// The phase accumulated from `birth` to the end of the field's support.
///
/// One integrand rather than two, so the momentum multiplies the potential
/// inside the quadrature rather than the finished integral outside it. The two
/// are the same number in exact arithmetic and are reached by different
/// operations, which is the whole reason this exists beside the other route.
///
/// Zero once the field is over, and that is exact rather than a tolerance: the
/// potential is identically zero there.
fn phase(
    field: StreakingField,
    squared_term: SquaredTerm,
    momentum: f64,
    birth: f64,
    intervals_per_cycle: u32,
) -> f64 {
    let end = field.half_duration();
    if !birth.is_finite() || birth >= end {
        return zero();
    }
    let start = birth.max(-end);
    let span = end - start;

    let wanted = (span / field.period()) * f64::from(intervals_per_cycle.max(1));
    let count = if wanted.is_finite() && wanted > one() {
        let ceiling = wanted.ceil().min(f64::from(MOST_INTERVALS));
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "bounded above by MOST_INTERVALS and below by one on the line above, and \
                      rounded before the conversion, so neither the high bits nor the sign can \
                      be lost"
        )]
        let ceiling = ceiling as u32;
        ceiling
    } else {
        1
    };

    let step = span / f64::from(count);
    let integrand = |time: f64| {
        let potential = field.potential(time);
        match squared_term {
            SquaredTerm::Kept => momentum * potential + potential * potential / two(),
            SquaredTerm::Dropped => momentum * potential,
        }
    };

    let mut total = (integrand(start) + integrand(end)) / two();
    for index in 1..count {
        total += integrand(start + step * f64::from(index));
    }
    total * step
}

/// The largest relative difference between two sets of amplitudes, cell by cell.
///
/// Relative to the largest modulus anywhere in the pair rather than to the cell,
/// because a trace has cells many orders below its peak and a relative
/// comparison at one of those measures the arithmetic's own floor rather than
/// the model. A caller wanting the other question asks it of one cell.
///
/// # Panics
///
/// The two sets are not on the same grid, which is a caller comparing two
/// different runs rather than two routes through one.
#[must_use]
pub fn largest_difference(left: &Amplitudes, right: &Amplitudes) -> f64 {
    assert!(
        left.momenta == right.momenta && left.delays == right.delays,
        "a grid of {} by {} against one of {} by {}",
        left.momenta,
        left.delays,
        right.momenta,
        right.delays
    );

    let modulus =
        |amplitudes: &Amplitudes, at: usize| amplitudes.real[at].hypot(amplitudes.imaginary[at]);
    let mut largest = zero();
    for at in 0..left.real.len() {
        largest = largest.max(modulus(left, at)).max(modulus(right, at));
    }
    // A pair of runs that produced nothing anywhere differ by nothing. Dividing
    // by one there returns that rather than a quotient of two zeroes, and it is
    // written as a positive comparison so the reader is not asked to negate one.
    let scale = if largest > zero() { largest } else { one() };

    let mut worst = zero();
    for at in 0..left.real.len() {
        let real = left.real[at] - right.real[at];
        let imaginary = left.imaginary[at] - right.imaginary[at];
        worst = worst.max(real.hypot(imaginary) / scale);
    }
    worst
}
