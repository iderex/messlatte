//! The streaking field as a vector potential, and the phase a photoelectron
//! accumulates in it (#43).
//!
//! Atomic units throughout, which is what #13 fixes for the numerics. The
//! conventions, their signs and a worked example a reader can check with
//! arithmetic are in `docs/format/streaking-field.md`, and that document is the
//! authority for them. What is here is one implementation of it.
//!
//! Three things decide whether two people mean the same field by the same
//! numbers, and all three are choices rather than consequences.
//!
//! The potential is the primary quantity and the electric field is derived from
//! it. Written the other way round, the potential is an integral of the field
//! and carries a constant nothing in the field fixes, and the choice that fixes
//! it, that the potential vanishes long before and long after the pulse, then
//! holds only for a field whose time integral is zero. Taking the potential as
//! primary with an envelope that vanishes at both ends makes both true by
//! construction rather than by a condition somebody has to remember to impose.
//!
//! The carrier is referred to the envelope's peak. The envelope peaks at the
//! origin of time, so the carrier-envelope phase is the phase of the carrier
//! there, and a field with a carrier-envelope phase of zero has a potential at
//! its own maximum at the origin.
//!
//! The sign is the one that makes the electric field minus the time derivative
//! of the potential, in the dipole approximation and the Coulomb gauge, so the
//! potential depends on time and not on position. A sign error here is
//! invisible in a plot of a trace and reverses every statement a reconstruction
//! makes about the direction of the chirp, which is why it is written in the
//! document and checked by a case rather than left in the code.
//!
//! The phase keeps the full momentum dependence. It is not expanded about a
//! central momentum, and the squared-potential term is kept unless the caller
//! names the choice to drop it. The approximation this repository exists to
//! quantify is exactly the one a convenient forward model would also make, so
//! the generator may not make it by accident.

use core::f64::consts::PI;

/// Two, as a double, built from an integer.
///
/// Not a physical constant, so not in the table in `messlatte-units`: that
/// table holds quantities of the world, each with a source and an uncertainty,
/// and this is arithmetic. It is built from an integer because the invariants
/// rule refuses a floating-point literal in this crate's source, and that rule
/// is blunt on purpose, since nothing in a text pattern separates a measured
/// constant from a two (#13).
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

/// What the phase does with the squared-potential term.
///
/// A variant rather than a boolean, because the two readings of `true` here are
/// opposite and a caller reading the call site cannot tell which was meant. The
/// term is kept unless a caller names the other choice, and a case that drops
/// it says so in its own declaration, which is #37.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquaredTerm {
    /// The term is in the phase, which is what this repository generates with.
    Kept,
    /// The term is left out, which is an approximation a case declares.
    Dropped,
}

/// One reason a field cannot be built.
///
/// A field is refused rather than clamped. A pulse with no duration or no
/// frequency produces a trace that looks like a hard case and is a broken one,
/// which is the shape #47 is about at the level of the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// An amplitude that is not a number.
    AmplitudeNotFinite,
    /// A frequency that is not positive, so the carrier has no period.
    FrequencyNotPositive,
    /// A duration that is not positive, so the envelope has no support.
    CyclesNotPositive,
    /// A carrier-envelope phase that is not a number.
    PhaseNotFinite,
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Refusal::AmplitudeNotFinite => write!(f, "the amplitude is not a finite number"),
            Refusal::FrequencyNotPositive => write!(
                f,
                "the angular frequency is not positive, so the carrier has no period and the \
                 envelope has no length to be measured in"
            ),
            Refusal::CyclesNotPositive => write!(
                f,
                "the envelope spans no whole cycles, so the field has no support and every \
                 quantity derived from it is zero for a reason nobody chose"
            ),
            Refusal::PhaseNotFinite => {
                write!(f, "the carrier-envelope phase is not a finite number")
            }
        }
    }
}

/// The streaking field, in atomic units.
///
/// Built through [`StreakingField::new`], which refuses the parameters that
/// have no field behind them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreakingField {
    amplitude: f64,
    angular_frequency: f64,
    cycles: f64,
    carrier_envelope_phase: f64,
}

impl StreakingField {
    /// A field from its four parameters, in atomic units and radians.
    ///
    /// `cycles` is the full width of the envelope's support in optical cycles
    /// of the carrier, not a width at half maximum. The two differ by a factor
    /// nobody agrees on, and a number that has to be converted before it can be
    /// compared is a number that will be compared without being converted.
    ///
    /// # Errors
    ///
    /// One of the four parameters describes no field. See [`Refusal`].
    pub fn new(
        amplitude: f64,
        angular_frequency: f64,
        cycles: f64,
        carrier_envelope_phase: f64,
    ) -> Result<Self, Refusal> {
        if !amplitude.is_finite() {
            return Err(Refusal::AmplitudeNotFinite);
        }
        if !(angular_frequency.is_finite() && angular_frequency > f64::from(0_u8)) {
            return Err(Refusal::FrequencyNotPositive);
        }
        if !(cycles.is_finite() && cycles > f64::from(0_u8)) {
            return Err(Refusal::CyclesNotPositive);
        }
        if !carrier_envelope_phase.is_finite() {
            return Err(Refusal::PhaseNotFinite);
        }
        Ok(Self {
            amplitude,
            angular_frequency,
            cycles,
            carrier_envelope_phase,
        })
    }

    /// The peak of the vector potential.
    #[must_use]
    pub const fn amplitude(self) -> f64 {
        self.amplitude
    }

    /// The carrier's angular frequency.
    #[must_use]
    pub const fn angular_frequency(self) -> f64 {
        self.angular_frequency
    }

    /// The carrier's phase at the envelope's peak.
    #[must_use]
    pub const fn carrier_envelope_phase(self) -> f64 {
        self.carrier_envelope_phase
    }

    /// One optical period of the carrier.
    #[must_use]
    pub fn period(self) -> f64 {
        two() * PI / self.angular_frequency
    }

    /// Half the envelope's support, so the field is zero outside
    /// `-half_duration ..= half_duration`.
    #[must_use]
    pub fn half_duration(self) -> f64 {
        self.cycles * self.period() / two()
    }

    /// The envelope, peaking at one at the origin and reaching zero with zero
    /// slope at both ends of its support.
    ///
    /// A raised cosine rather than a Gaussian. A Gaussian never reaches zero,
    /// so a potential built on one is truncated somewhere, and where it is
    /// truncated is a parameter nobody declares and every implementation
    /// chooses differently. This one ends where it says it ends.
    #[must_use]
    pub fn envelope(self, time: f64) -> f64 {
        let half = self.half_duration();
        if !time.is_finite() || time.abs() > half {
            return f64::from(0_u8);
        }
        let shape = (PI * time / (two() * half)).cos();
        shape * shape
    }

    /// The derivative of the envelope with respect to time.
    fn envelope_slope(self, time: f64) -> f64 {
        let half = self.half_duration();
        if !time.is_finite() || time.abs() > half {
            return f64::from(0_u8);
        }
        // The derivative of the squared cosine, written as one sine so the
        // expression is the one in the document rather than a product a reader
        // has to fold.
        -(PI / (two() * half)) * (PI * time / half).sin()
    }

    /// The vector potential at a time, in atomic units.
    #[must_use]
    pub fn potential(self, time: f64) -> f64 {
        self.amplitude * self.envelope(time) * self.carrier(time).cos()
    }

    /// The electric field at a time, in atomic units.
    ///
    /// Minus the time derivative of [`StreakingField::potential`], written out
    /// rather than differenced, so the two agree to the precision of the
    /// arithmetic and not to the precision of a step size.
    #[must_use]
    pub fn electric_field(self, time: f64) -> f64 {
        let carrier = self.carrier(time);
        -self.amplitude
            * (self.envelope_slope(time) * carrier.cos()
                - self.angular_frequency * self.envelope(time) * carrier.sin())
    }

    /// The carrier's argument, referred to the envelope's peak.
    fn carrier(self, time: f64) -> f64 {
        self.angular_frequency * time + self.carrier_envelope_phase
    }
}

/// The phase a photoelectron accumulates in the streaking field (#43).
///
/// In the strong-field approximation and the velocity gauge, an electron born
/// at `birth` with asymptotic momentum `momentum` along the polarisation
/// accumulates
///
/// ```text
///     integral from birth to the end of the pulse of
///         momentum * potential(t) + potential(t) * potential(t) / 2
/// ```
///
/// The upper limit is the end of the envelope's support rather than infinity,
/// which is exact rather than a truncation: the potential is identically zero
/// outside it.
///
/// The momentum enters as itself. Expanding the first term about a central
/// momentum is the approximation whose cost this repository exists to measure,
/// so the generator does not make it, and a method that does is the method's
/// declaration to make.
#[derive(Debug, Clone, Copy)]
pub struct VolkovPhase {
    field: StreakingField,
    squared_term: SquaredTerm,
    intervals: u32,
}

impl VolkovPhase {
    /// The default number of quadrature intervals per optical cycle.
    ///
    /// Simpson's rule on a smooth periodic integrand, so the error falls as the
    /// fourth power of the step. What this number buys is measured in
    /// `crates/generator/tests/field.rs` against the derivative of the
    /// accumulated integral rather than asserted here.
    pub const INTERVALS_PER_CYCLE: u32 = 512;

    /// A phase over a field, with the squared term kept.
    #[must_use]
    pub const fn new(field: StreakingField) -> Self {
        Self {
            field,
            squared_term: SquaredTerm::Kept,
            intervals: Self::INTERVALS_PER_CYCLE,
        }
    }

    /// The same, with the squared-potential term left out.
    ///
    /// Named rather than defaulted. A case generated this way is a case with a
    /// declared approximation in it, and #37 is where the declaration lives.
    #[must_use]
    pub const fn without_squared_term(mut self) -> Self {
        self.squared_term = SquaredTerm::Dropped;
        self
    }

    /// The quadrature resolution, in intervals per optical cycle.
    ///
    /// Rounded up to an even number, because Simpson's rule pairs its
    /// intervals, and a caller asking for an odd count would otherwise get a
    /// silently different rule.
    #[must_use]
    pub const fn with_intervals_per_cycle(mut self, intervals: u32) -> Self {
        self.intervals = if intervals < 2 { 2 } else { intervals };
        self
    }

    /// The integral of the potential from `birth` to the end of the pulse.
    ///
    /// This is the coefficient the momentum multiplies, and it is public
    /// because it is the quantity a reader can check against the potential:
    /// its derivative with respect to `birth` is minus the potential there.
    #[must_use]
    pub fn displacement(&self, birth: f64) -> f64 {
        self.integrate(birth, |potential| potential)
    }

    /// The integral of half the squared potential from `birth` to the end.
    #[must_use]
    pub fn squared_integral(&self, birth: f64) -> f64 {
        self.integrate(birth, |potential| potential * potential / two())
    }

    /// The phase itself.
    #[must_use]
    pub fn at(&self, momentum: f64, birth: f64) -> f64 {
        let linear = momentum * self.displacement(birth);
        match self.squared_term {
            SquaredTerm::Kept => linear + self.squared_integral(birth),
            SquaredTerm::Dropped => linear,
        }
    }

    /// Composite Simpson's rule from `birth` to the end of the support.
    ///
    /// Zero once the pulse is over, and that is exact rather than a tolerance:
    /// the potential is identically zero there, so there is nothing left to
    /// accumulate.
    fn integrate(&self, birth: f64, integrand: impl Fn(f64) -> f64) -> f64 {
        let end = self.field.half_duration();
        if !birth.is_finite() || birth >= end {
            return f64::from(0_u8);
        }
        let start = birth.max(-end);
        let span = end - start;
        let cycles = span / self.field.period();
        // At least one pair of intervals however short the remaining span is,
        // so a birth time just before the end is integrated rather than
        // skipped.
        let wanted = cycles * f64::from(self.intervals);
        let count = if wanted.is_finite() && wanted > f64::from(2_u8) {
            // The ceiling to an even count. `wanted` is positive and finite
            // here, so the conversion cannot wrap or lose a sign.
            let ceiling = wanted.ceil().min(f64::from(u32::from(u16::MAX)));
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "bounded above by u16::MAX and below by two on the line above, and \
                          rounded before the conversion, so neither the high bits nor the sign \
                          can be lost"
            )]
            let ceiling = ceiling as u32;
            ceiling + (ceiling % 2)
        } else {
            2
        };

        let step = span / f64::from(count);
        let mut total =
            integrand(self.field.potential(start)) + integrand(self.field.potential(start + span));
        for index in 1..count {
            let weight = if index % 2 == 1 { four() } else { two() };
            let sample = self.field.potential(start + step * f64::from(index));
            total += weight * integrand(sample);
        }
        total * step / three()
    }
}
