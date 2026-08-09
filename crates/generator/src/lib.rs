//! The forward model that produces a trace, and the perturbations applied to it.
//!
//! No reconstruction method may depend on this crate. A method whose trial trace
//! comes from the routine that made the data is fitting a model that is exactly
//! true, so its error measures the search rather than the physics. The direction
//! is declared in `layout.toml` and refused by `messlatte-layout`, and the
//! reasons are #5.
//!
//! [`field`] is the streaking field and the phase an electron accumulates in
//! it, which is #43. The streaking operator that uses them is #42, the target's
//! dipole is #44 and the perturbations are milestone 05. None of those is here.

pub mod field;

pub use field::{Refusal, SquaredTerm, StreakingField, VolkovPhase};
