//! The streaking operator, and the form it prints (#42).
//!
//! One function turns a pulse, a streaking field, a target and two grids into a
//! trace, and one function prints the expression it evaluated. The document
//! beside them is `docs/format/streaking-operator.md`, which holds the printed
//! form verbatim so that a model which moves in the code and not in the
//! document reddens a case rather than being noticed later by a reader.
//!
//! Why the printed form is a requirement and not a nicety. The reviewers this
//! board needs are physicists who will not read Rust, and the difference
//! between a defensible benchmark and an elaborate mistake is one sign or one
//! factor inside this integral. A printed expression turns the review from an
//! exercise in reading an unfamiliar language into a comparison of two
//! expressions.
//!
//! Atomic units throughout. The grids arrive in atomic units and are converted
//! into the units a file admits at the one place the trace is built, which is
//! what #13 asks for.
//!
//! What this model leaves out is printed beside the expression rather than left
//! for a reader to infer from an absence, because the absence of a term is not
//! a claim that the term is negligible. The list is inside [`PRINTED_FORM`].

use messlatte_formats::dipole::{Lookup, Table};
use messlatte_formats::npy::Array;
use messlatte_formats::trace::{Axis, Cells, Electron, Trace};
use messlatte_units::{Energy, Momentum, Time};

use crate::field::{SquaredTerm, StreakingField, VolkovPhase};

/// Two, as a double, built from an integer.
///
/// The reasoning is the one at the same helper in [`crate::field`]: the
/// invariants rule refuses a floating-point literal in this crate's source, and
/// it is blunt on purpose because nothing in a text pattern separates a
/// measured constant from a two.
fn two() -> f64 {
    f64::from(2_u8)
}

/// Three, on the same reasoning. Simpson's rule needs it.
fn three() -> f64 {
    f64::from(3_u8)
}

/// Four, on the same reasoning.
fn four() -> f64 {
    f64::from(4_u8)
}

/// Zero, on the same reasoning.
fn zero() -> f64 {
    f64::from(0_u8)
}

/// The extreme-ultraviolet pulse, as a complex field on its own uniform time
/// grid.
///
/// Uniform and its own, which is what makes the operator exact in this factor.
/// The integral is taken over the pulse's samples, with the streaking field and
/// the phase evaluated at the shifted time, so nothing here ever interpolates
/// the pulse and a delay is not restricted to a whole number of samples.
///
/// What a truth file carries and how this is written down is #36, and this type
/// is the operator's input rather than that format.
#[derive(Debug, Clone, PartialEq)]
pub struct Pulse {
    /// The time of the first sample, in atomic units.
    pub first: f64,
    /// The spacing of the samples, in atomic units.
    pub step: f64,
    /// The real part of the field at each sample, in atomic units.
    pub real: Vec<f64>,
    /// The imaginary part, sample for sample.
    pub imaginary: Vec<f64>,
}

impl Pulse {
    /// How many samples the pulse carries.
    #[must_use]
    pub fn samples(&self) -> usize {
        self.real.len()
    }

    /// The time of one sample, in atomic units.
    #[must_use]
    pub fn time(&self, index: usize) -> f64 {
        self.first + self.step * index_as_f64(index)
    }
}

/// The streaking field and the two choices made about the phase in it.
///
/// The phase is built here rather than handed in, so a caller cannot pass a
/// phase computed over one field together with a different field. The two have
/// to be the same field for the expression to mean anything and nothing else in
/// this crate could check it.
#[derive(Debug, Clone, Copy)]
pub struct Streaking {
    pub field: StreakingField,
    /// Whether the squared-potential term is in the phase. A case that drops it
    /// declares it, which is #37.
    pub squared_term: SquaredTerm,
    /// The quadrature resolution the phase is integrated at.
    pub intervals_per_cycle: u32,
}

impl Streaking {
    /// A streaking field with the choices this repository generates with.
    #[must_use]
    pub fn new(field: StreakingField) -> Self {
        Self {
            field,
            squared_term: SquaredTerm::Kept,
            intervals_per_cycle: VolkovPhase::INTERVALS_PER_CYCLE,
        }
    }

    fn phase(self) -> VolkovPhase {
        let phase = VolkovPhase::new(self.field).with_intervals_per_cycle(self.intervals_per_cycle);
        match self.squared_term {
            SquaredTerm::Kept => phase,
            SquaredTerm::Dropped => phase.without_squared_term(),
        }
    }
}

/// The target: what is ionised and what its transition amplitude is.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    /// The ionisation potential, in atomic units.
    pub ionisation_potential: f64,
    /// The transition amplitude over the photoelectron's kinetic energy, which
    /// is #44 and `docs/format/dipole.md`.
    pub dipole: Table,
}

/// The two axes the trace is evaluated on, both in atomic units.
#[derive(Debug, Clone, PartialEq)]
pub struct Grids {
    /// Final momenta along the polarisation.
    pub momenta: Vec<f64>,
    /// Delays of the pulse relative to the streaking field's envelope peak.
    pub delays: Vec<f64>,
}

/// One reason a trace cannot be produced.
///
/// Refused rather than clamped, on the reasoning [`crate::field::Refusal`]
/// gives: a trace produced from parameters that describe nothing looks like a
/// hard case and is a broken one.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// The pulse grid cannot be integrated over: fewer than three samples, an
    /// even number of them, columns of different lengths, or a step that is not
    /// positive.
    ///
    /// The sample count is odd because the quadrature is composite Simpson,
    /// which pairs its intervals. A pulse with an even count would be
    /// integrated by a rule the caller did not choose.
    PulseGrid { detail: String },
    /// An axis with no samples on it, so the trace has no cells to be about.
    EmptyGrid { axis: String },
    /// A number in the grids or in the target that is not finite.
    NotFinite { what: String },
    /// The shifted momentum reached an energy the dipole table does not cover.
    ///
    /// A case whose momentum window reaches beyond its table is a case that
    /// needs a wider table. Extending the last sample instead would put an
    /// invented tail into the trace, and #44 refuses that at the table rather
    /// than here.
    DipoleOutsideItsRange { momentum: f64, detail: String },
    /// The trace the operator built is one the trace format refuses.
    NotATrace { detail: String },
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Refusal::PulseGrid { detail } => write!(f, "the pulse cannot be integrated: {detail}"),
            Refusal::EmptyGrid { axis } => write!(
                f,
                "the {axis} axis carries no samples, so the trace has no cells to be about"
            ),
            Refusal::NotFinite { what } => write!(f, "the {what} is not a finite number"),
            Refusal::DipoleOutsideItsRange { momentum, detail } => write!(
                f,
                "the shifted momentum at a final momentum of {momentum} left the dipole table: \
                 {detail}"
            ),
            Refusal::NotATrace { detail } => {
                write!(f, "the values are not a trace this format admits: {detail}")
            }
        }
    }
}

/// The expression the operator evaluates, its symbol table and its omissions.
///
/// Held here and reproduced verbatim in `docs/format/streaking-operator.md`,
/// with a case comparing the two, so the model cannot move in one without
/// moving in the other.
pub const PRINTED_FORM: &str = "\
The streaking operator, in atomic units.

    S(p, tau) = | a(p, tau) |^2

    a(p, tau) = integral over s of
                    d_amp(  e(s + tau) )
                  * exp( i * d_phase( e(s + tau) ) )
                  * E(s)
                  * exp( -i * phi(p, s + tau) )
                  * exp( i * ( p^2 / 2 + Ip ) * s )

    e(t)      = ( p + A(t) )^2 / 2

Symbols.

    p         the final momentum along the polarisation
    tau       the delay of the pulse relative to the streaking envelope's peak
    s         time on the pulse's own grid
    t         time referred to the streaking envelope's peak, t = s + tau
    E(s)      the complex extreme-ultraviolet field on that grid
    A(t)      the streaking field's vector potential, docs/format/streaking-field.md
    phi(p, t) the phase accumulated in the streaking field from t onwards, the
              same document. It enters with a minus beside a drift term that
              enters with a plus, which is what makes the exponent stationary
              where the instantaneous kinetic energy is the photon energy less
              the ionisation potential. Written with a plus, the trace streaks
              the wrong way and looks right.
    e(t)      the kinetic energy of the shifted momentum
    d_amp     the target's transition amplitude at that energy, docs/format/dipole.md
    d_phase   its phase at that energy, in this repository's convention
    Ip        the ionisation potential of the target

What is left out, and each is a case this board cannot currently speak about.

    no depletion of the ground state
    no space charge
    no propagation of either field through the target
    no vector character beyond the single declared polarisation direction
    one active electron
    one final state per target, unless a case declares more

Two factors of the amplitude are absent for arithmetic rather than physical
reasons, and both are stated because a reader comparing this with a publication
will look for them.

    the prefactor -i, which is a constant of modulus one
    the factor exp( i * ( p^2 / 2 + Ip ) * tau ), which does not depend on s

Neither survives the squared modulus, and the second is a large phase whose
cosine a double resolves badly, so it is dropped rather than computed and
discarded.

The integral is composite Simpson over the pulse's own samples, which is why the
pulse carries an odd number of them. The phase is integrated by the quadrature
`crates/generator/src/field.rs` describes.
";

/// One index as a double, which is a conversion this crate's lint set refuses
/// to make silently.
fn index_as_f64(index: usize) -> f64 {
    // A grid this repository can hold has far fewer samples than the integers a
    // double represents exactly, and the conversion is written once here rather
    // than at every site that needs it.
    let bounded = u32::try_from(index).unwrap_or(u32::MAX);
    f64::from(bounded)
}

/// The complex amplitude at every cell, before anything is divided by anything.
///
/// This is `a(p, tau)` in [`PRINTED_FORM`], and it is public because a trace is
/// not enough to compare two implementations of that expression with. The trace
/// is normalised by its own largest cell, so any error that multiplies the whole
/// of it is removed before a reader sees it, and a comparison made after that
/// division reports agreement for the reason that both sides were divided by
/// their own largest cell. #46 measured one such error: the fours and the twos
/// of the quadrature's weights swapped, which no case in this tree caught.
///
/// The cells are in row-major order over the momentum axis, so cell
/// `(row, column)` is at `row * delays + column`, which is the order
/// [`messlatte_formats::npy::Array`] holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Amplitudes {
    /// How many momenta the grid carried.
    pub momenta: usize,
    /// How many delays it carried.
    pub delays: usize,
    /// The real part of the amplitude at each cell, in atomic units.
    pub real: Vec<f64>,
    /// The imaginary part, cell for cell.
    pub imaginary: Vec<f64>,
}

impl Amplitudes {
    /// The squared modulus at one cell, which is the trace's value before the
    /// normalisation.
    ///
    /// # Panics
    ///
    /// The indices are outside the grid the amplitudes were built on.
    #[must_use]
    pub fn intensity(&self, momentum: usize, delay: usize) -> f64 {
        assert!(
            momentum < self.momenta && delay < self.delays,
            "cell ({momentum}, {delay}) is outside a grid of {} by {}",
            self.momenta,
            self.delays
        );
        let at = momentum * self.delays + delay;
        self.real[at] * self.real[at] + self.imaginary[at] * self.imaginary[at]
    }
}

/// The trace this model produces.
///
/// # Errors
///
/// The pulse, the grids or the target describe nothing this can evaluate, or
/// the shifted momentum leaves the dipole table. See [`Refusal`].
pub fn trace(
    case: &str,
    pulse: &Pulse,
    streaking: Streaking,
    target: &Target,
    grids: &Grids,
) -> Result<Trace, Refusal> {
    let found = amplitudes(pulse, streaking, target, grids)?;
    let cells = found
        .real
        .iter()
        .zip(&found.imaginary)
        .map(|(real, imaginary)| real * real + imaginary * imaginary)
        .collect();
    build(case, cells, grids)
}

/// The amplitude the trace is the squared modulus of, on the same grids.
///
/// # Errors
///
/// The same set [`trace`] refuses, and for the same reasons: this is the half of
/// it that evaluates the expression.
pub fn amplitudes(
    pulse: &Pulse,
    streaking: Streaking,
    target: &Target,
    grids: &Grids,
) -> Result<Amplitudes, Refusal> {
    check(pulse, target, grids)?;

    let phase = streaking.phase();
    let field = streaking.field;
    let samples = pulse.samples();

    // The phase and the potential depend on the time and not on the final
    // momentum, so they are evaluated once per (delay, sample) pair rather than
    // once per cell. phi(p, t) is p times the displacement plus the squared
    // integral, and both of those are what the field's own quadrature returns.
    let mut potential = vec![zero(); grids.delays.len() * samples];
    let mut displacement = vec![zero(); grids.delays.len() * samples];
    let mut squared = vec![zero(); grids.delays.len() * samples];
    for (column, delay) in grids.delays.iter().enumerate() {
        for index in 0..samples {
            let time = pulse.time(index) + delay;
            let at = column * samples + index;
            potential[at] = field.potential(time);
            displacement[at] = phase.displacement(time);
            squared[at] = phase.at(zero(), time);
        }
    }

    let cells = grids.momenta.len() * grids.delays.len();
    let mut real_part = vec![zero(); cells];
    let mut imaginary_part = vec![zero(); cells];
    for (row, momentum) in grids.momenta.iter().enumerate() {
        let drift = momentum * momentum / two() + target.ionisation_potential;
        for (column, _) in grids.delays.iter().enumerate() {
            let mut real = zero();
            let mut imaginary = zero();
            for index in 0..samples {
                let at = column * samples + index;
                let shifted = momentum + potential[at];
                let energy = shifted * shifted / two();
                let dipole = target
                    .dipole
                    .at(Energy::from_hartree(energy))
                    .map_err(|lookup| outside(*momentum, &lookup))?;

                let angle = dipole.phase - momentum * displacement[at] - squared[at]
                    + drift * pulse.time(index);
                let weight = simpson_weight(index, samples);
                let scale = weight * dipole.amplitude;
                real += scale
                    * (pulse.real[index] * angle.cos() - pulse.imaginary[index] * angle.sin());
                imaginary += scale
                    * (pulse.real[index] * angle.sin() + pulse.imaginary[index] * angle.cos());
            }
            let step = pulse.step / three();
            let at = row * grids.delays.len() + column;
            real_part[at] = real * step;
            imaginary_part[at] = imaginary * step;
        }
    }

    Ok(Amplitudes {
        momenta: grids.momenta.len(),
        delays: grids.delays.len(),
        real: real_part,
        imaginary: imaginary_part,
    })
}

/// The composite Simpson weight of one sample of an odd-length grid.
fn simpson_weight(index: usize, samples: usize) -> f64 {
    if index == 0 || index + 1 == samples {
        f64::from(1_u8)
    } else if index % 2 == 1 {
        four()
    } else {
        two()
    }
}

fn outside(momentum: f64, lookup: &Lookup) -> Refusal {
    Refusal::DipoleOutsideItsRange {
        momentum,
        detail: lookup.to_string(),
    }
}

/// Everything that stops the operator before it evaluates anything.
///
/// Shared with [`crate::quadrature`] rather than written twice. The two
/// implementations are compared on their arithmetic, and two copies of a
/// validation would let them disagree about which inputs they are comparable on
/// without either being wrong.
pub(crate) fn check(pulse: &Pulse, target: &Target, grids: &Grids) -> Result<(), Refusal> {
    let samples = pulse.samples();
    if samples != pulse.imaginary.len() {
        return Err(Refusal::PulseGrid {
            detail: format!(
                "the real part carries {samples} samples and the imaginary part carries {}, and \
                 a sample is both",
                pulse.imaginary.len()
            ),
        });
    }
    if samples < 3 || samples % 2 == 0 {
        return Err(Refusal::PulseGrid {
            detail: format!(
                "the pulse carries {samples} samples. Composite Simpson pairs its intervals, so \
                 an odd count of at least three is what it can be applied to"
            ),
        });
    }
    if !(pulse.step.is_finite() && pulse.step > zero()) || !pulse.first.is_finite() {
        return Err(Refusal::PulseGrid {
            detail: "the grid's first sample or its step is not a positive finite number"
                .to_string(),
        });
    }
    for (index, value) in pulse.real.iter().zip(&pulse.imaginary).enumerate() {
        if !(value.0.is_finite() && value.1.is_finite()) {
            return Err(Refusal::NotFinite {
                what: format!("pulse sample {index}"),
            });
        }
    }
    if !target.ionisation_potential.is_finite() {
        return Err(Refusal::NotFinite {
            what: "ionisation potential".to_string(),
        });
    }
    for (axis, values) in [("momentum", &grids.momenta), ("delay", &grids.delays)] {
        if values.is_empty() {
            return Err(Refusal::EmptyGrid {
                axis: axis.to_string(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(Refusal::NotFinite {
                    what: format!("{axis} sample {index}"),
                });
            }
        }
    }
    Ok(())
}

/// The trace, with its axes converted into the units a file admits.
///
/// The values are on no absolute scale, so they are normalised counts with the
/// sentence saying what they were divided by, which is what the trace format
/// requires of anything that is not raw counts. A case declaring an absolute
/// photon number is #48 and does not exist.
fn build(case: &str, cells: Vec<f64>, grids: &Grids) -> Result<Trace, Refusal> {
    let largest = cells.iter().copied().fold(zero(), f64::max);
    let divisor = if largest > zero() {
        largest
    } else {
        f64::from(1_u8)
    };
    let values = cells.iter().map(|cell| cell / divisor).collect();
    let array = Array::new(grids.momenta.len(), grids.delays.len(), values)
        .map_err(|detail| Refusal::NotATrace { detail })?;

    let mut electron = Vec::with_capacity(grids.momenta.len());
    for value in &grids.momenta {
        electron.push(
            Momentum::from_atomic(*value)
                .in_si("kg m/s")
                .map_err(|unknown| Refusal::NotATrace {
                    detail: unknown.to_string(),
                })?,
        );
    }
    let mut delay = Vec::with_capacity(grids.delays.len());
    for value in &grids.delays {
        delay.push(Time::from_atomic(*value).in_si("as").map_err(|unknown| {
            Refusal::NotATrace {
                detail: unknown.to_string(),
            }
        })?);
    }

    let trace = Trace {
        case: case.to_string(),
        electron_quantity: Electron::Momentum,
        electron: Axis {
            unit: "kg m/s".to_string(),
            values: electron,
        },
        delay: Axis {
            unit: "as".to_string(),
            values: delay,
        },
        cells: Cells::Normalised {
            normalisation: "the largest cell in this trace. The operator returns a squared \
                            modulus in atomic units and no case declares an absolute photon \
                            number yet, so these are counts on no scale but their own"
                .to_string(),
        },
        values: array,
    };
    match trace.to_bytes() {
        Ok(_) => Ok(trace),
        Err(refusals) => Err(Refusal::NotATrace {
            detail: refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("; "),
        }),
    }
}
