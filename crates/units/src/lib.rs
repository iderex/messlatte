//! Units and physical constants.
//!
//! The bottom of the workspace. Atomic units inside the numerics, SI in every
//! file an operator reads or writes, one conversion layer between them and one
//! table of constants with a source per entry. That decision is #13 and none of
//! it is implemented here yet.

/// A seeded use of a standard library method stabilised after the declared
/// floor, added to prove the gate goes red and removed again in the next
/// commit. `f64::next_up` is stable since 1.86 and the floor is 1.85.0.
pub fn seeded_above_the_floor(x: f64) -> f64 {
    x.next_up()
}
