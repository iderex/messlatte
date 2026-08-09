//! The one place a physical constant lives (#13).
//!
//! Every number in this file is a measured or defined quantity, and every entry
//! carries where the value came from. Nothing else in the numerical crates may
//! hold one: the greppable invariants gate refuses a floating-point literal in
//! the physics crates outside this file, so a constant written at the site that
//! wants it is refused rather than reviewed.
//!
//! Why the table rather than a constant beside its use. A conversion factor
//! written where it is needed is written twice within a year, the second copy
//! carries a typo or a different revision of the same measurement, and the two
//! disagree by an amount too small to look wrong on a plot. This field publishes
//! in attoseconds and electronvolts while the equations are written in atomic
//! units, so that disagreement lands in a number an eye cannot check.
//!
//! What an entry does not carry. There is no correlation between entries here,
//! and the uncertainties are the standard uncertainties the source states rather
//! than anything propagated through a conversion. Nothing in this workspace
//! propagates them yet, so a converted quantity carries no uncertainty of its
//! own, and a reader should not read one into it.

/// One constant, with what is known about it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constant {
    /// The name the source gives it, so an entry can be found again in that
    /// source rather than recognised by its digits.
    pub name: &'static str,
    /// The value, in [`Constant::unit`].
    pub value: f64,
    /// The standard uncertainty the source states, in the same unit, or `None`
    /// where the source states the value as exact.
    ///
    /// `None` and a very small number are different statements. An exact value
    /// is exact because a definition fixes it, and no future measurement moves
    /// it; a small uncertainty is a measurement that may move.
    pub uncertainty: Option<f64>,
    pub unit: &'static str,
    /// Where the value came from, given precisely enough to look up.
    pub source: &'static str,
}

/// The NIST reference every measured entry below is read from.
const CODATA: &str = "CODATA 2022 recommended values, NIST reference on constants, units and \
                      uncertainty, physics.nist.gov/cgi-bin/cuu/Value";

/// The SI Brochure, for the two entries that are definitions rather than
/// measurements.
const SI: &str = "The International System of Units (SI), 9th edition, BIPM, table of SI \
                  prefixes";

/// The Hartree energy, the atomic unit of energy.
pub const HARTREE_ENERGY: Constant = Constant {
    name: "Hartree energy",
    value: 4.359_744_722_206_0e-18,
    uncertainty: Some(0.000_000_000_004_8e-18),
    unit: "J",
    source: CODATA,
};

/// The elementary charge.
///
/// It is here as the joules in an electronvolt as well as as a charge. The
/// electronvolt is the work done moving one elementary charge through a
/// potential difference of one volt, so the number that converts the two is
/// this one and there is no second entry for it. That is the point of the
/// table: a separate electronvolt entry would be a second copy of this
/// measurement, free to drift against it.
pub const ELEMENTARY_CHARGE: Constant = Constant {
    name: "elementary charge",
    value: 1.602_176_634e-19,
    uncertainty: None,
    unit: "C",
    source: CODATA,
};

/// The atomic unit of time.
pub const ATOMIC_UNIT_OF_TIME: Constant = Constant {
    name: "atomic unit of time",
    value: 2.418_884_326_586_4e-17,
    uncertainty: Some(0.000_000_000_002_6e-17),
    unit: "s",
    source: CODATA,
};

/// The atomic unit of momentum.
pub const ATOMIC_UNIT_OF_MOMENTUM: Constant = Constant {
    name: "atomic unit of momentum",
    value: 1.992_851_915_45e-24,
    uncertainty: Some(0.000_000_000_31e-24),
    unit: "kg m/s",
    source: CODATA,
};

/// The SI prefix femto.
pub const FEMTO: Constant = Constant {
    name: "femto",
    value: 1e-15,
    uncertainty: None,
    unit: "1",
    source: SI,
};

/// The SI prefix atto.
pub const ATTO: Constant = Constant {
    name: "atto",
    value: 1e-18,
    uncertainty: None,
    unit: "1",
    source: SI,
};

/// Every entry, so the table can be read as a table rather than as a list of
/// names somebody has to know to ask for.
///
/// A constant added above and left out of this is invisible to anything that
/// prints or checks the table, which is why the check on it counts entries
/// rather than trusting the list.
pub const TABLE: &[Constant] = &[
    HARTREE_ENERGY,
    ELEMENTARY_CHARGE,
    ATOMIC_UNIT_OF_TIME,
    ATOMIC_UNIT_OF_MOMENTUM,
    FEMTO,
    ATTO,
];
