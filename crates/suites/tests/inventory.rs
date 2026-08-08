//! What the default run says about the suites it did and did not run.
//!
//! This target carries no test harness, declared in this crate's manifest. The
//! reason is that the harness captures what a passing case prints, and a
//! statement that only appears when something fails is not a statement the run
//! makes. Without the harness the target is an ordinary program, so `cargo
//! test` shows its output whether the run is green or red.
//!
//! It is a case as well as a printer: a suite variable holding something nobody
//! can read is a run whose coverage cannot be stated, and this exits non-zero
//! rather than printing a report it does not believe.

fn main() {
    match messlatte_suites::suite::report_from_env() {
        Ok(text) => print!("{text}"),
        Err(reason) => {
            eprintln!("the suites this run covered cannot be stated: {reason}");
            std::process::exit(1);
        }
    }
}
