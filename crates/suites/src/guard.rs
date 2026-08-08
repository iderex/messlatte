//! What a default-suite case may not do, judged from the text of the case.
//!
//! Two things are refused, and both come straight out of #14. A case in the
//! default suite binds no socket, and a case in the default suite writes
//! nowhere but its own temporary directory. The reason each is refused is the
//! same shape: a suite that needs a firewall answer or a writable path outside
//! its own directory is a suite that stops running on somebody's machine, and
//! once it stops running the gate means nothing. On a Windows host a socket
//! bound off loopback raises a dialog only an administrator can answer, which
//! turns one case into a prompt nobody sees in a pipeline.
//!
//! What this reads, and therefore what it cannot see. It reads characters. It
//! refuses the shapes named in the two tables below wherever they appear in a
//! default-suite file, a comment included, because a comment proposing the call
//! is the line that later stops being a comment. It does not resolve a name, so
//! a bind reached through an alias, through a helper in another crate or
//! through a macro is invisible to it, and so is a write whose path is built
//! somewhere else. It is a floor rather than a proof: it holds the shapes that
//! have a reason to be written by hand, and it will not catch one nobody has
//! written yet.
//!
//! The sanctioned way to write a file from a default-suite case is
//! [`crate::Scratch`], whose directory is under the temporary directory this
//! machine declares and is removed when the case ends. A case using it names
//! none of the tokens below.

use std::fmt;

/// The shapes that open a socket. Every one is a std entry point somebody would
/// type deliberately, which is what makes a text rule adequate for them.
const SOCKET_TOKENS: &[&str] = &[
    "TcpListener::bind",
    "TcpStream::connect",
    "UdpSocket::bind",
    "UnixListener::bind",
    "UnixStream::connect",
];

/// The shapes that write, create or remove a path. The trailing parenthesis is
/// part of the token so that the word alone in prose is not a finding.
const WRITE_TOKENS: &[&str] = &[
    "fs::write(",
    "File::create(",
    "File::create_new(",
    "OpenOptions::new(",
    "fs::create_dir(",
    "fs::create_dir_all(",
    "fs::remove_file(",
    "fs::remove_dir(",
    "fs::remove_dir_all(",
    "fs::copy(",
    "fs::rename(",
    "fs::hard_link(",
    "fs::set_permissions(",
];

/// The attribute a unit test sits behind, held in two pieces on purpose.
///
/// Written whole, this file would select itself as a default-suite file, and
/// then the tables above would make it their own first finding. The pieces
/// carry the same bytes and match nothing.
const UNIT_TEST_MARKER: &str = concat!("#[cfg(", "test)]");

/// One thing a default-suite case did that the default suite forbids.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breach {
    /// The case binds or connects a socket.
    BindsASocket {
        path: String,
        line: usize,
        token: String,
    },
    /// The case writes through a path this rule cannot see the destination of.
    WritesOutsideItsTemporaryDirectory {
        path: String,
        line: usize,
        token: String,
    },
}

impl fmt::Display for Breach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Breach::BindsASocket { path, line, token } => write!(
                f,
                "{path}:{line} names {token}, and a default-suite case binds no socket. \
                 A case that needs one belongs in a suite named for what it needs."
            ),
            Breach::WritesOutsideItsTemporaryDirectory { path, line, token } => write!(
                f,
                "{path}:{line} names {token}, and a default-suite case writes only inside its own \
                 temporary directory. Write through messlatte_suites::Scratch, which creates one \
                 and removes it when the case ends."
            ),
        }
    }
}

/// Whether a tracked file is a case in the default suite.
///
/// Two shapes count. A `.rs` file under a crate's `tests/` directory, which
/// cargo compiles as an integration test target. And a `.rs` file carrying the
/// unit-test attribute, which is how a case inside `src/` is written. Both run
/// under a plain `cargo test`, which is what makes them default-suite cases.
///
/// A case that is inside neither shape is not judged, and that is the bound
/// worth knowing: a helper crate compiled only under test, or a case generated
/// by a macro from somewhere else, is not read here.
pub fn is_default_suite_test(path: &str, source: &str) -> bool {
    let normalised = path.replace('\\', "/");
    if !normalised.ends_with(".rs") {
        return false;
    }
    normalised.contains("/tests/") || source.contains(UNIT_TEST_MARKER)
}

/// Every breach in one file, in the order the lines carry them.
///
/// The file is judged only if [`is_default_suite_test`] says it is one, so a
/// caller can hand this the whole tracked tree.
pub fn breaches(path: &str, source: &str) -> Vec<Breach> {
    if !is_default_suite_test(path, source) {
        return Vec::new();
    }

    let mut found = Vec::new();
    for (index, text) in source.lines().enumerate() {
        let line = index + 1;
        for token in SOCKET_TOKENS {
            if text.contains(token) {
                found.push(Breach::BindsASocket {
                    path: path.to_string(),
                    line,
                    token: (*token).to_string(),
                });
            }
        }
        for token in WRITE_TOKENS {
            if text.contains(token) {
                found.push(Breach::WritesOutsideItsTemporaryDirectory {
                    path: path.to_string(),
                    line,
                    token: (*token).to_string(),
                });
            }
        }
    }
    found
}
