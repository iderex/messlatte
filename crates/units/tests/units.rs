//! The conversion layer and the constant table (#13).
//!
//! The round trip that matters to the file formats is not here. It is in
//! `crates/formats/tests/units.rs`, because the set it has to cover is the set
//! a file format admits, and this crate may not depend on the crate that
//! declares that set: `layout.toml` gives the units role an empty list. What is
//! here is the layer judged against itself and the table judged against its own
//! rules.
//!
//! Every comparison below is a relative one with the tolerance written at the
//! site. A round trip is a divide and a multiply by one factor, which is two
//! roundings and therefore not the identity in binary floating point, so an
//! exact comparison here would be a test of the rounding and not of the layer.

use messlatte_units::constants::{
    ATOMIC_UNIT_OF_MOMENTUM, ATOMIC_UNIT_OF_TIME, ATTO, ELEMENTARY_CHARGE, FEMTO, HARTREE_ENERGY,
    TABLE,
};
use messlatte_units::{Energy, Momentum, Time};

/// Two roundings, so a few units in the last place and no more. Written as a
/// multiple of the machine epsilon rather than as a number, so it means the
/// same thing if this ever runs at another precision.
const TOLERANCE: f64 = 4.0 * f64::EPSILON;

#[track_caller]
fn close(left: f64, right: f64) {
    let scale = left.abs().max(right.abs());
    assert!(
        (left - right).abs() <= TOLERANCE * scale,
        "{left:e} and {right:e} differ by more than {TOLERANCE:e} of the larger"
    );
}

/// The magnitudes are the ones this field actually writes down, plus one at
/// each end, because a factor applied to a well-behaved number can hide an
/// overflow a real case would meet.
const VALUES: &[f64] = &[1.0, 1e-30, 1e30, 137.036, 0.000_123];

#[test]
fn every_admitted_energy_unit_survives_a_round_trip() {
    for unit in Energy::UNITS {
        for value in VALUES {
            let there_and_back = Energy::from_si(*value, unit)
                .expect("an admitted unit converts")
                .in_si(unit)
                .expect("an admitted unit converts");
            close(there_and_back, *value);
        }
    }
}

#[test]
fn every_admitted_time_unit_survives_a_round_trip() {
    for unit in Time::UNITS {
        for value in VALUES {
            let there_and_back = Time::from_si(*value, unit)
                .expect("an admitted unit converts")
                .in_si(unit)
                .expect("an admitted unit converts");
            close(there_and_back, *value);
        }
    }
}

#[test]
fn every_admitted_momentum_unit_survives_a_round_trip() {
    for unit in Momentum::UNITS {
        for value in VALUES {
            let there_and_back = Momentum::from_si(*value, unit)
                .expect("an admitted unit converts")
                .in_si(unit)
                .expect("an admitted unit converts");
            close(there_and_back, *value);
        }
    }
}

#[test]
fn a_unit_outside_the_admitted_set_is_refused_and_says_what_is_admitted() {
    let refusal = Energy::from_si(1.0, "hartree").expect_err("atomic units are not a file unit");
    assert_eq!(refusal.quantity, "energy");
    assert_eq!(refusal.unit, "hartree");
    assert_eq!(refusal.admitted, Energy::UNITS);
    let said = refusal.to_string();
    assert!(said.contains("J") && said.contains("eV"), "{said}");
}

#[test]
fn the_unit_a_file_states_is_matched_exactly_and_not_by_shape() {
    // The near-miss. A layer that trimmed, lower-cased or accepted a plural
    // would read "EV" as electronvolts, and the two are a factor of a hundred
    // million apart. Refusing is the only safe answer to a spelling nobody
    // wrote down.
    for spelling in ["ev", "EV", " eV", "eV ", "electronvolt"] {
        assert!(
            Energy::from_si(1.0, spelling).is_err(),
            "{spelling:?} was accepted"
        );
    }
}

#[test]
fn an_electronvolt_is_the_elementary_charge_in_joules() {
    // Not a restatement of the code: it is the definition the table's comment
    // leans on, checked against the entry the layer actually reads. A layer
    // that grew a second, drifted electronvolt entry would fail here.
    let one_electronvolt = Energy::from_si(1.0, "eV").expect("eV is admitted");
    close(
        one_electronvolt.in_si("J").expect("J is admitted"),
        ELEMENTARY_CHARGE.value,
    );
}

#[test]
fn a_femtosecond_is_a_thousand_attoseconds() {
    let one_femtosecond = Time::from_si(1.0, "fs").expect("fs is admitted");
    close(one_femtosecond.in_si("as").expect("as is admitted"), 1000.0);
}

#[test]
fn one_atomic_unit_is_what_the_table_says_it_is() {
    close(
        Energy::from_hartree(1.0).in_si("J").expect("J is admitted"),
        HARTREE_ENERGY.value,
    );
    close(
        Time::from_atomic(1.0).in_si("s").expect("s is admitted"),
        ATOMIC_UNIT_OF_TIME.value,
    );
    close(
        Momentum::from_atomic(1.0)
            .in_si("kg m/s")
            .expect("kg m/s is admitted"),
        ATOMIC_UNIT_OF_MOMENTUM.value,
    );
}

#[test]
fn every_entry_in_the_table_carries_what_an_entry_owes() {
    for entry in TABLE {
        assert!(!entry.name.is_empty(), "an entry with no name");
        assert!(!entry.unit.is_empty(), "{} carries no unit", entry.name);
        assert!(
            entry.source.len() > 20,
            "{} carries no source anybody could look up: {:?}",
            entry.name,
            entry.source
        );
        assert!(
            entry.value.is_finite() && entry.value > 0.0,
            "{} has no positive value",
            entry.name
        );
        if let Some(uncertainty) = entry.uncertainty {
            assert!(
                uncertainty > 0.0 && uncertainty < entry.value,
                "{} states an uncertainty of {uncertainty:e} against a value of {:e}",
                entry.name,
                entry.value
            );
        }
    }
}

#[test]
fn every_constant_this_crate_names_is_in_the_table() {
    // The near-miss is the omission rather than the addition. A constant added
    // above and left out of TABLE keeps working at its call site and is
    // invisible to anything that reads the table, so this counts rather than
    // trusting the list.
    let named = [
        HARTREE_ENERGY,
        ELEMENTARY_CHARGE,
        ATOMIC_UNIT_OF_TIME,
        ATOMIC_UNIT_OF_MOMENTUM,
        FEMTO,
        ATTO,
    ];
    assert_eq!(
        TABLE.len(),
        named.len(),
        "the table holds {} entries and this crate names {}",
        TABLE.len(),
        named.len()
    );
    for entry in named {
        assert!(TABLE.contains(&entry), "{} is not in the table", entry.name);
    }
}

#[test]
fn no_two_entries_share_a_name() {
    for (index, entry) in TABLE.iter().enumerate() {
        for other in &TABLE[index + 1..] {
            assert_ne!(entry.name, other.name, "two entries named {}", entry.name);
        }
    }
}
