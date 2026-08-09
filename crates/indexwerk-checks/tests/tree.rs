//! The check itself, run over this tree.
//!
//! This file is scanned like any other source, so a construct written here
//! would be refused here.

use std::collections::BTreeSet;
use std::fs;

use indexwerk_checks::terms::INVARIANTS;
use indexwerk_checks::{catalogue_markdown, files_the_scan_reads, scan_workspace, workspace_root};

const CATALOGUE: &str = "docs/invariants.md";

#[test]
fn this_tree_breaks_none_of_the_greppable_invariants() {
    let findings = scan_workspace();
    if !findings.is_empty() {
        let broken: BTreeSet<&str> = findings
            .iter()
            .map(|finding| finding.invariant().title())
            .collect();
        let report: Vec<String> = findings.iter().map(|finding| finding.to_string()).collect();
        panic!(
            "{} greppable invariant(s) broken in {} place(s):\n{}\n\n\
             Each line names where its rule comes from. The invariants and their sources \
             are listed in {}, which is rendered from the table in \
             crates/indexwerk-checks/src/terms.rs. Broken: {:?}",
            broken.len(),
            findings.len(),
            report.join("\n"),
            CATALOGUE,
            broken
        );
    }
}

#[test]
fn the_catalogue_in_the_tree_is_what_the_check_renders() {
    // The document may not drift against the thing it describes. Adding an
    // invariant to the table without regenerating this file reds the check,
    // and so does writing an entry here that no invariant backs.
    let path = workspace_root().join(CATALOGUE);
    let tracked = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        !tracked.is_empty(),
        "{} is missing or empty; it is rendered by catalogue_markdown()",
        path.display()
    );
    // The tree stores LF, declared in .gitattributes, so a checkout that
    // handed back CRLF would fail this for a reason that is not drift.
    let tracked = tracked.replace("\r\n", "\n");
    assert_eq!(
        tracked,
        catalogue_markdown(),
        "{CATALOGUE} is not what the check renders; regenerate it"
    );
}

/// How the catalogue is regenerated:
///
///     cargo test -p indexwerk-checks --test tree -- --ignored regenerate
///
/// Ignored by default because it writes into the tree, and a check that repairs
/// the thing it is checking is a check that never fails.
#[test]
#[ignore = "writes into the tree; run it deliberately after changing the table"]
fn regenerate_the_catalogue() {
    let path = workspace_root().join(CATALOGUE);
    fs::write(&path, catalogue_markdown()).expect("the docs directory is writable");
    println!("wrote {}", path.display());
}

#[test]
fn the_scan_actually_reached_the_sources_and_the_documentation() {
    // A scan that found nothing because it walked nothing would pass the test
    // above and prove nothing. This is the leg that separates the two.
    let root = workspace_root();
    assert!(
        root.join("crates/indexwerk-core/src/lib.rs").exists(),
        "the workspace root was derived wrongly: {}",
        root.display()
    );
    let sources = count_files(&root.join("crates"), "rs");
    assert!(
        sources >= 5,
        "expected the walk to reach at least the five sources this workspace has, reached {sources}"
    );
    let documents = count_files(&root.join("docs"), "md");
    assert!(
        documents >= INVARIANTS.len(),
        "expected the walk to reach the documentation directory, reached {documents} file(s)"
    );
}

#[test]
fn the_walk_opens_the_harness_as_well_as_the_workspace() {
    // The leg above counts what is on disk, which is not the same as what the
    // walk opened, and the difference is the whole of this one. `harness/` is
    // outside the workspace on purpose, so it is the directory a walk written
    // around `cargo metadata` would miss, and the miss would be silent: with
    // no violation there, dropping it changes no other test.
    let read = files_the_scan_reads();
    assert!(
        read.iter().any(|path| path == "harness/src/lib.rs"),
        "the walk did not open the harness; it opened {read:?}"
    );
    assert!(
        read.iter().any(|path| path == "harness/tests/refusal.rs"),
        "the walk did not open the harness suite; it opened {read:?}"
    );
    assert!(
        read.iter()
            .any(|path| path == "crates/indexwerk-core/src/lib.rs"),
        "the walk did not open the core"
    );
    // Build output is not source, and the harness has its own.
    assert!(
        !read.iter().any(|path| path.contains("/target/")),
        "the walk descended into build output"
    );
    // The exclusions are applied by the walk rather than after it, so a file
    // named there is never opened at all.
    assert!(
        !read
            .iter()
            .any(|path| path == "crates/indexwerk-checks/src/terms.rs"),
        "the walk opened a file the exclusion list names"
    );
}

fn count_files(directory: &std::path::Path, extension: &str) -> usize {
    let mut total = 0;
    let Ok(entries) = fs::read_dir(directory) else {
        return 0;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            total += count_files(&path, extension);
        } else if path.extension().is_some_and(|found| found == extension) {
            total += 1;
        }
    }
    total
}
