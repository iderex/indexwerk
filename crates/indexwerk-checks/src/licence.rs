//! The licence of this repository, read from the manifests that declare it and
//! from the file they name (#47).
//!
//! `LICENSE` is the terms. An identifier in a manifest is what a package index,
//! a bill of materials generator and a dependency policy read instead of the
//! terms, and the two are separate strings that can disagree. A manifest saying
//! one thing while the file says another is the discrepancy that surfaces at the
//! worst moment, which is the release, so it is refused here rather than left to
//! whoever reads both.
//!
//! Three failures are refused. A manifest that declares no licence at all, which
//! is the state a crate added later arrives in unless something asks. A manifest
//! that declares a different identifier from the rest of the tree. And a
//! `LICENSE` file that is not the text the identifier names, which is what a
//! relicensing half done looks like from here.
//!
//! What this cannot do. It reads a manifest as lines rather than as parsed TOML,
//! because this crate has no dependencies and a TOML parser would be one, so it
//! judges the spelling of a key at the start of a line. A key written inside a
//! multi-line string, or a table this reader cannot tell from another, is
//! outside what it sees. Cargo itself refuses a malformed manifest before this
//! check runs, which is what makes the crude reader affordable.

use std::fs;
use std::path::{Path, PathBuf};

use crate::workspace_root;

/// The identifier every manifest in this tree declares.
///
/// The strict spelling rather than `AGPL-3.0-or-later`, because that is what
/// `LICENSE` and the readme say between them: the readme states version 3 and
/// names no later one, and the `or (at your option) any later version` line in
/// the appendix of `LICENSE` is the notice that document tells a program to
/// attach to its own sources, which nothing in this tree does yet. Widening it
/// is a decision rather than a reading, it belongs to #2 with the answer that
/// chose the licence, and it is one constant here plus the manifests this check
/// then reds until they follow.
pub const IDENTIFIER: &str = "AGPL-3.0-only";

/// The file at the root carrying the terms the identifier names.
pub const LICENCE_FILE: &str = "LICENSE";

/// Two lines the text of that file has to carry. The title alone would accept
/// version 2 of the same licence, which is a different set of terms under a
/// name that reads the same at a glance.
pub const LICENCE_TITLE: &str = "GNU AFFERO GENERAL PUBLIC LICENSE";
/// The version line, with the date the text carries.
pub const LICENCE_VERSION: &str = "Version 3, 19 November 2007";

/// The manifest the members inherit from, relative to the workspace root.
pub const ROOT_MANIFEST: &str = "Cargo.toml";

/// What one manifest says about the licence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    /// `license = "<identifier>"`.
    Literal(String),
    /// `license.workspace = true`, in either of the two spellings cargo takes.
    Inherited,
    /// `license-file = "<path>"`. A path is not an identifier: it names bytes
    /// rather than terms, and nothing that reads an identifier reads it.
    APath(String),
    /// Nothing at all.
    Absent,
}

/// One manifest, or the licence file, saying something the rest of the tree does
/// not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wrong {
    /// The identifier names a file that is not there.
    NoLicenceFile,
    /// The file is there and it is not the text the identifier names.
    LicenceFileIsAnotherText {
        /// The first line of the file that carries anything, so the message says
        /// what was found rather than only what was wanted.
        opening: String,
    },
    /// A manifest the walk reached and could not read. A manifest that cannot be
    /// read is not a manifest that was found to declare the licence.
    Unreadable { manifest: String },
    /// A manifest that declares no licence.
    Undeclared { manifest: String },
    /// A manifest that declares another identifier.
    Another { manifest: String, declared: String },
    /// A manifest that points at a file where an identifier belongs.
    APathRatherThanAnIdentifier { manifest: String, path: String },
    /// A manifest inheriting the key from a workspace that declares none, which
    /// cargo refuses on the member and which reads here as a declaration.
    InheritedFromNothing { manifest: String },
}

impl Wrong {
    pub fn message(&self) -> String {
        match self {
            Wrong::NoLicenceFile => format!(
                "every manifest declares {IDENTIFIER} and {LICENCE_FILE} is not in the tree, so \
                 the identifier names terms nobody can read"
            ),
            Wrong::LicenceFileIsAnotherText { opening } => format!(
                "{LICENCE_FILE} is not the text {IDENTIFIER} names: it carries neither \
                 \"{LICENCE_TITLE}\" nor \"{LICENCE_VERSION}\", and it opens with {opening:?}"
            ),
            Wrong::Unreadable { manifest } => {
                format!("{manifest} could not be read, so what it declares is unknown")
            }
            Wrong::Undeclared { manifest } => format!(
                "{manifest} declares no licence. Every manifest in this tree declares \
                 {IDENTIFIER}, literally or with `license.workspace = true`"
            ),
            Wrong::Another { manifest, declared } => format!(
                "{manifest} declares {declared:?} where the rest of the tree declares \
                 {IDENTIFIER:?}"
            ),
            Wrong::APathRatherThanAnIdentifier { manifest, path } => format!(
                "{manifest} declares `license-file = {path:?}` where an identifier belongs. A \
                 path names bytes rather than terms, and a package index reads the identifier"
            ),
            Wrong::InheritedFromNothing { manifest } => format!(
                "{manifest} inherits the licence from a workspace that declares none, so it \
                 declares nothing"
            ),
        }
    }
}

/// The value of a quoted key on a line, given the key and the line's text after
/// it. Returns nothing where the value is not a quoted string.
fn quoted_value(after_key: &str) -> Option<String> {
    let rest = after_key.trim_start().strip_prefix('=')?;
    let value = rest.trim_start().strip_prefix('"')?;
    value.split('"').next().map(str::to_owned)
}

/// What a manifest's text declares.
///
/// Split from the file reading so a fixture can be judged without existing on
/// disk, which is how the refusals above are proved.
pub fn declaration_in(text: &str) -> Declaration {
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(after) = trimmed.strip_prefix("license-file") {
            if let Some(path) = quoted_value(after) {
                return Declaration::APath(path);
            }
            continue;
        }
        if let Some(after) = trimmed.strip_prefix("license.workspace") {
            if after.trim_start().starts_with('=') && after.contains("true") {
                return Declaration::Inherited;
            }
            continue;
        }
        if let Some(after) = trimmed.strip_prefix("license") {
            if let Some(identifier) = quoted_value(after) {
                return Declaration::Literal(identifier);
            }
            // The other spelling of the same thing, `license = { workspace = true }`.
            if after.trim_start().starts_with('=')
                && after.contains("workspace")
                && after.contains("true")
            {
                return Declaration::Inherited;
            }
        }
    }
    Declaration::Absent
}

/// Whether a manifest opens a workspace of its own, and so inherits from no
/// other. `harness/Cargo.toml` is that case in this tree.
pub fn carries_its_own_workspace(text: &str) -> bool {
    text.lines().any(|line| line.trim() == "[workspace]")
}

/// The first line carrying anything, trimmed. Empty where there is none.
fn opening_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_owned()
}

/// Judge the manifests against each other and against the licence file.
///
/// A manifest is a relative path and its text, or nothing where it could not be
/// read. `licence` is the text of [`LICENCE_FILE`], or nothing where the file is
/// absent.
pub fn judge(manifests: &[(String, Option<String>)], licence: Option<&str>) -> Vec<Wrong> {
    let mut wrongs = Vec::new();

    match licence {
        None => wrongs.push(Wrong::NoLicenceFile),
        Some(text) => {
            if !text.contains(LICENCE_TITLE) || !text.contains(LICENCE_VERSION) {
                wrongs.push(Wrong::LicenceFileIsAnotherText {
                    opening: opening_line(text),
                });
            }
        }
    }

    let root_declares_it = manifests
        .iter()
        .find(|(path, _)| path == ROOT_MANIFEST)
        .and_then(|(_, text)| text.as_deref())
        .map(declaration_in)
        == Some(Declaration::Literal(IDENTIFIER.to_owned()));

    for (path, text) in manifests {
        let Some(text) = text else {
            wrongs.push(Wrong::Unreadable {
                manifest: path.clone(),
            });
            continue;
        };
        match declaration_in(text) {
            Declaration::Literal(identifier) if identifier == IDENTIFIER => {}
            Declaration::Literal(declared) => wrongs.push(Wrong::Another {
                manifest: path.clone(),
                declared,
            }),
            Declaration::APath(found) => wrongs.push(Wrong::APathRatherThanAnIdentifier {
                manifest: path.clone(),
                path: found,
            }),
            Declaration::Absent => wrongs.push(Wrong::Undeclared {
                manifest: path.clone(),
            }),
            Declaration::Inherited => {
                let inherits_from_the_root =
                    path != ROOT_MANIFEST && !carries_its_own_workspace(text);
                if !inherits_from_the_root || !root_declares_it {
                    wrongs.push(Wrong::InheritedFromNothing {
                        manifest: path.clone(),
                    });
                }
            }
        }
    }

    wrongs
}

/// Every `Cargo.toml` in the tree, as a relative path with forward slashes,
/// sorted.
///
/// The set is walked rather than listed, because a listed set is a set that
/// stops covering the crate somebody adds next, and that crate arriving without
/// a licence key is the failure this check is for.
pub fn manifest_paths(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    collect_manifests(root, root, &mut found);
    found.sort();
    found
}

fn collect_manifests(directory: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Build output and the repository's own metadata carry manifests
            // that nothing here ships.
            let skip = path
                .file_name()
                .is_some_and(|name| name == "target" || name == ".git");
            if !skip {
                collect_manifests(&path, root, out);
            }
        } else if path.file_name().is_some_and(|name| name == "Cargo.toml") {
            if let Ok(relative) = path.strip_prefix(root) {
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

/// Judge this tree.
pub fn judge_the_tree() -> Vec<Wrong> {
    let root = workspace_root();
    judge_below(&root)
}

/// Judge a tree rooted anywhere, so the walk itself can be exercised on a
/// fixture directory rather than only on this repository.
pub fn judge_below(root: &Path) -> Vec<Wrong> {
    let manifests: Vec<(String, Option<String>)> = manifest_paths(root)
        .into_iter()
        .map(|relative| {
            let absolute: PathBuf = root.join(&relative);
            (relative, fs::read_to_string(absolute).ok())
        })
        .collect();
    let licence = fs::read_to_string(root.join(LICENCE_FILE)).ok();
    judge(&manifests, licence.as_deref())
}
