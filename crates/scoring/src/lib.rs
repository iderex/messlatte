//! The quotient, the error measures and the clustering.
//!
//! This crate may not depend on any method crate, so a measure cannot be tuned
//! to the internals of the thing it scores.
//!
//! The quotient is here. The error measures are milestone 07 and the
//! clustering is #72, and neither is implemented here yet, so the obligation in
//! #8 that every measure and the clustering call the quotient has nothing to
//! read today and is not discharged by this crate's current contents.

pub mod quotient;

pub use quotient::{quotient, Aligned, Amplitude, Freedoms, Transformation};
