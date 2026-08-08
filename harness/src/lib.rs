#![forbid(unsafe_code)]

//! The hardware-bound harness (#18).
//!
//! What is here is not a measurement. It is the part that decides whether a
//! measurement may be printed, and the answer is no unless the machine it came
//! from and the versions it compared are both carried with it.
//!
//! The legs themselves are declared in `README.md` next to this file, one row
//! per leg, saying what each one requires before it says what it measures.
//! Nothing here produces a result yet, because the engine those legs would
//! measure does not exist. [`dispatch`] says so and exits non-zero rather than
//! printing an empty table that could be mistaken for a clean run.

use std::fmt;

/// The declared legs and what each one needs, as written.
///
/// Included at compile time rather than read from disk, so the binary says the
/// same thing wherever it was started from and a missing file is a build
/// failure rather than an empty list at run time.
pub const LEGS_AS_DECLARED: &str = include_str!("../README.md");

/// The machine a number came from.
///
/// One free-text field rather than a set of parsed ones. What matters about a
/// machine changes per leg: a scaling number needs the core count, a
/// feature-gated path needs the feature, a large-memory case needs the memory
/// size. A schema fixed now would be wrong for the leg written next, and the
/// property this crate enforces is that the description is present and not
/// blank, which holds whatever it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
    pub description: String,
}

/// One of the things a leg compared, and the version of it that ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compared {
    pub name: String,
    pub version: String,
}

/// What a leg produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    /// The leg identifier, as the table in `README.md` spells it.
    pub leg: String,
    /// The number and its unit, as the leg wrote it.
    pub value: String,
    /// Everything the leg compared, each with its version. Empty is refused:
    /// a comparison whose versions are unrecorded cannot be repeated.
    pub compared: Vec<Compared>,
}

/// Why a measurement was not printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    NoLeg,
    NoValue,
    NoMachineDescription,
    NothingCompared,
}

impl Refused {
    pub fn reason(self) -> &'static str {
        match self {
            Refused::NoLeg => "the measurement names no leg",
            Refused::NoValue => "the measurement carries no value",
            Refused::NoMachineDescription => {
                "the machine description is blank, and a number with no machine is not a result"
            }
            Refused::NothingCompared => {
                "nothing is recorded as compared, so no version of anything is carried"
            }
        }
    }
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "refused: {}", self.reason())
    }
}

/// Render a measurement, or refuse it.
///
/// This is the only way this crate emits a number. There is no second path and
/// no flag that skips the checks, because a check with a way round it is a
/// check that will be gone round on the night somebody wants a figure quickly.
pub fn render(machine: &Machine, measurement: &Measurement) -> Result<String, Refused> {
    if measurement.leg.trim().is_empty() {
        return Err(Refused::NoLeg);
    }
    if measurement.value.trim().is_empty() {
        return Err(Refused::NoValue);
    }
    if machine.description.trim().is_empty() {
        return Err(Refused::NoMachineDescription);
    }
    if measurement.compared.is_empty() {
        return Err(Refused::NothingCompared);
    }

    let mut out = String::new();
    out.push_str(&format!("leg: {}\n", measurement.leg.trim()));
    out.push_str(&format!("machine: {}\n", machine.description.trim()));
    out.push_str("compared:\n");
    for entry in &measurement.compared {
        out.push_str(&format!(
            "  {} {}\n",
            entry.name.trim(),
            entry.version.trim()
        ));
    }
    out.push_str(&format!("result: {}\n", measurement.value.trim()));
    Ok(out)
}

/// Whether the table in `README.md` declares this leg.
///
/// A row is spelled with the identifier in backticks in the first column, so
/// that is what is looked for. A prose mention of the same word elsewhere in
/// the file is not a declaration and does not match.
pub fn is_declared(leg: &str) -> bool {
    LEGS_AS_DECLARED
        .lines()
        .any(|line| line.trim_start().starts_with(&format!("| `{leg}` |")))
}

/// What the binary decided to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Print this on standard output and exit zero.
    Print(String),
    /// Print this on standard error and exit with this code.
    Refuse(String, u8),
}

/// The whole of the binary's decision, separated from the binary so that it is
/// testable without starting a process.
pub fn dispatch(arguments: &[String]) -> Outcome {
    match arguments.split_first() {
        None => Outcome::Print(LEGS_AS_DECLARED.to_owned()),
        Some((verb, rest)) if verb == "list" && rest.is_empty() => {
            Outcome::Print(LEGS_AS_DECLARED.to_owned())
        }
        Some((verb, rest)) if verb == "run" => match rest {
            [leg] if is_declared(leg) => Outcome::Refuse(
                format!(
                    "{leg} is declared and has no measurement to make. The engine it would \
                     measure does not exist in this tree yet, so there is nothing to run and \
                     nothing to report. See harness/README.md for what this leg will require \
                     when it does."
                ),
                1,
            ),
            [leg] => Outcome::Refuse(
                format!(
                    "{leg} is not a declared leg. The table in harness/README.md is the list, \
                     and adding a leg means adding a row there."
                ),
                2,
            ),
            _ => Outcome::Refuse("run takes exactly one leg identifier".to_owned(), 2),
        },
        Some((verb, _)) => Outcome::Refuse(
            format!("{verb} is not a verb this harness has. The verbs are list and run."),
            2,
        ),
    }
}
