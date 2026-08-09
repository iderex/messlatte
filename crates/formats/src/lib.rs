//! Readers, writers and validators for the file formats.
//!
//! The trace file, the truth file, the case declaration, the submission and the
//! case index, with a validator per format. The trace file is here, in `trace`,
//! on the two containers it is made of: `npy` for the array and `json` for the
//! header. The rest of milestone 03 is not implemented here yet.
//!
//! The document beside the code is `docs/format/trace.md`, and what it holds
//! that this module cannot is the reasoning a reader outside this workspace
//! needs in order to write one of these files without reading Rust.

pub mod json;
pub mod npy;
pub mod trace;

pub use npy::Array;
pub use trace::{Axis, Cells, Electron, Trace};
