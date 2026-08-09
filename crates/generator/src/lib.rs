//! The forward model that produces a trace, and the perturbations applied to it.
//!
//! No reconstruction method may depend on this crate. A method whose trial trace
//! comes from the routine that made the data is fitting a model that is exactly
//! true, so its error measures the search rather than the physics. The direction
//! is declared in `layout.toml` and refused by `messlatte-layout`, and the
//! reasons are #5.
//!
//! [`field`] is the streaking field and the phase an electron accumulates in
//! it, which is #43. [`operator`] is the model that turns them into a trace,
//! which is #42, and the form it prints is `docs/format/streaking-operator.md`.
//! The target's dipole is #44 and is a file format rather than code here. The
//! perturbations are milestone 05 and are not here.

pub mod field;
pub mod operator;

pub use field::{SquaredTerm, StreakingField, VolkovPhase};
pub use operator::{Grids, Pulse, Streaking, Target, PRINTED_FORM};
