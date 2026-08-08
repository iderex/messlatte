//! The one directory a default-suite case may write in.
//!
//! It sits under the temporary directory this machine declares, it is named so
//! that two cases running at once cannot collide, and it is removed when the
//! case ends. A case that writes through it names none of the tokens the guard
//! refuses, which is the point: the rule beside it forbids a shape and this is
//! what to write instead.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU32, Ordering};

/// Distinguishes two scratch directories made by one process. The process id
/// distinguishes two processes, and cargo runs test targets as separate
/// processes and cases inside one target as threads, so both halves are load
/// bearing.
static NEXT: AtomicU32 = AtomicU32::new(0);

/// One segment of a path, holding nothing that could leave the directory it is
/// joined to. A separator, a parent segment, an empty segment and a drive
/// letter are each refused, and the set is an allowance rather than a list of
/// what to reject, so a shape nobody thought of is refused by default.
fn is_plain_name(segment: &str) -> bool {
    !segment.is_empty()
        && segment != ".."
        && segment != "."
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// A directory under the machine's temporary directory, removed on drop.
#[derive(Debug)]
pub struct Scratch {
    path: PathBuf,
}

impl Scratch {
    /// A fresh directory, labelled so that a leftover one says which case made
    /// it.
    ///
    /// The label is refused unless it is letters, digits, hyphens and
    /// underscores. A label carrying a separator or a parent segment would
    /// place the directory somewhere other than under the temporary directory,
    /// which is the one thing this type exists to guarantee, and removing it on
    /// drop would then remove something else.
    pub fn new(label: &str) -> io::Result<Scratch> {
        if !is_plain_name(label) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "the label {label:?} is not a plain name, so the directory would not be under \
                     the temporary directory"
                ),
            ));
        }

        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("messlatte-{label}-{}-{serial}", process::id()));
        fs::create_dir_all(&path)?;
        Ok(Scratch { path })
    }

    /// Where it is.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Write a file inside it, creating the directories above it, and return
    /// where it went.
    ///
    /// The path is relative and its separator is a forward slash on every
    /// platform. It is refused unless every segment is a plain name, so a case
    /// cannot reach out of the directory through the path it writes to, and the
    /// removal on drop cannot be pointed at something else.
    pub fn write(&self, relative: &str, bytes: &[u8]) -> io::Result<PathBuf> {
        let mut target = self.path.clone();
        let segments: Vec<&str> = relative.split('/').collect();
        if segments.is_empty() || !segments.iter().all(|s| is_plain_name(s)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("the path {relative:?} is not a relative path inside this directory"),
            ));
        }
        for segment in segments {
            target.push(segment);
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, bytes)?;
        Ok(target)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // A failure to remove is not raised. The case has already decided
        // whether it passed, and turning a cleanup error into a failure would
        // redden a run for a reason the case is not about. What is left behind
        // is under the temporary directory and carries the label.
        let _ = fs::remove_dir_all(&self.path);
    }
}
