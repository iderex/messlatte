//! Every unit a file format admits, round-tripped through the conversion layer
//! (#13).
//!
//! The set is taken from the format rather than written here. The format is the
//! authority for what a file may say, and a list of units restated in a test
//! goes stale the day the format grows one, in the direction that leaves the
//! test green.
//!
//! One format admits units today, the trace file of #35. The truth file, the
//! case declaration, the submission and the case index are #36 through #39 and
//! do not exist, so this covers the units of one format out of five and cannot
//! be read as covering the set the formats will eventually admit.

use messlatte_formats::trace::{Electron, DELAY_UNITS};
use messlatte_units::{Energy, Momentum, Time};

/// Two roundings and no more. The reasoning is in `crates/units/tests/units.rs`
/// beside the same constant.
const TOLERANCE: f64 = 4.0 * f64::EPSILON;

#[track_caller]
fn close(left: f64, right: f64) {
    let scale = left.abs().max(right.abs());
    assert!(
        (left - right).abs() <= TOLERANCE * scale,
        "{left:e} and {right:e} differ by more than {TOLERANCE:e} of the larger"
    );
}

const VALUES: &[f64] = &[1.0, 1e-30, 1e30, 137.036, 0.000_123];

#[track_caller]
fn assert_same_set(format: &[&str], layer: &[&str], quantity: &str) {
    for unit in format {
        assert!(
            layer.contains(unit),
            "the trace format admits a {quantity} in {unit:?} and the conversion layer does not \
             convert it, so a file this repository admits cannot be read into the numerics"
        );
    }
    for unit in layer {
        assert!(
            format.contains(unit),
            "the conversion layer converts a {quantity} in {unit:?} and no format admits it. \
             Either a format lost a unit or the layer grew one nothing writes"
        );
    }
}

#[test]
fn the_conversion_layer_covers_exactly_the_units_the_trace_format_admits() {
    // Both directions. A layer missing a unit is a file this repository writes
    // and cannot read; a layer with a spare one is a conversion nobody exercises
    // and nobody maintains, and it is the half a one-directional check misses.
    assert_same_set(Electron::Energy.units(), Energy::UNITS, "energy");
    assert_same_set(Electron::Momentum.units(), Momentum::UNITS, "momentum");
    assert_same_set(DELAY_UNITS, Time::UNITS, "time");
}

#[test]
fn every_unit_the_trace_format_admits_round_trips_through_atomic_units() {
    for value in VALUES {
        for unit in Electron::Energy.units() {
            let quantity = Energy::from_si(*value, unit).expect("the format admits it");
            close(quantity.in_si(unit).expect("the format admits it"), *value);
        }
        for unit in Electron::Momentum.units() {
            let quantity = Momentum::from_si(*value, unit).expect("the format admits it");
            close(quantity.in_si(unit).expect("the format admits it"), *value);
        }
        for unit in DELAY_UNITS {
            let quantity = Time::from_si(*value, unit).expect("the format admits it");
            close(quantity.in_si(unit).expect("the format admits it"), *value);
        }
    }
}

/// The joules in an electronvolt, and the seconds in a femtosecond and in an
/// attosecond, restated here on purpose.
///
/// Everything else in these two files reads its numbers out of the constant
/// table, which is the rule. A check that a factor is right cannot: comparing
/// the layer's factor against the entry the layer read it from passes whatever
/// the factor does with it. So the three numbers below are written again, from
/// the definitions rather than from the table, and they are the only numbers in
/// either test file that are not the table's.
///
/// All three are exact by definition and none of them can move. The first is
/// the SI definition of the elementary charge, and the electronvolt is the work
/// done moving one of them through one volt. The other two are SI prefixes.
const JOULES_PER_ELECTRONVOLT: f64 = 1.602_176_634e-19;
const SECONDS_PER_FEMTOSECOND: f64 = 1e-15;
const SECONDS_PER_ATTOSECOND: f64 = 1e-18;

#[test]
fn the_two_admitted_units_of_one_quantity_stand_in_the_ratio_that_defines_them() {
    // A round trip through one unit passes even if that unit's factor is wrong,
    // because the same wrong number is applied twice, and so does a comparison
    // against the table entry the factor was built from. This is the leg that
    // does not: one unit of each admitted spelling, converted to atomic units,
    // has to come out in the ratio the definitions above fix, and a wrong factor
    // moves only its own side of that ratio.
    let one_joule = Energy::from_si(1.0, "J").expect("J is admitted");
    let one_electronvolt = Energy::from_si(1.0, "eV").expect("eV is admitted");
    close(
        one_electronvolt.in_hartree(),
        one_joule.in_hartree() * JOULES_PER_ELECTRONVOLT,
    );

    let one_second = Time::from_si(1.0, "s").expect("s is admitted");
    let one_femtosecond = Time::from_si(1.0, "fs").expect("fs is admitted");
    let one_attosecond = Time::from_si(1.0, "as").expect("as is admitted");
    close(
        one_femtosecond.in_atomic(),
        one_second.in_atomic() * SECONDS_PER_FEMTOSECOND,
    );
    close(
        one_attosecond.in_atomic(),
        one_second.in_atomic() * SECONDS_PER_ATTOSECOND,
    );

    // Momentum admits one unit, so it has no such ratio and nothing here checks
    // its factor against anything but the table entry it came from. That is a
    // gap in this file and not a property of the layer, and it closes when a
    // format admits a second momentum unit.
}
