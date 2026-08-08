//! The suite split, and the guard that holds the default suite to it (#14).
//!
//! Three things live here and they are one decision. Which suites exist and how
//! one is asked for, in `suite`. What a default-suite test may not do, in
//! `guard`. And the one sanctioned way for a default-suite test to write a file,
//! in `scratch`, because a rule that forbids every write and offers nothing in
//! its place is a rule somebody routes around.
//!
//! The crate holds no product code and nothing depends on it. It is a workspace
//! member so that `cargo test` runs it with everything else, which is the same
//! reason `messlatte-tree` is one.

pub mod guard;
pub mod scratch;
pub mod suite;

pub use guard::{breaches, is_default_suite_test, Breach};
pub use scratch::Scratch;
pub use suite::{enrol, report, Enrolment, Suite};
