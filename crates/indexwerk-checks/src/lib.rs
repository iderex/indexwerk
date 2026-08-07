#![forbid(unsafe_code)]

//! Checks over this tree that the compiler cannot make.
//!
//! This crate ships to nobody. It is repository tooling, not one of the three
//! layers `docs/adr/0005-layering.md` fixes, and that record says so.
//!
//! What is here today is the headless and unelevated birth requirement, #17.
//! Every test this project plans runs on a machine with no display attached and
//! under an ordinary user account, and a change that breaks that is a defect
//! rather than a step to document. The greppable invariants of #41 belong here
//! too when they arrive.

pub mod terms;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use terms::{Class, EXCLUDED, LOOPBACK_ESCAPES, TERMS};

/// One construct found where it may not be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Path relative to the workspace root, with forward slashes.
    pub file: String,
    /// One-based, so it matches what an editor shows.
    pub line: usize,
    pub class: Class,
    pub needle: &'static str,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} ({}), refused by the headless and unelevated birth requirement, issue #17",
            self.file,
            self.line,
            self.class.name(),
            self.needle
        )
    }
}

/// Scan one file's text. Split out from the walk so that a fixture can be fed
/// to it without existing on disk.
pub fn scan_text(relative_path: &str, text: &str) -> Vec<Finding> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for term in TERMS {
            if !line.contains(term.needle) {
                continue;
            }
            // A loopback bind is the compliant form of the same construct, so a
            // line naming loopback is not a finding.
            if term.class == Class::OffLoopbackBind
                && LOOPBACK_ESCAPES.iter().any(|escape| line.contains(escape))
            {
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
    found
}

/// The workspace root, derived from this crate's own location at compile time
/// rather than from the working directory, so the result does not depend on
/// where the test runner was started.
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("this crate sits two directories below the workspace root")
        .to_path_buf()
}

/// Scan every Rust source in the workspace.
///
/// The scan is wider than #17 asks for. #17 names the test sources; this reads
/// every `.rs` file under `crates/`, which is a superset, because a shipped
/// source that binds off loopback is a worse defect than a test that does and
/// there is no reason to look at only one of them.
pub fn scan_workspace() -> Vec<Finding> {
    let root = workspace_root();
    let mut findings = Vec::new();
    let mut files = Vec::new();
    collect_rust_sources(&root.join("crates"), &root, &mut files);
    files.sort();
    for (relative, absolute) in files {
        if EXCLUDED.contains(&relative.as_str()) {
            continue;
        }
        let text = match fs::read_to_string(&absolute) {
            Ok(text) => text,
            // A file that cannot be read is not a file that was found clean.
            Err(error) => {
                findings.push(Finding {
                    file: relative,
                    line: 0,
                    class: Class::Elevation,
                    needle: "unreadable",
                });
                let _ = error;
                continue;
            }
        };
        findings.extend(scan_text(&relative, &text));
    }
    findings
}

fn collect_rust_sources(directory: &Path, root: &Path, out: &mut Vec<(String, PathBuf)>) {
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
            collect_rust_sources(&path, root, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            if let Ok(relative) = path.strip_prefix(root) {
                let relative = relative.to_string_lossy().replace('\\', "/");
                out.push((relative, path.clone()));
            }
        }
    }
}
