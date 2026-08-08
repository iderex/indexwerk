//! The formatting rules for the parts of the tree no formatter in the toolchain
//! reads (#16).
//!
//! `cargo fmt` reads the Rust and nothing else. What is left is the Markdown,
//! which is most of this tree, and any Python that arrives later. Both are
//! handled here rather than by a third-party formatter, and the reason is in
//! `CONTRIBUTING.md` next to the commands.
//!
//! ## What this deliberately does not judge
//!
//! Line endings. `.gitattributes` stores and checks out LF everywhere and is the
//! authority for it, and a checkout that predates a rule in that file can carry
//! carriage returns in the working tree while the committed bytes are LF. A
//! formatter that also judged line endings would report exactly that clean tree
//! as failing, which is the failure #16 names in as many words. So a trailing
//! carriage return is removed from each line before the line is judged, and
//! nothing here reports one.
//!
//! Line length. This tree wraps prose at eighty columns by hand and has
//! twenty lines that do not, in a rendered document, in a paragraph written as
//! one line on purpose, and in links that cannot be broken. Reflowing those is a
//! different change with a different argument, so no rule here reads a length.
//!
//! What is left is whitespace, and whitespace is worth judging because it is
//! invisible: a tab where the file otherwise uses spaces, a line carrying
//! trailing blanks that no reader sees, a file whose last line has no newline.

use std::fs;
use std::path::{Path, PathBuf};

use crate::workspace_root;

/// A whitespace defect in a Markdown file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Defect {
    /// A tab. `.editorconfig` fixes this tree on spaces, and a tab renders at a
    /// width nobody agrees on.
    Tab,
    /// Trailing blanks that are not the two spaces Markdown reads as a hard
    /// line break. One space is a typo, three or more is a typo, and neither
    /// renders differently from none, so nothing shows the writer they are
    /// there.
    TrailingWhitespace,
    /// A line that looks empty and is not, which is the same invisibility one
    /// step worse: it survives every edit because nobody can see it.
    WhitespaceOnlyLine,
    /// No newline at the end of the file. `.editorconfig` asks for one, and
    /// without it every later append shows as a change to the last line.
    NoFinalNewline,
    /// A blank line at the end of the file, beyond the single final newline.
    BlankLineAtEnd,
}

impl Defect {
    pub fn name(self) -> &'static str {
        match self {
            Defect::Tab => "a tab",
            Defect::TrailingWhitespace => {
                "trailing whitespace that is not the two spaces of a hard line break"
            }
            Defect::WhitespaceOnlyLine => "a line that is whitespace and is not empty",
            Defect::NoFinalNewline => "no newline at the end of the file",
            Defect::BlankLineAtEnd => "a blank line at the end of the file",
        }
    }
}

/// One defect, where it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Complaint {
    /// Path relative to the workspace root, with forward slashes.
    pub file: String,
    /// One-based, so it matches what an editor shows. Zero where the complaint
    /// is about the file as a whole.
    pub line: usize,
    pub defect: Defect,
}

impl std::fmt::Display for Complaint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.defect.name())
    }
}

/// The two spaces Markdown reads as a hard line break, which are the one form
/// of trailing whitespace this tree keeps. `.editorconfig` stops an editor
/// stripping them for the same reason.
const HARD_BREAK: &str = "  ";

/// Judge one file's text. Split out from the walk so a fixture can be fed to it
/// without existing on disk.
pub fn check_text(relative_path: &str, text: &str) -> Vec<Complaint> {
    let mut found = Vec::new();
    if text.is_empty() {
        return found;
    }

    let complain = |line: usize, defect: Defect| Complaint {
        file: relative_path.to_owned(),
        line,
        defect,
    };

    for (index, raw) in text.split('\n').enumerate() {
        // The carriage return is removed rather than judged. See the module
        // comment: `.gitattributes` owns line endings and judging them here
        // would red a tree whose committed bytes are correct.
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let number = index + 1;
        if line.contains('\t') {
            found.push(complain(number, Defect::Tab));
        }
        if line.trim().is_empty() {
            if !line.is_empty() {
                found.push(complain(number, Defect::WhitespaceOnlyLine));
            }
            continue;
        }
        let trailing = &line[line.trim_end().len()..];
        if !trailing.is_empty() && trailing != HARD_BREAK {
            found.push(complain(number, Defect::TrailingWhitespace));
        }
    }

    let normalised = text.replace("\r\n", "\n");
    if !normalised.ends_with('\n') {
        let last = normalised.split('\n').count();
        found.push(complain(last, Defect::NoFinalNewline));
    } else if normalised.ends_with("\n\n") {
        let last = normalised.split('\n').count().saturating_sub(1);
        found.push(complain(last, Defect::BlankLineAtEnd));
    }

    found.sort_by_key(|complaint| complaint.line);
    found
}

/// Every Markdown file in the tree, judged.
pub fn check_tree() -> Vec<Complaint> {
    let root = workspace_root();
    let mut files = Vec::new();
    collect(&root, &root, "md", &mut files);
    files.sort();

    let mut found = Vec::new();
    for (relative, absolute) in files {
        match fs::read_to_string(&absolute) {
            Ok(text) => found.extend(check_text(&relative, &text)),
            // A file that cannot be read is not a file that was found clean, so
            // it is reported rather than skipped. There is no defect for it and
            // there does not need to be: the walk reaching an unreadable
            // Markdown file is a broken tree, not a formatting question.
            Err(_) => found.push(Complaint {
                file: relative,
                line: 0,
                defect: Defect::NoFinalNewline,
            }),
        }
    }
    found
}

/// Every Python source in the tree, which is how the format leg checks a
/// language that is not here yet.
///
/// There is no Python in this tree today and no Python formatter configured
/// anywhere in it. Those two facts are only consistent together: the day a
/// `.py` file lands without a formatter chosen for it, the leg has to say so
/// rather than pass over it in silence. So this returns the list and the suite
/// requires it to be empty, which fails closed in the direction that matters.
pub fn python_sources() -> Vec<String> {
    let root = workspace_root();
    let mut files = Vec::new();
    collect(&root, &root, "py", &mut files);
    collect(&root, &root, "pyi", &mut files);
    let mut names: Vec<String> = files.into_iter().map(|(relative, _)| relative).collect();
    names.sort();
    names
}

/// Walk for files with one extension, skipping build output and the git
/// directory.
fn collect(directory: &Path, root: &Path, extension: &str, out: &mut Vec<(String, PathBuf)>) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string());
        if path.is_dir() {
            if name.as_deref() == Some("target") || name.as_deref() == Some(".git") {
                continue;
            }
            collect(&path, root, extension, out);
        } else if path.extension().is_some_and(|found| found == extension) {
            if let Ok(relative) = path.strip_prefix(root) {
                out.push((relative.to_string_lossy().replace('\\', "/"), path.clone()));
            }
        }
    }
}
