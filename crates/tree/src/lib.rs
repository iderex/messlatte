//! What the tracked tree itself may contain.
//!
//! The crate holds no product code. It exists so that a rule about the bytes
//! git stores has somewhere to live that is run by `cargo test` along with
//! everything else.

/// The workspace members a root manifest declares.
///
/// It reads the `members` list out of `[workspace]` by scanning for the key and
/// taking the quoted strings until the closing bracket. That is a reader for
/// this one manifest rather than a TOML parser, deliberately: `messlatte-tree`
/// may depend on nothing, which is what `layout.toml` declares for its role, so
/// a parser here would be a dependency the layout check refuses.
///
/// What that costs is worth stating. A `members` key inside some other table, or
/// a member path written with an escape in it, would be read wrongly. Neither is
/// in this manifest, and a manifest that grew either would be caught by the
/// member count rather than passing quietly, because every member this misses is
/// a member the caller then never checks.
pub fn workspace_members(manifest: &str) -> Result<Vec<String>, String> {
    let rest = manifest
        .split_once("members")
        .map(|(_, rest)| rest)
        .ok_or_else(|| "the manifest declares no members list".to_string())?;
    let rest = rest
        .split_once('[')
        .map(|(_, rest)| rest)
        .ok_or_else(|| "the members key is followed by no list".to_string())?;
    let inside = rest
        .split_once(']')
        .map(|(inside, _)| inside)
        .ok_or_else(|| "the members list is not closed".to_string())?;

    let mut members = Vec::new();
    let mut characters = inside.chars();
    while let Some(character) = characters.next() {
        if character != '"' {
            continue;
        }
        let member: String = characters
            .by_ref()
            .take_while(|next| *next != '"')
            .collect();
        members.push(member);
    }
    if members.is_empty() {
        return Err("the members list names nothing".to_string());
    }
    Ok(members)
}

/// Whether a member manifest takes the workspace lint set.
///
/// True only for a `[lints]` table carrying `workspace = true`. A crate without
/// it is linted at the default level, which for the security set in the root
/// manifest means `unsafe_code` is allowed in it: `forbid` cannot refuse what it
/// was never applied to.
///
/// `[lints.rust]` and `[lints.clippy]` are the crate declaring its own set and
/// are not this, so a table header longer than `[lints]` ends the search rather
/// than continuing it.
pub fn takes_workspace_lints(manifest: &str) -> bool {
    let mut inside = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[lints]";
            continue;
        }
        if !inside {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "workspace" {
            let value = value.split('#').next().unwrap_or(value);
            return value.trim() == "true";
        }
    }
    false
}

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
