#![forbid(unsafe_code)]

//! The harness entry point.
//!
//! It does nothing but hand the arguments to [`indexwerk_harness::dispatch`]
//! and act on what comes back, so that every decision this program makes is
//! made in a function the suite can call without starting a process.

use std::process::ExitCode;

use indexwerk_harness::{Outcome, dispatch};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&arguments) {
        Outcome::Print(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Outcome::Refuse(message, code) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}
