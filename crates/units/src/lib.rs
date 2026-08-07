//! Units and physical constants.
//!
//! The bottom of the workspace. Atomic units inside the numerics, SI in every
//! file an operator reads or writes, one conversion layer between them and one
//! table of constants with a source per entry. That decision is #13 and none of
//! it is implemented here yet.

/// A seeded violation, added to prove the two gates go red and removed again
/// in the next commit. It is badly formatted and it compares two floats for
/// equality.
pub fn seeded_violation(a:f64,b:f64)->bool{a==b}
