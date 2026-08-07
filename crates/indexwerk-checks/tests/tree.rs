//! The check itself, run over this tree.
//!
//! This file is scanned like any other source, so a construct written here
//! would be refused here.

use indexwerk_checks::{scan_workspace, workspace_root};

#[test]
fn no_source_in_this_tree_needs_a_display_the_network_or_elevation() {
    let findings = scan_workspace();
    if !findings.is_empty() {
        let report: Vec<String> = findings.iter().map(|finding| finding.to_string()).collect();
        panic!(
            "the headless and unelevated birth requirement is broken in {} place(s):\n{}\n\n\
             A change that makes the suite need elevation is a defect, not a step to \
             document. Where a test genuinely needs one of these it goes into the \
             hardware-bound harness of issue #18 and never into the default suite.",
            findings.len(),
            report.join("\n")
        );
    }
}

#[test]
fn the_scan_actually_reached_some_files() {
    // A scan that found nothing because it walked nothing would pass the test
    // above and prove nothing. This is the leg that separates the two.
    let root = workspace_root();
    assert!(
        root.join("crates/indexwerk-core/src/lib.rs").exists(),
        "the workspace root was derived wrongly: {}",
        root.display()
    );
    let sources = count_rust_sources(&root.join("crates"));
    assert!(
        sources >= 5,
        "expected the walk to reach at least the five sources this workspace has, reached {sources}"
    );
}

fn count_rust_sources(directory: &std::path::Path) -> usize {
    let mut total = 0;
    let Ok(entries) = std::fs::read_dir(directory) else {
        return 0;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            total += count_rust_sources(&path);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            total += 1;
        }
    }
    total
}
