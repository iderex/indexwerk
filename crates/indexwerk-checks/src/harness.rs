//! The declared legs of the hardware-bound harness, read from where they are
//! declared (#18).
//!
//! `harness/` is not a member of this workspace, so nothing in the gate builds
//! it and nothing in the gate runs it. That is the point of the separation and
//! it has a cost: a green run here says nothing about those legs, and a reader
//! could take it as saying something.
//!
//! So the table in `harness/README.md` is parsed rather than admired. It is the
//! authority for which legs exist and what each one needs, this module is the
//! only reader of it inside the workspace, and the suite uses what it returns
//! to report every leg as one that did not run.

use std::fs;

use crate::workspace_root;

/// The four kinds of requirement #18 fixes. A fifth is a change here as well as
/// a change to the table, which is what stops the column drifting into prose.
pub const KINDS: &[&str] = &[
    "core count",
    "processor feature",
    "memory",
    "external licence",
];

/// Where the table lives, relative to the workspace root.
pub const DECLARATION: &str = "harness/README.md";

/// The header row the parser anchors on.
const HEADER: &str = "| Leg | Kind of requirement | What it requires | How to run it |";

/// The command every row's last column has to start with, so that a row cannot
/// declare a leg run by something the harness does not own.
const COMMAND_PREFIX: &str = "cargo run --manifest-path harness/Cargo.toml --";

/// One declared leg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leg {
    pub id: String,
    pub kind: String,
    pub requires: String,
    pub command: String,
}

/// Why a declaration was not accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Malformed {
    Unreadable(String),
    NoTable,
    NoLegs,
    Row { row: usize, why: String },
}

impl Malformed {
    pub fn message(&self) -> String {
        match self {
            Malformed::Unreadable(path) => {
                format!("{path} could not be read, so which legs exist is unknown")
            }
            Malformed::NoTable => format!(
                "{DECLARATION} carries no leg table. The header the parser anchors on is\n  \
                 {HEADER}"
            ),
            Malformed::NoLegs => format!(
                "{DECLARATION} carries a leg table with no rows, which would report a harness \
                 with no legs as covered"
            ),
            Malformed::Row { row, why } => {
                format!("{DECLARATION}, leg row {row}: {why}")
            }
        }
    }
}

/// Split a markdown table row into its cells, dropping the empty pieces the
/// leading and trailing pipes produce.
fn cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed
        .strip_prefix('|')
        .and_then(|rest| rest.strip_suffix('|'))
        .unwrap_or(trimmed);
    inner
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect()
}

/// Parse the leg table out of the declaration's text.
///
/// Split from the file reading so a fixture can be fed to it without existing
/// on disk, which is how the refusals below are proved.
pub fn legs_from(text: &str) -> Result<Vec<Leg>, Malformed> {
    let lines: Vec<&str> = text.lines().collect();
    let Some(header) = lines.iter().position(|line| line.trim() == HEADER) else {
        return Err(Malformed::NoTable);
    };

    let mut legs = Vec::new();
    let mut row = 0usize;
    for line in lines.iter().skip(header + 1) {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            break;
        }
        // The separator under the header is not a leg.
        if trimmed.replace(['|', '-', ' '], "").is_empty() {
            continue;
        }
        row += 1;
        let cells = cells(trimmed);
        if cells.len() != 4 {
            return Err(Malformed::Row {
                row,
                why: format!("{} cells rather than four", cells.len()),
            });
        }
        let id = cells[0].trim_matches('`').trim().to_owned();
        let kind = cells[1].clone();
        let requires = cells[2].clone();
        let command = cells[3].trim_matches('`').trim().to_owned();

        if id.is_empty() {
            return Err(Malformed::Row {
                row,
                why: "the leg has no identifier".to_owned(),
            });
        }
        if !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        {
            return Err(Malformed::Row {
                row,
                why: format!(
                    "the identifier {id} is not lower case letters and hyphens, and it is what \
                     the run command and the report both spell"
                ),
            });
        }
        if !KINDS.contains(&kind.as_str()) {
            return Err(Malformed::Row {
                row,
                why: format!(
                    "{id} declares the requirement kind {kind:?}, which is not one of {KINDS:?}"
                ),
            });
        }
        if requires.is_empty() {
            return Err(Malformed::Row {
                row,
                why: format!(
                    "{id} says what kind of thing it needs and not what it needs, which is the \
                     half a reader deciding whether they can run it has to have"
                ),
            });
        }
        if !command.starts_with(COMMAND_PREFIX) {
            return Err(Malformed::Row {
                row,
                why: format!("{id} is run by something other than the harness: {command}"),
            });
        }
        if legs.iter().any(|earlier: &Leg| earlier.id == id) {
            return Err(Malformed::Row {
                row,
                why: format!("{id} is declared twice"),
            });
        }
        legs.push(Leg {
            id,
            kind,
            requires,
            command,
        });
    }

    if legs.is_empty() {
        return Err(Malformed::NoLegs);
    }
    Ok(legs)
}

/// The declared legs, read from the tree.
pub fn declared_legs() -> Result<Vec<Leg>, Malformed> {
    let path = workspace_root().join(DECLARATION);
    match fs::read_to_string(&path) {
        Ok(text) => legs_from(&text),
        Err(_) => Err(Malformed::Unreadable(DECLARATION.to_owned())),
    }
}

/// What the ordinary suite says about a leg it did not run.
///
/// One line per leg, naming the leg, what running it would require and the
/// command that runs it, so that a green run cannot be read as covering it.
pub fn did_not_run(leg: &Leg) -> String {
    format!(
        "not run here: {} requires {} ({}); run it with `{}`",
        leg.id, leg.requires, leg.kind, leg.command
    )
}
