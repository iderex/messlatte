//! What the tracked tree itself may contain.
//!
//! The crate holds no product code. It exists so that a rule about the bytes
//! git stores has somewhere to live that is run by `cargo test` along with
//! everything else.

/// The paths `git grep` reported, given its output and its exit status.
///
/// `git grep -l` exits 0 when it found something, 1 when it found nothing, and
/// anything else on an error. Treating a non-zero status as "nothing found"
/// would turn a broken invocation into a clean result, which is the failure
/// this function is shaped to avoid: it returns an error for every status it
/// does not recognise, and the caller has nothing to mistake for a pass.
pub fn offenders(status: Option<i32>, stdout: &str, stderr: &str) -> Result<Vec<String>, String> {
    match status {
        Some(0) => {
            let paths: Vec<String> = stdout
                .lines()
                .map(str::trim_end)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect();
            if paths.is_empty() {
                return Err(
                    "git grep reported a match and named no path, so its output was not understood"
                        .to_string(),
                );
            }
            Ok(paths)
        }
        Some(1) => {
            if stdout.trim().is_empty() {
                Ok(Vec::new())
            } else {
                Err(format!(
                    "git grep reported no match and named paths anyway: {}",
                    stdout.trim()
                ))
            }
        }
        Some(other) => Err(format!(
            "git grep failed with status {other}: {}",
            stderr.trim()
        )),
        None => Err(format!(
            "git grep was killed by a signal: {}",
            stderr.trim()
        )),
    }
}
