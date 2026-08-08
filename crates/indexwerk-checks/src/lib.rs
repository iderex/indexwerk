#![forbid(unsafe_code)]

//! Checks over this tree that the compiler cannot make.
//!
//! This crate ships to nobody. It is repository tooling, not one of the three
//! layers `docs/adr/0005-layering.md` fixes, and that record says so.
//!
//! What is here is the greppable invariants of #41, one of which is the
//! headless and unelevated birth requirement of #17 that arrived first. Each
//! invariant names the decision record or the issue it comes from, so somebody
//! meeting a red check learns why rather than how to silence it. The list of
//! them is data, in `terms.rs`, and `docs/invariants.md` is rendered from that
//! data rather than written beside it.
//!
//! What this cannot do is stated where the bound is: a line scan reads text and
//! never a parse tree, so it judges spellings. A construct written in a
//! spelling no term names walks through, and widening the terms is the repair.
//!
//! [`harness`] is the other kind of check here. It reads nothing in this
//! workspace: it reads the declaration of the legs that deliberately do not run
//! in the gate, so that the suite reports what it did not cover rather than
//! leaving a green run to be read as covering everything.

pub mod harness;
pub mod terms;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use terms::{
    ABORT_IS_CORRECT_MARKER, COMPILE_TIME_REFUSAL, Class, EXCLUDED, INVARIANTS, Invariant,
    LOOPBACK_ESCAPES, Match, ROOTS_THAT_MUST_FORBID_UNSAFE, SOURCE_DISTANCE, SOURCE_WORDS, Scope,
    TERMS, TIME_UNITS, Term,
};

/// One construct found where it may not be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Path relative to the workspace root, with forward slashes.
    pub file: String,
    /// One-based, so it matches what an editor shows. Zero where the finding is
    /// about the file as a whole rather than about a line in it.
    pub line: usize,
    pub class: Class,
    pub needle: &'static str,
}

impl Finding {
    pub fn invariant(&self) -> Invariant {
        self.class.invariant()
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let invariant = self.invariant();
        write!(
            f,
            "{}:{}: {} ({}), refused by the invariant \"{}\", from {}",
            self.file,
            self.line,
            self.class.name(),
            self.needle,
            invariant.title(),
            invariant.source()
        )
    }
}

/// Whether a needle occurs in a line under the given matching rule.
fn line_matches(line: &str, needle: &str, matching: Match) -> bool {
    match matching {
        Match::Substring => line.contains(needle),
        Match::Token => {
            let bytes = line.as_bytes();
            let mut from = 0;
            while let Some(offset) = line[from..].find(needle) {
                let start = from + offset;
                let end = start + needle.len();
                let before_is_word = start > 0 && is_word_byte(bytes[start - 1]);
                let after_is_word = end < bytes.len() && is_word_byte(bytes[end]);
                if !before_is_word && !after_is_word {
                    return true;
                }
                from = start + 1;
            }
            false
        }
    }
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Whether this term may fire on this line at all, given what else the line and
/// the lines above it carry.
fn escaped(term: &Term, lines: &[&str], index: usize) -> bool {
    let line = lines[index];
    match term.class {
        // A bind to loopback is exactly what the rule permits, so refusing it
        // would refuse the compliant form along with the violating one.
        Class::OffLoopbackBind => LOOPBACK_ESCAPES.iter().any(|escape| line.contains(escape)),
        // The named list #41 allows: a place where a violated internal
        // invariant genuinely should abort, carrying a comment saying why.
        Class::PanicPath => {
            line.contains(ABORT_IS_CORRECT_MARKER) || admitted_by_the_comment_above(lines, index)
        }
        _ => false,
    }
}

/// Whether the contiguous run of comment lines directly above this one carries
/// the marker.
///
/// The run has to be contiguous and has to reach this line, so a marker written
/// once at the top of a file does not admit every panic below it. That is the
/// property this walk exists for: the admission sits on the statement it
/// admits, which is what makes the named list greppable.
fn admitted_by_the_comment_above(lines: &[&str], index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        let trimmed = lines[cursor].trim_start();
        if !trimmed.starts_with("//") {
            return false;
        }
        if trimmed.contains(ABORT_IS_CORRECT_MARKER) {
            return true;
        }
    }
    false
}

/// Scan one file's text. Split out from the walk so that a fixture can be fed
/// to it without existing on disk, and so that the path decides the scope in
/// exactly the way it does for a real file.
pub fn scan_text(relative_path: &str, text: &str) -> Vec<Finding> {
    let mut found = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let test_region = test_regions(&lines);

    for (index, line) in lines.iter().enumerate() {
        for term in TERMS {
            let scope = term.class.invariant().scope();
            if !scope.covers(relative_path) {
                continue;
            }
            // #[cfg(test)] regions are test code. A panic in a test is how a
            // test reports, so the one invariant scoped outside tests skips
            // them, and every other invariant still reads them.
            if scope == Scope::LibrarySourcesOutsideTests && test_region[index] {
                continue;
            }
            if !line_matches(line, term.needle, term.matching) {
                continue;
            }
            if escaped(term, &lines, index) {
                continue;
            }
            found.push(Finding {
                file: relative_path.to_owned(),
                line: index + 1,
                class: term.class,
                needle: term.needle,
            });
        }
    }

    if Invariant::NoPerformanceNumberWithoutItsSource
        .scope()
        .covers(relative_path)
    {
        found.extend(unsourced_performance_numbers(relative_path, &lines));
    }

    found.sort_by(|a, b| a.line.cmp(&b.line));
    found
}

/// Which lines sit inside a `#[cfg(test)]` item.
///
/// Brace counting rather than a parse. It starts at the attribute, waits for
/// the first line carrying an opening brace, and ends when the depth returns to
/// where it started. A brace inside a string literal or a comment would fool
/// it, which is the bound: it can only ever mark too much or too little of a
/// test module, and both directions are visible in the fixtures below.
fn test_regions(lines: &[&str]) -> Vec<bool> {
    let mut inside = vec![false; lines.len()];
    let mut index = 0;
    while index < lines.len() {
        if !lines[index].contains("#[cfg(test)]") {
            index += 1;
            continue;
        }
        let start = index;
        let mut depth: i32 = 0;
        let mut opened = false;
        let mut cursor = index;
        while cursor < lines.len() {
            for byte in lines[cursor].bytes() {
                if byte == b'{' {
                    depth += 1;
                    opened = true;
                } else if byte == b'}' {
                    depth -= 1;
                }
            }
            cursor += 1;
            if opened && depth <= 0 {
                break;
            }
        }
        for entry in inside.iter_mut().take(cursor).skip(start) {
            *entry = true;
        }
        index = cursor.max(start + 1);
    }
    inside
}

/// A time figure with neither a command block nor a source label near it.
fn unsourced_performance_numbers(relative_path: &str, lines: &[&str]) -> Vec<Finding> {
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !carries_time_figure(line) {
            continue;
        }
        if source_is_near(lines, index) {
            continue;
        }
        found.push(Finding {
            file: relative_path.to_owned(),
            line: index + 1,
            class: Class::UnsourcedPerformanceNumber,
            needle: "a time figure",
        });
    }
    found
}

/// A run of digits followed by an optional single space and a time unit that is
/// not the start of a longer word.
fn carries_time_figure(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let mut end = index;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        let mut after = end;
        if after < bytes.len() && bytes[after] == b' ' {
            after += 1;
        }
        for unit in TIME_UNITS {
            let stop = after + unit.len();
            if stop <= bytes.len()
                && &line[after..stop] == *unit
                && (stop == bytes.len() || !is_word_byte(bytes[stop]))
            {
                return true;
            }
        }
        index = end;
    }
    false
}

fn source_is_near(lines: &[&str], index: usize) -> bool {
    let first = index.saturating_sub(SOURCE_DISTANCE);
    let last = (index + SOURCE_DISTANCE).min(lines.len().saturating_sub(1));
    for line in &lines[first..=last] {
        if is_command_block(line) {
            return true;
        }
        if SOURCE_WORDS.iter().any(|word| line.contains(word)) {
            return true;
        }
    }
    false
}

/// An indented block or a fence. This tree writes commands as four-space
/// indented blocks, and a fence is accepted so that the rule does not depend on
/// which of the two a writer reached for.
fn is_command_block(line: &str) -> bool {
    line.trim_start().starts_with("```") || (line.starts_with("    ") && !line.trim().is_empty())
}

/// The workspace root, derived from this crate's own location at compile time
/// rather than from the working directory, so the result does not depend on
/// where the test runner was started.
pub fn workspace_root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent);
    // aborts on a violated internal invariant: this crate's manifest directory
    // is fixed at compile time and sits two levels below the workspace root, so
    // a None here means the tree was restructured without this function, and
    // every path derived from it would be wrong rather than absent. It is the
    // one admitted abort in this crate, and this comment is what admits it.
    root.expect("this crate sits two levels below the workspace root")
        .to_path_buf()
}

/// Scan the whole tree: every Rust source under `crates/`, every markdown file
/// the documentation scope covers, and the crate roots that must carry the
/// compile-time refusal of unsafe code.
pub fn scan_workspace() -> Vec<Finding> {
    let root = workspace_root();
    let mut findings = Vec::new();

    let mut files = Vec::new();
    collect_files(&root.join("crates"), &root, "rs", &mut files);
    collect_files(&root.join("docs"), &root, "md", &mut files);
    let readme = root.join("README.md");
    if readme.is_file() {
        files.push(("README.md".to_owned(), readme));
    }
    files.sort();

    for (relative, absolute) in files {
        if EXCLUDED.contains(&relative.as_str()) {
            continue;
        }
        let text = match fs::read_to_string(&absolute) {
            Ok(text) => text,
            // A file that cannot be read is not a file that was found clean.
            Err(_) => {
                findings.push(Finding {
                    file: relative,
                    line: 0,
                    class: Class::UnreadableFile,
                    needle: "unreadable",
                });
                continue;
            }
        };
        findings.extend(scan_text(&relative, &text));
    }

    findings.extend(crate_roots_missing_the_refusal(&root));
    findings.sort_by(|a, b| (a.file.clone(), a.line).cmp(&(b.file.clone(), b.line)));
    findings
}

/// The half of the unsafe invariant that catches deleting the attribute rather
/// than writing an unsafe block.
///
/// `docs/adr/0005-layering.md` puts every unsafe line in one crate, and the
/// compiler holds that per crate through `#![forbid(unsafe_code)]`. Deleting
/// that line removes the refusal and the compiler says nothing, which is the
/// gap #7 names and this reads.
fn crate_roots_missing_the_refusal(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    for relative in ROOTS_THAT_MUST_FORBID_UNSAFE {
        let absolute = root.join(relative);
        let carries = fs::read_to_string(&absolute)
            .map(|text| text.lines().any(|line| line.trim() == COMPILE_TIME_REFUSAL))
            .unwrap_or(false);
        if !carries {
            findings.push(Finding {
                file: (*relative).to_owned(),
                line: 0,
                class: Class::MissingCompileTimeRefusal,
                needle: COMPILE_TIME_REFUSAL,
            });
        }
    }
    findings
}

fn collect_files(directory: &Path, root: &Path, extension: &str, out: &mut Vec<(String, PathBuf)>) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // `target` is build output, not source.
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            collect_files(&path, root, extension, out);
        } else if path.extension().is_some_and(|found| found == extension) {
            if let Ok(relative) = path.strip_prefix(root) {
                let relative = relative.to_string_lossy().replace('\\', "/");
                out.push((relative, path.clone()));
            }
        }
    }
}

/// The invariant list, rendered.
///
/// `docs/invariants.md` is this string and nothing else, asserted by a test, so
/// adding an invariant to the table in `terms.rs` without regenerating the file
/// reds the check. That is what keeps the document from drifting against the
/// thing it describes.
pub fn catalogue_markdown() -> String {
    let mut out = String::new();
    out.push_str("# The greppable invariants of this tree\n\n");
    out.push_str(
        "Generated from the table in `crates/indexwerk-checks/src/terms.rs`. A test asserts\n\
         that this file is exactly what that table renders, so an invariant added to the\n\
         check and not to this file reds the gate, and an entry written here that the check\n\
         does not hold reds it too. Do not edit this file by hand.\n\n",
    );
    out.push_str(
        "The mechanism is taken from the merge gate this board takes as its quality target,\n\
         [Flowfin/jellyfin-plugin-sso](https://github.com/Flowfin/jellyfin-plugin-sso). The\n\
         invariants are not, because an invariant worth enforcing is a property of this code\n\
         rather than of that one. Issue #41 is where the set was argued.\n\n",
    );
    out.push_str("| Invariant | Where it comes from | What it reads |\n");
    out.push_str("| --- | --- | --- |\n");
    for invariant in INVARIANTS {
        out.push_str(&format!(
            "| {} | {} | `{}` |\n",
            invariant.title(),
            invariant.source(),
            invariant.scope_description()
        ));
    }
    out.push_str(
        "\n## What a finding says\n\n\
         A finding names the file, the line, the construct, the invariant it broke and where\n\
         that invariant comes from. The source is the point: somebody meeting a red check is\n\
         being told why the rule exists, not how to silence it.\n\n\
         ## What this cannot do\n\n\
         It is a line scan. It reads text and never a parse tree, so it judges spellings, and\n\
         a construct written in a spelling no term names walks through it. Widening the terms\n\
         is a change to the check that shows up in a diff, which is the repair.\n\n\
         The performance-number invariant is the crudest of the six and deliberately so. It\n\
         reads a digit run followed by an ASCII time unit, and it accepts a command block or\n\
         a word saying the figure is somebody else's within ten lines. A figure spelled out\n\
         in words is not seen, and a command ten lines away that produced something else is\n\
         accepted. A crude check that fires on a real defect is worth more than an exact one\n\
         nobody writes.\n",
    );
    out
}
