//! The one conversion layer between what a file says and what the numerics do
//! (#13).
//!
//! Atomic units inside, SI at the edge. A quantity read out of a file arrives
//! here with the unit the file stated, is converted once, and travels as a type
//! from then on. Going the other way, a quantity is converted once more at the
//! moment it is written. Nothing in between multiplies by anything.
//!
//! The types are why this is a layer rather than a pair of functions. An
//! [`Energy`] cannot be handed to something expecting a [`Time`], because they
//! are different types and not two `f64`s distinguished by the name of the
//! variable holding them. What that costs is a wrapper around a double; what it
//! buys is that the mistake this module exists to prevent stops the build.
//!
//! No compile-fail case proves that last sentence. Proving it needs a target
//! that compiles a program and expects it to fail, which is a dependency this
//! crate may not take: `layout.toml` gives the units role an empty list, over
//! every dependency kind. The claim is therefore a property of the signatures
//! below rather than a measurement, and it is written here as one.
//!
//! Every factor comes from [`crate::constants`] and none is written here, which
//! the invariants gate enforces by refusing a floating-point literal in this
//! file at all.

use core::fmt;

use crate::constants::{
    ATOMIC_UNIT_OF_MOMENTUM, ATOMIC_UNIT_OF_TIME, ATTO, ELEMENTARY_CHARGE, FEMTO, HARTREE_ENERGY,
};

/// A unit this layer was asked for and does not know.
///
/// It carries what was admitted rather than only what was refused, because the
/// caller is usually a reader holding a file somebody else wrote, and the next
/// thing that reader has to do is tell its author what to write instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownUnit {
    /// The quantity the unit was offered for.
    pub quantity: &'static str,
    /// The unit as the file spelled it.
    pub unit: String,
    /// The units this layer converts for that quantity.
    pub admitted: &'static [&'static str],
}

impl fmt::Display for UnknownUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a {} in {:?}, and this converts {}",
            self.quantity,
            self.unit,
            self.admitted.join(", ")
        )
    }
}

impl std::error::Error for UnknownUnit {}

fn unknown(quantity: &'static str, unit: &str, admitted: &'static [&'static str]) -> UnknownUnit {
    UnknownUnit {
        quantity,
        unit: unit.to_string(),
        admitted,
    }
}

/// An energy, carried in hartree.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Energy(f64);

impl Energy {
    /// The units a file may state an energy in.
    ///
    /// Small and closed rather than open. Every entry is SI, with the
    /// electronvolt the exception SI itself makes for it. Atomic units are not
    /// admitted in a file: they belong inside the numerics, and a file is what
    /// an operator reads.
    pub const UNITS: &'static [&'static str] = &["J", "eV"];

    /// From atomic units, which is what the numerics hold.
    #[must_use]
    pub const fn from_hartree(value: f64) -> Self {
        Self(value)
    }

    /// In atomic units.
    #[must_use]
    pub const fn in_hartree(self) -> f64 {
        self.0
    }

    /// From the SI value and unit a file carries.
    ///
    /// # Errors
    ///
    /// The unit is outside [`Energy::UNITS`].
    pub fn from_si(value: f64, unit: &str) -> Result<Self, UnknownUnit> {
        Ok(Self(value / Self::per_hartree(unit)?))
    }

    /// In the unit a file is to carry.
    ///
    /// # Errors
    ///
    /// The unit is outside [`Energy::UNITS`].
    pub fn in_si(self, unit: &str) -> Result<f64, UnknownUnit> {
        Ok(self.0 * Self::per_hartree(unit)?)
    }

    /// How many of `unit` there are in one hartree.
    fn per_hartree(unit: &str) -> Result<f64, UnknownUnit> {
        match unit {
            "J" => Ok(HARTREE_ENERGY.value),
            // The electronvolt is the elementary charge through one volt, so
            // the joules in an electronvolt are the coulombs in an elementary
            // charge, numerically. One entry serves both.
            "eV" => Ok(HARTREE_ENERGY.value / ELEMENTARY_CHARGE.value),
            _ => Err(unknown("energy", unit, Self::UNITS)),
        }
    }
}

/// A time, carried in atomic units of time.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Time(f64);

impl Time {
    /// The units a file may state a time in.
    pub const UNITS: &'static [&'static str] = &["s", "fs", "as"];

    /// From atomic units.
    #[must_use]
    pub const fn from_atomic(value: f64) -> Self {
        Self(value)
    }

    /// In atomic units.
    #[must_use]
    pub const fn in_atomic(self) -> f64 {
        self.0
    }

    /// From the SI value and unit a file carries.
    ///
    /// # Errors
    ///
    /// The unit is outside [`Time::UNITS`].
    pub fn from_si(value: f64, unit: &str) -> Result<Self, UnknownUnit> {
        Ok(Self(value / Self::per_atomic(unit)?))
    }

    /// In the unit a file is to carry.
    ///
    /// # Errors
    ///
    /// The unit is outside [`Time::UNITS`].
    pub fn in_si(self, unit: &str) -> Result<f64, UnknownUnit> {
        Ok(self.0 * Self::per_atomic(unit)?)
    }

    /// How many of `unit` there are in one atomic unit of time.
    fn per_atomic(unit: &str) -> Result<f64, UnknownUnit> {
        match unit {
            "s" => Ok(ATOMIC_UNIT_OF_TIME.value),
            "fs" => Ok(ATOMIC_UNIT_OF_TIME.value / FEMTO.value),
            "as" => Ok(ATOMIC_UNIT_OF_TIME.value / ATTO.value),
            _ => Err(unknown("time", unit, Self::UNITS)),
        }
    }
}

/// A momentum, carried in atomic units of momentum.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Momentum(f64);

impl Momentum {
    /// The units a file may state a momentum in.
    pub const UNITS: &'static [&'static str] = &["kg m/s"];

    /// From atomic units.
    #[must_use]
    pub const fn from_atomic(value: f64) -> Self {
        Self(value)
    }

    /// In atomic units.
    #[must_use]
    pub const fn in_atomic(self) -> f64 {
        self.0
    }

    /// From the SI value and unit a file carries.
    ///
    /// # Errors
    ///
    /// The unit is outside [`Momentum::UNITS`].
    pub fn from_si(value: f64, unit: &str) -> Result<Self, UnknownUnit> {
        Ok(Self(value / Self::per_atomic(unit)?))
    }

    /// In the unit a file is to carry.
    ///
    /// # Errors
    ///
    /// The unit is outside [`Momentum::UNITS`].
    pub fn in_si(self, unit: &str) -> Result<f64, UnknownUnit> {
        Ok(self.0 * Self::per_atomic(unit)?)
    }

    /// How many of `unit` there are in one atomic unit of momentum.
    fn per_atomic(unit: &str) -> Result<f64, UnknownUnit> {
        match unit {
            "kg m/s" => Ok(ATOMIC_UNIT_OF_MOMENTUM.value),
            _ => Err(unknown("momentum", unit, Self::UNITS)),
        }
    }
}
