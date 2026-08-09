//! Readers, writers and validators for the file formats.
//!
//! The trace file, the truth file, the case declaration, the submission and the
//! case index, with a validator per format. The trace file is here, in `trace`,
//! on the two containers it is made of: `npy` for the array and `json` for the
//! header. The rest of milestone 03 is not implemented here yet.
//!
//! `dipole` sits beside them and is not one of them. It is the target's
//! transition matrix element as data, which is #44 and belongs to the forward
//! model rather than to a case, and it is here because a method reads one too
//! and no method may reach the generator.
//!
//! The documents beside the code are `docs/format/trace.md` and
//! `docs/format/dipole.md`, and what they hold that these modules cannot is the
//! reasoning a reader outside this workspace needs in order to write one of
//! these files without reading Rust.

pub mod dipole;
pub mod json;
pub mod npy;
pub mod trace;

pub use dipole::{Convention, Dipole, Table};
pub use npy::Array;
pub use trace::{Axis, Cells, Electron, Trace};
