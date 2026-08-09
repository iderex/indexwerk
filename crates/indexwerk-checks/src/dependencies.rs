//! The dependency policy, read from the manifests that declare dependencies and
//! from the file that gives each one its reason (#38).
//!
//! Two properties, over the same walk.
//!
//! Every direct dependency on something outside this tree carries a sentence in
//! `docs/dependencies.md` saying why it is there. A list with no reasons is a
//! list nobody can prune: three years later every entry looks load bearing and
//! nobody can tell the one that was needed once from the one that is needed now.
//! The register fails closed in both directions. A dependency with no entry is
//! refused, and an entry naming a dependency no manifest declares is refused
//! too, because a reasons file that keeps rows for departed crates stops being
//! read.
//!
//! No two locked packages of one name sit in different compatibility series.
//! That is how an audited version and an unaudited one end up in the same
//! binary, and it is invisible in a manifest because neither manifest asked for
//! it: it arrives through two dependencies that disagree about a third.
//!
//! Path dependencies inside this workspace are not entries. `indexwerk-ffi`
//! depending on `indexwerk-core` is the layering of
//! `docs/adr/0005-layering.md` rather than a supply chain entry, and asking it
//! for a reason would fill the register with rows about this repository.
//!
//! What this cannot do. It reads a manifest and a lock file as lines rather than
//! as parsed TOML, for the reason `licence` gives: this crate has no
//! dependencies and a TOML parser would be one. So it judges the spelling of a
//! table header and of a key at the start of a line, and a dependency written in
//! a shape this reader does not know walks through. Cargo refuses a malformed
//! manifest before this runs, which is what makes the crude reader affordable.
//! The lock file is generated rather than written, so its shape is the narrower
//! risk of the two.

use std::fs;
use std::path::Path;

use crate::workspace_root;

/// The file carrying one sentence per direct dependency, relative to the
/// workspace root.
pub const REASONS_FILE: &str = "docs/dependencies.md";

/// The lock file the duplicate check reads.
pub const LOCK_FILE: &str = "Cargo.lock";

/// One direct dependency, as one manifest declares it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Direct {
    /// Path of the manifest declaring it, relative to the workspace root.
    pub manifest: String,
    /// The crate name as the manifest spells it.
    pub name: String,
    /// Whether it is a path inside this tree rather than something fetched.
    pub inside_the_tree: bool,
}

/// One locked package.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Locked {
    pub name: String,
    pub version: String,
}

impl Locked {
    /// The series cargo treats as compatible: the leading component that is not
    /// zero, or the pair below it while that component is zero. `0.1` and `0.2`
    /// are as incompatible as `1` and `2`, and reporting them as one series
    /// would hide the case this check is for.
    pub fn series(&self) -> String {
        let mut parts = self.version.split('.');
        let major = parts.next().unwrap_or("0");
        if major != "0" {
            return major.to_owned();
        }
        let minor = parts.next().unwrap_or("0");
        format!("0.{minor}")
    }
}

/// A dependency, a register entry, or a lock file saying something the policy
/// refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wrong {
    /// The register is not in the tree at all.
    NoReasonsFile,
    /// A manifest the walk reached and could not read. A manifest that cannot be
    /// read is not a manifest that was found to declare nothing.
    UnreadableManifest { manifest: String },
    /// The lock file could not be read, so what is locked is unknown.
    UnreadableLockFile,
    /// A direct dependency with no sentence saying why it is there.
    Unaccounted { manifest: String, name: String },
    /// A sentence about a dependency no manifest declares.
    AccountedForNothing { name: String },
    /// Two locked packages of one name in different compatibility series.
    SeveralSeries { name: String, versions: Vec<String> },
}

impl Wrong {
    pub fn message(&self) -> String {
        match self {
            Wrong::NoReasonsFile => format!(
                "{REASONS_FILE} is not in this tree, so no direct dependency has a reason \
                 recorded and nothing can be pruned later"
            ),
            Wrong::UnreadableManifest { manifest } => {
                format!("{manifest} could not be read, so what it depends on is unknown")
            }
            Wrong::UnreadableLockFile => format!(
                "{LOCK_FILE} could not be read, so whether one crate is locked at two \
                 incompatible versions is unknown"
            ),
            Wrong::Unaccounted { manifest, name } => format!(
                "{manifest} depends on {name:?} and {REASONS_FILE} gives no reason for it. Add \
                 the entry in the same change that adds the dependency, or the reason is \
                 reconstructed later by somebody who was not there"
            ),
            Wrong::AccountedForNothing { name } => format!(
                "{REASONS_FILE} gives a reason for {name:?} and no manifest in this tree \
                 depends on it. A register carrying rows for departed crates stops being read"
            ),
            Wrong::SeveralSeries { name, versions } => format!(
                "{name:?} is locked at {}, which are different compatibility series, so two \
                 copies reach one binary and only one of them was looked at",
                versions.join(" and ")
            ),
        }
    }
}

/// Whether a table header opens a dependency table.
///
/// The four shapes cargo takes: the three kinds at the top level, the same three
/// under `[target.<cfg>.…]`, the workspace-wide table, and the per-dependency
/// sub-table `[dependencies.<name>]`, whose name is in the header rather than on
/// a line inside it.
fn dependency_table(header: &str) -> Option<Option<String>> {
    let kinds = ["dependencies", "dev-dependencies", "build-dependencies"];
    let segments: Vec<&str> = header.split('.').collect();
    // The kind is looked for wherever it sits rather than at a fixed position,
    // because what precedes it varies: nothing at the top level, `workspace`,
    // and `target` followed by a platform expression whose length is not fixed.
    // Anything after it is the name of a single dependency's own table.
    let at = segments
        .iter()
        .position(|segment| kinds.contains(segment))?;
    match &segments[at + 1..] {
        [] => Some(None),
        [name] => Some(Some((*name).to_owned())),
        _ => None,
    }
}

/// The dependency name a line inside a dependency table declares, if any.
///
/// `name = "1"`, `name = { … }` and `name.workspace = true` all declare `name`.
fn declared_on(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let (key, _) = trimmed.split_once('=')?;
    let key = key.trim();
    let name = key.split('.').next()?.trim().trim_matches('"');
    if name.is_empty() {
        return None;
    }
    Some(name.to_owned())
}

/// Every direct dependency one manifest declares.
///
/// Split from the file reading so a fixture can be read without existing on
/// disk, which is how the refusals below are proved.
pub fn directs_in(manifest: &str, text: &str) -> Vec<Direct> {
    let mut found: Vec<Direct> = Vec::new();
    let mut table: Option<Option<String>> = None;
    let mut sub_table_is_a_path = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(header) = trimmed.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            if let Some(entry) = found.last_mut() {
                if sub_table_is_a_path {
                    entry.inside_the_tree = true;
                }
            }
            sub_table_is_a_path = false;
            table = dependency_table(header.trim().trim_start_matches('['));
            if let Some(Some(name)) = table.clone() {
                found.push(Direct {
                    manifest: manifest.to_owned(),
                    name,
                    inside_the_tree: false,
                });
            }
            continue;
        }
        match &table {
            None => continue,
            // Inside `[dependencies.<name>]` the lines describe the dependency
            // named by the header rather than new ones.
            Some(Some(_)) => {
                if trimmed.starts_with("path") {
                    sub_table_is_a_path = true;
                }
            }
            Some(None) => {
                if let Some(name) = declared_on(trimmed) {
                    found.push(Direct {
                        manifest: manifest.to_owned(),
                        name,
                        inside_the_tree: trimmed.contains("path"),
                    });
                }
            }
        }
    }
    if let Some(entry) = found.last_mut() {
        if sub_table_is_a_path {
            entry.inside_the_tree = true;
        }
    }
    found.sort();
    found.dedup();
    found
}

/// The crate names the register gives a reason for.
///
/// One row per crate, as a list item opening with the name in backticks. The
/// shape is fixed here rather than in the document, so a row written in another
/// shape is a row this check does not see, and the document says so.
pub fn accounted_in(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("- `") else {
            continue;
        };
        let Some((name, tail)) = rest.split_once('`') else {
            continue;
        };
        // A row is a reason as well as a name. A bare name with nothing after it
        // is the shape somebody reaches for when they want the check to pass.
        if name.is_empty() || tail.trim().len() < 2 {
            continue;
        }
        names.push(name.to_owned());
    }
    names.sort();
    names.dedup();
    names
}

/// Every locked package, as the lock file records it.
pub fn locked_in(text: &str) -> Vec<Locked> {
    let mut found = Vec::new();
    let mut name: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            name = None;
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("name = ") {
            name = Some(value.trim_matches('"').to_owned());
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("version = ") {
            if let Some(name) = name.take() {
                found.push(Locked {
                    name,
                    version: value.trim_matches('"').to_owned(),
                });
            }
        }
    }
    found.sort();
    found
}

/// Judge the manifests against the register, and the lock file against itself.
///
/// A manifest is a relative path and its text, or nothing where it could not be
/// read. `reasons` is the text of [`REASONS_FILE`], or nothing where the file is
/// absent, and `lock` the same for [`LOCK_FILE`].
pub fn judge(
    manifests: &[(String, Option<String>)],
    reasons: Option<&str>,
    lock: Option<&str>,
) -> Vec<Wrong> {
    let mut wrongs = Vec::new();

    let mut directs: Vec<Direct> = Vec::new();
    for (path, text) in manifests {
        let Some(text) = text else {
            wrongs.push(Wrong::UnreadableManifest {
                manifest: path.clone(),
            });
            continue;
        };
        directs.extend(directs_in(path, text));
    }

    let accounted = match reasons {
        None => {
            wrongs.push(Wrong::NoReasonsFile);
            Vec::new()
        }
        Some(text) => accounted_in(text),
    };

    let mut wanted: Vec<String> = Vec::new();
    for direct in &directs {
        if direct.inside_the_tree {
            continue;
        }
        wanted.push(direct.name.clone());
        if !accounted.contains(&direct.name) {
            wrongs.push(Wrong::Unaccounted {
                manifest: direct.manifest.clone(),
                name: direct.name.clone(),
            });
        }
    }
    for name in accounted {
        if !wanted.contains(&name) {
            wrongs.push(Wrong::AccountedForNothing { name });
        }
    }

    match lock {
        None => wrongs.push(Wrong::UnreadableLockFile),
        Some(text) => {
            let locked = locked_in(text);
            let mut reported: Vec<String> = Vec::new();
            for package in &locked {
                if reported.contains(&package.name) {
                    continue;
                }
                let mut series: Vec<String> = Vec::new();
                let mut versions: Vec<String> = Vec::new();
                for other in locked.iter().filter(|other| other.name == package.name) {
                    if !series.contains(&other.series()) {
                        series.push(other.series());
                        versions.push(other.version.clone());
                    }
                }
                if series.len() > 1 {
                    reported.push(package.name.clone());
                    wrongs.push(Wrong::SeveralSeries {
                        name: package.name.clone(),
                        versions,
                    });
                }
            }
        }
    }

    wrongs
}

/// Judge this tree.
pub fn judge_the_tree() -> Vec<Wrong> {
    judge_below(&workspace_root())
}

/// Judge a tree rooted anywhere, so the walk itself can be exercised on a
/// fixture directory rather than only on this repository.
pub fn judge_below(root: &Path) -> Vec<Wrong> {
    let manifests: Vec<(String, Option<String>)> = crate::licence::manifest_paths(root)
        .into_iter()
        .map(|relative| {
            let text = fs::read_to_string(root.join(&relative)).ok();
            (relative, text)
        })
        .collect();
    let reasons = fs::read_to_string(root.join(REASONS_FILE)).ok();
    let lock = fs::read_to_string(root.join(LOCK_FILE)).ok();
    judge(&manifests, reasons.as_deref(), lock.as_deref())
}
