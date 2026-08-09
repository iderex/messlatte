//! Units and physical constants.
//!
//! The bottom of the workspace. Atomic units inside the numerics, SI in every
//! file an operator reads or writes, one conversion layer between them and one
//! table of constants with a source per entry. That decision is #13.
//!
//! [`constants`] is the table, and it is the only file in the physics crates
//! allowed to hold a floating-point literal. [`convert`] is the layer, and it
//! holds none: every factor in it is read from the table.
//!
//! What is not here. Nothing propagates an uncertainty. The table states the
//! standard uncertainty each source gives, a converted quantity carries no
//! uncertainty of its own, and no route in this workspace turns the first into
//! the second.

pub mod constants;
pub mod convert;

pub use constants::{Constant, TABLE};
pub use convert::{Energy, Momentum, Time, UnknownUnit};
