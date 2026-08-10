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

/// Whether a member manifest takes the workspace license.
///
/// True only for `license.workspace = true` inside `[package]`. A member
/// spelling the identifier out itself is a second place the answer lives, and
/// the two copies come apart the first time one of them is edited, so it reads
/// as false here rather than as a different way of saying the same thing.
///
/// A table header ends the search, for the same reason it does in
/// [`takes_workspace_lints`]: a key of this name under some other table is a
/// different key.
pub fn takes_workspace_license(manifest: &str) -> bool {
    let mut inside = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[package]";
            continue;
        }
        if !inside {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "license.workspace" {
            let value = value.split('#').next().unwrap_or(value);
            return value.trim() == "true";
        }
    }
    false
}

/// The constant a module declares to say which of this project's formats it
/// reads and writes.
///
/// Joined at run time rather than written whole. This file is tracked under
/// `crates/` like every other, and the case using it searches every tracked
/// module for it, so a literal here would make this source the first thing that
/// search found and the guard would be judging its own text.
/// `crates/tree/tests/license_sentence.rs` splits a sentence for the same
/// reason. Nothing else below is a search pattern, so nothing else is split.
#[must_use]
pub fn format_module_marker() -> String {
    ["pub const ", "FORMAT"].concat()
}

/// Whether a module compares the major part of a version anywhere.
///
/// Both operators and both orders, because `a.major != b.major` and
/// `b.major == a.major` are the same decision written by two people and neither
/// is a mistake. What this refuses is the reader that compares the whole
/// version, which refuses a higher minor version as well.
///
/// A comparison spelled some third way reads as absent here. That is a refusal
/// of a correct module, which is a cost, and it is the direction this errs in on
/// purpose: the failure is loud, it names what to write, and the alternative
/// errs by passing a reader that judges the wrong thing.
fn compares_a_major_part(source: &str) -> bool {
    source.contains(".major != ") || source.contains(".major == ")
}

/// What a module is missing of the version rule (#41), sentence by sentence.
///
/// Every file this project writes carries a format version, a reader refuses a
/// major version it does not know rather than guessing, and a higher minor
/// version reads with the fields the reader does not recognise ignored. Three
/// modules apply that rule today and each carries its own copy, so nothing
/// connects them and a fourth format could land with the rule half applied while
/// every case in the tree stayed green.
///
/// The condition is two-sided, in the way `license_sentence.rs` is two-sided. A
/// module declaring no format name gets an empty answer, because it is not one
/// of this project's formats and this says nothing about it. What is refused is
/// the combination: a module declaring a format and then not carrying the rule.
///
/// It reads text rather than compiled code, and what that costs is worth stating
/// rather than leaving to be discovered. A marker written in a comment satisfies
/// it as readily as one written in code. A format naming its constant something
/// else is invisible to it. And it judges that a major part is compared
/// somewhere in the module rather than that the reader reaches that comparison,
/// so a module carrying the comparison in a branch nothing runs passes. What it
/// does catch is the part skipped outright, which is how a format arrives
/// without the rule.
#[must_use]
pub fn version_rule_gaps(source: &str) -> Vec<String> {
    if !source.contains(&format_module_marker()) {
        return Vec::new();
    }

    let mut gaps = Vec::new();
    if !source.contains("pub const VERSION") {
        gaps.push(
            "it declares a format and no `pub const VERSION`, so nothing says which version it \
             writes"
                .to_string(),
        );
    }
    if !source.contains("UnknownMajorVersion") {
        gaps.push(
            "it carries no `UnknownMajorVersion`, so a reader has no refusal for a major version \
             it does not know"
                .to_string(),
        );
    }
    if !compares_a_major_part(source) {
        gaps.push(
            "nothing in it compares a `.major` part, so what a reader refuses on is the whole \
             version, and the higher minor version the rule says has to read is refused with it"
                .to_string(),
        );
    }
    gaps
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
