//! The freedoms removed before two fields are compared, in one place (#8).
//!
//! Three are removed always: a constant phase offset, a shift of the time
//! origin, and, where the case admits it, the sign of the time axis. A fourth,
//! a global amplitude scale, is removed in every case that does not declare an
//! absolute photon number. Without this a reconstruction that is perfect apart
//! from arriving two samples late scores as a large error, and an ensemble of
//! perfect reconstructions with random offsets scores as a wide interval, so
//! the number this repository reports would be a symmetry rather than a method.
//!
//! The alignment is computed rather than tuned. For a fixed orientation and a
//! fixed shift the complex scale that minimises the residual is the ratio of
//! two inner products and has a closed form, so nothing here converges on
//! anything. The shift is the one exhaustive part: every whole-sample lag is
//! tried and the best is kept, which costs the square of the sample count and
//! is what the doc comment on [`quotient`] states.
//!
//! What it does not remove, written here rather than left to be discovered. A
//! shift that is not a whole number of samples, because moving a field by part
//! of a sample means resampling it and this returns the candidate's own values
//! rather than an interpolation of them. The time-reversed complex conjugate,
//! which is a different freedom from the reversal named in #8 and is not one of
//! these unless a geometry is shown to produce it. And anything spectral: the
//! measures in milestone 07 that work on a spectral phase remove their own
//! linear term, and that removal is theirs to state.

/// A complex field value.
///
/// Held here rather than in `messlatte-units`, which is about physical
/// quantities, and rather than in `messlatte-formats`, which has no field type
/// yet. When the truth and submission formats land in milestone 03 they bring
/// one, and this moves to it rather than staying beside it.
#[derive(Debug, Clone, Copy, Default)]
pub struct Amplitude {
    pub re: f64,
    pub im: f64,
}

impl Amplitude {
    /// A value from its real and imaginary parts.
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// The complex conjugate.
    pub const fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    /// The squared modulus, which is what an inner product accumulates.
    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// The modulus.
    pub fn abs(self) -> f64 {
        self.norm_sqr().sqrt()
    }

    /// The argument, which is the constant phase when this value is a scale.
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// The complex product.
    pub fn times(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    /// The product with a real factor.
    pub fn scaled(self, factor: f64) -> Self {
        Self::new(self.re * factor, self.im * factor)
    }

    fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
}

/// Which freedoms this case admits.
///
/// Neither is a default anybody should inherit. `amplitude_scale` is true
/// unless the case declares an absolute photon number, in which case the
/// magnitude is a measured quantity and removing it would hide an error.
/// `time_reversal` is true only where the trace geometry genuinely cannot tell
/// a pulse from its reverse; folding the two together elsewhere would report a
/// wrong answer as a right one.
#[derive(Debug, Clone, Copy)]
pub struct Freedoms {
    pub amplitude_scale: bool,
    pub time_reversal: bool,
}

/// The share of the candidate's own energy a displacement may push off the end
/// of the grid.
///
/// It exists because the shift is otherwise a way of throwing the candidate
/// away. With an absolute photon number declared the scale cannot absorb a
/// wrong magnitude, so a candidate three times too large scores better as an
/// empty grid than as itself, and the quotient would report the error of a
/// submission nobody made. A displacement that discards field is not a shift of
/// the time origin, it is a different field.
///
/// The number is the double-precision floor rather than a judgement: a field
/// whose tail carries this share of its energy is at the rounding level of the
/// sums below, so no measure could see the difference. A pulse close enough to
/// the edge of its own grid for this to bind is a case whose grid is too short,
/// and the refusal is the right answer there.
const DISCARDABLE: f64 = 1e-12;

/// What was done to the candidate to line it up with the reference.
///
/// Returned rather than discarded because a reconstruction that needed a large
/// shift is a different fact from one that needed none, and the shift is worth
/// reporting. The two are ordered: the candidate is read backwards first, and
/// the shift is then in the samples of that reversed axis.
#[derive(Debug, Clone, Copy)]
pub struct Transformation {
    /// Whether the candidate was read backwards.
    pub reversed: bool,
    /// Whole samples the candidate was moved by, positive towards later times.
    pub shift: isize,
    /// The complex scale applied. Its argument is the constant phase removed
    /// and its modulus is the amplitude scale, which is one where the case
    /// declares an absolute photon number.
    pub scale: Amplitude,
}

/// The candidate after the quotient, what was done to it, and what is left.
#[derive(Debug, Clone)]
pub struct Aligned {
    pub candidate: Vec<Amplitude>,
    pub transformation: Transformation,
    /// The residual after alignment, relative to the reference's own norm, so
    /// that it does not carry the units of the field.
    pub distance: f64,
}

/// The quotient. Every error measure and the clustering call this rather than
/// aligning on their own, so that no two of them can disagree about what the
/// measurement determines.
///
/// The cost is the square of the sample count: every whole-sample lag from one
/// end to the other is tried, and each one is an inner product over the grid.
/// Ties are broken by the order the lags are tried, which is the forward
/// orientation before the reversed one and the smallest displacement first, so
/// two runs on the same input return the same transformation.
///
/// It refuses rather than repairs. Slices of different lengths, an empty
/// slice, a non-finite value, and a reference whose norm is zero are each an
/// error: the last one because a relative distance has nothing to be relative
/// to, and reporting zero there would read as a perfect reconstruction.
pub fn quotient(
    reference: &[Amplitude],
    candidate: &[Amplitude],
    freedoms: Freedoms,
) -> Result<Aligned, String> {
    if reference.len() != candidate.len() {
        return Err(format!(
            "the reference holds {} samples and the candidate holds {}, so they are not on one grid",
            reference.len(),
            candidate.len()
        ));
    }
    if reference.is_empty() {
        return Err("an empty grid has no field on it to compare".to_string());
    }
    if let Some(index) = reference.iter().position(|value| !value.is_finite()) {
        return Err(format!(
            "the reference holds a non-finite value at sample {index}"
        ));
    }
    if let Some(index) = candidate.iter().position(|value| !value.is_finite()) {
        return Err(format!(
            "the candidate holds a non-finite value at sample {index}"
        ));
    }

    let reference_norm_sqr: f64 = reference.iter().map(|value| value.norm_sqr()).sum();
    if reference_norm_sqr <= 0.0 {
        return Err(
            "the reference field is zero everywhere, so a relative distance to it is undefined"
                .to_string(),
        );
    }

    let reversed_candidate: Vec<Amplitude> = candidate.iter().copied().rev().collect();
    let mut orientations = vec![(false, candidate)];
    if freedoms.time_reversal {
        orientations.push((true, reversed_candidate.as_slice()));
    }

    let candidate_norm_sqr: f64 = candidate.iter().map(|value| value.norm_sqr()).sum();

    let mut best: Option<(f64, Aligned)> = None;
    for (reversed, oriented) in orientations {
        for (later, magnitude) in displacements(oriented.len()) {
            if discarded(oriented, later, magnitude) > DISCARDABLE * candidate_norm_sqr {
                continue;
            }
            let moved = displaced(oriented, later, magnitude);
            let Some((scale, residual_sqr)) = fit(reference, &moved, freedoms) else {
                continue;
            };
            if best
                .as_ref()
                .is_some_and(|(lowest, _)| residual_sqr >= *lowest)
            {
                continue;
            }
            let samples = isize::try_from(magnitude)
                .map_err(|_| format!("a displacement of {magnitude} samples does not fit"))?;
            best = Some((
                residual_sqr,
                Aligned {
                    candidate: moved.iter().map(|value| value.times(scale)).collect(),
                    transformation: Transformation {
                        reversed,
                        shift: if later { samples } else { -samples },
                        scale,
                    },
                    distance: (residual_sqr / reference_norm_sqr).max(0.0).sqrt(),
                },
            ));
        }
    }

    best.map(|(_, aligned)| aligned).ok_or_else(|| {
        "the candidate is zero at every displacement, so no alignment to the reference exists"
            .to_string()
    })
}

/// Every whole-sample displacement of a grid of this length, each one once, in
/// the order the tie-break depends on.
fn displacements(length: usize) -> impl Iterator<Item = (bool, usize)> {
    std::iter::once((true, 0)).chain((1..length).flat_map(|magnitude| {
        std::iter::once((true, magnitude)).chain(std::iter::once((false, magnitude)))
    }))
}

/// The candidate moved by whole samples, with zeros where it moved in from.
///
/// Zero fill rather than a wrap, because a pulse that left one end of the grid
/// did not arrive at the other and a wrap would compare it against a part of
/// the reference it never overlapped.
fn displaced(candidate: &[Amplitude], later: bool, magnitude: usize) -> Vec<Amplitude> {
    let kept = candidate.len() - magnitude;
    let mut moved = vec![Amplitude::default(); candidate.len()];
    if later {
        moved[magnitude..].copy_from_slice(&candidate[..kept]);
    } else {
        moved[..kept].copy_from_slice(&candidate[magnitude..]);
    }
    moved
}

/// The energy a displacement would push off the end of the grid.
fn discarded(candidate: &[Amplitude], later: bool, magnitude: usize) -> f64 {
    let leaving = if later {
        &candidate[candidate.len() - magnitude..]
    } else {
        &candidate[..magnitude]
    };
    leaving.iter().map(|value| value.norm_sqr()).sum()
}

/// The scale that minimises the residual for one displacement, and that
/// residual, or nothing where the displaced candidate carries no field at all.
///
/// With the amplitude free this is the least-squares ratio of inner products.
/// With it fixed the same argument gives the phase and the modulus is held at
/// one, so a candidate that is right in shape and wrong in size keeps the size
/// error the case asked to be scored on.
///
/// The residual is then summed difference by difference rather than expanded
/// into norms and an overlap. The expanded form is three numbers the size of
/// the field's own energy cancelling down to something near zero, which puts a
/// floor of the square root of the machine epsilon under every distance this
/// returns: a pair differing by nothing measured 1e-8 apart before this was
/// written the other way round.
fn fit(
    reference: &[Amplitude],
    moved: &[Amplitude],
    freedoms: Freedoms,
) -> Option<(Amplitude, f64)> {
    let mut overlap = Amplitude::default();
    let mut moved_norm_sqr = 0.0;
    for (left, right) in moved.iter().zip(reference.iter()) {
        let term = left.conj().times(*right);
        overlap = Amplitude::new(overlap.re + term.re, overlap.im + term.im);
        moved_norm_sqr += left.norm_sqr();
    }
    if moved_norm_sqr <= 0.0 {
        return None;
    }

    let scale = if freedoms.amplitude_scale {
        overlap.scaled(1.0 / moved_norm_sqr)
    } else {
        let modulus = overlap.abs();
        if modulus <= 0.0 {
            Amplitude::new(1.0, 0.0)
        } else {
            overlap.scaled(1.0 / modulus)
        }
    };

    let residual_sqr = moved
        .iter()
        .zip(reference.iter())
        .map(|(left, right)| {
            let fitted = left.times(scale);
            Amplitude::new(right.re - fitted.re, right.im - fitted.im).norm_sqr()
        })
        .sum();
    Some((scale, residual_sqr))
}
