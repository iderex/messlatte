//! The one binary at the top of the workspace.

use std::process::ExitCode;

fn main() -> ExitCode {
    // What an operator runs and what it prints is #88, and the run that reports
    // the toolchain versions is the second half of #1. Neither is here. One verb
    // is, because #42 asks for a command that prints the forward model, and a
    // model nobody can read without opening the source is the thing that issue
    // exists to prevent.
    match std::env::args().nth(1).as_deref() {
        None => {
            println!("messlatte {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("model") => {
            print!("{}", messlatte_generator::operator::PRINTED_FORM);
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!(
                "messlatte: there is no verb {other:?}. This binary carries one, model, which \
                 prints the forward model and what it leaves out. The rest of what an operator \
                 runs is #88 and is not built."
            );
            ExitCode::FAILURE
        }
    }
}
