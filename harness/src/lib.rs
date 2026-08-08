#![forbid(unsafe_code)]

//! The hardware-bound harness (#18).
//!
//! What is here is not a measurement. It is the part that decides whether a
//! measurement may be printed, and the answer is no unless everything a reader
//! needs in order to produce the number again is carried with it: the command,
//! the revision it was run at, the machine, the versions of everything
//! compared, how many times it ran and how far the runs spread. Those are the
//! rules #31 fixes, and refusing a result short of any of them is the half of
//! that issue a machine can hold.
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

/// The smallest number of runs this crate will print a central value from.
///
/// One run is a sample and not a measurement, which is #31's sentence rather
/// than a threshold chosen here.
pub const FEWEST_REPETITIONS: u32 = 2;

/// The shortest prefix of an object name accepted as a revision.
pub const SHORTEST_REVISION: usize = 7;

/// What a leg produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    /// The leg identifier, as the table in `README.md` spells it.
    pub leg: String,
    /// The number and its unit, as the leg wrote it.
    pub value: String,
    /// The command a reader runs to produce this number again. Blank is
    /// refused: a figure whose command is unrecorded cannot be re-run, and
    /// re-running it is the only thing that settles a disagreement about it.
    pub command: String,
    /// The revision the number was produced at, as an object name. A branch or
    /// a tag name is refused, because both move and the number does not move
    /// with them.
    pub revision: String,
    /// How many times the leg ran. Below [`FEWEST_REPETITIONS`] is refused.
    pub repetitions: u32,
    /// The spread across those runs, as the leg wrote it, next to the central
    /// value rather than instead of it. Blank is refused: a central value with
    /// no spread hides whether the runs agreed.
    pub spread: String,
    /// Everything the leg compared, each with its version. Empty is refused:
    /// a comparison whose versions are unrecorded cannot be repeated.
    pub compared: Vec<Compared>,
}

/// Why a measurement was not printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    NoLeg,
    NoValue,
    NoCommand,
    NoRevision,
    /// A revision that is not an object name, which is what a branch or a tag
    /// looks like from here.
    RevisionThatMoves,
    TooFewRepetitions,
    NoSpread,
    NoMachineDescription,
    NothingCompared,
}

impl Refused {
    pub fn reason(self) -> &'static str {
        match self {
            Refused::NoLeg => "the measurement names no leg",
            Refused::NoValue => "the measurement carries no value",
            Refused::NoCommand => {
                "no command is recorded, so the number cannot be produced again by anybody"
            }
            Refused::NoRevision => {
                "no revision is recorded, so what was measured is unknown even with the command"
            }
            Refused::RevisionThatMoves => {
                "the revision is not an object name. A branch or a tag moves and the number \
                 does not move with it, so the pair stops being true without either changing"
            }
            Refused::TooFewRepetitions => {
                "fewer runs than a spread can be taken across. A single run is not a measurement"
            }
            Refused::NoSpread => {
                "no spread is recorded, so whether the runs agreed with each other is unknown"
            }
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
    if measurement.command.trim().is_empty() {
        return Err(Refused::NoCommand);
    }
    let revision = measurement.revision.trim();
    if revision.is_empty() {
        return Err(Refused::NoRevision);
    }
    if !is_an_object_name(revision) {
        return Err(Refused::RevisionThatMoves);
    }
    if measurement.repetitions < FEWEST_REPETITIONS {
        return Err(Refused::TooFewRepetitions);
    }
    if measurement.spread.trim().is_empty() {
        return Err(Refused::NoSpread);
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
    out.push_str(&format!("command: {}\n", measurement.command.trim()));
    out.push_str(&format!("revision: {revision}\n"));
    out.push_str("compared:\n");
    for entry in &measurement.compared {
        out.push_str(&format!(
            "  {} {}\n",
            entry.name.trim(),
            entry.version.trim()
        ));
    }
    out.push_str(&format!(
        "runs: {}, spread {}\n",
        measurement.repetitions,
        measurement.spread.trim()
    ));
    out.push_str(&format!("result: {}\n", measurement.value.trim()));
    Ok(out)
}

/// Whether a string looks like a git object name rather than a name that moves.
///
/// Hexadecimal and long enough to be one. This cannot tell an object name that
/// exists from one that does not, and it is not meant to: what it separates is
/// `main` and `v0.1` from a revision, which is the mistake somebody makes when
/// the number is wanted quickly.
pub fn is_an_object_name(revision: &str) -> bool {
    revision.len() >= SHORTEST_REVISION && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
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
