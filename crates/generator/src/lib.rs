//! The forward model that produces a trace, and the perturbations applied to it.
//!
//! No reconstruction method may depend on this crate. A method whose trial trace
//! comes from the routine that made the data is fitting a model that is exactly
//! true, so its error measures the search rather than the physics. The direction
//! is declared in `layout.toml` and refused by `messlatte-layout`, and the
//! reasons are #5. Nothing is implemented here yet.
