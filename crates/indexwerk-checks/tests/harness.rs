//! What the ordinary suite says about the legs it did not run (#18).
//!
//! Two things are checked here. That the declaration in `harness/README.md` is
//! well formed, because a malformed table would report fewer legs than exist
//! and the report would be worth less than nothing. And that the separation
//! keeping those legs out of this suite is a property of the manifests rather
//! than a convention.
//!
//! The report itself is written straight to the process's standard error rather
//! than through `println!`, because the test harness captures the macros and a
//! report nobody sees is not a report. `cargo test --workspace --locked` prints
//! it on every run, green or not.

use std::fs;
use std::io::Write;

use indexwerk_checks::harness::{
    DECLARATION, KINDS, Malformed, declared_legs, did_not_run, legs_from,
};
use indexwerk_checks::workspace_root;

/// A table with the shape the parser wants, so each refusal test can break one
/// thing and leave everything else valid.
fn a_declaration(rows: &str) -> String {
    format!(
        "# A harness\n\nSome prose that is not a table.\n\n\
         | Leg | Kind of requirement | What it requires | How to run it |\n\
         | --- | --- | --- | --- |\n{rows}\nMore prose.\n"
    )
}

fn a_row(id: &str, kind: &str, requires: &str) -> String {
    format!(
        "| `{id}` | {kind} | {requires} | `cargo run --manifest-path harness/Cargo.toml -- run {id}` |\n"
    )
}

#[test]
fn every_declared_leg_is_reported_as_not_run() {
    let legs = match declared_legs() {
        Ok(legs) => legs,
        Err(why) => panic!("{}", why.message()),
    };

    // Built whole and written once. Tests run in parallel and each write is a
    // place another test's line can land in the middle of this one.
    let mut report = format!(
        "\nThe hardware-bound harness of #18 did not run in this suite. {} leg(s), each with \
         what running it would require:\n",
        legs.len()
    );
    for leg in &legs {
        let line = did_not_run(leg);
        assert!(line.contains(&leg.id), "the report has to name the leg");
        assert!(
            line.contains(&leg.requires),
            "the report has to say what running the leg would require, not only that it did not run"
        );
        assert!(
            line.contains(&leg.command),
            "the report has to carry the command that runs the leg"
        );
        report.push_str(&format!("  {line}\n"));
    }
    report.push_str("  A green run of this suite says nothing about any of them.\n\n");
    let _ = std::io::stderr().write_all(report.as_bytes());
}

#[test]
fn the_declaration_in_the_tree_parses() {
    match declared_legs() {
        Ok(legs) => assert!(!legs.is_empty(), "the tree declares at least one leg"),
        Err(why) => panic!("{}", why.message()),
    }
}

#[test]
fn every_kind_the_issue_fixes_is_accepted() {
    for kind in KINDS {
        let text = a_declaration(&a_row("a-leg", kind, "something real"));
        assert!(
            legs_from(&text).is_ok(),
            "{kind} is one of the four kinds #18 fixes and has to parse"
        );
    }
}

#[test]
fn a_requirement_kind_outside_the_four_is_refused() {
    let text = a_declaration(&a_row("a-leg", "a fast afternoon", "something real"));
    match legs_from(&text) {
        Err(Malformed::Row { row, why }) => {
            assert_eq!(row, 1);
            assert!(
                why.contains("a fast afternoon"),
                "the refusal names it: {why}"
            );
        }
        other => panic!("a kind outside the four has to be refused, got {other:?}"),
    }
}

#[test]
fn a_leg_that_says_only_what_kind_of_thing_it_needs_is_refused() {
    let text = a_declaration(&a_row("a-leg", "memory", ""));
    match legs_from(&text) {
        Err(Malformed::Row { row, why }) => {
            assert_eq!(row, 1);
            assert!(why.contains("a-leg"), "the refusal names the leg: {why}");
        }
        other => panic!("an empty requirement has to be refused, got {other:?}"),
    }
}

#[test]
fn a_leg_run_by_something_other_than_the_harness_is_refused() {
    let text = a_declaration("| `a-leg` | memory | a lot of it | `cargo test --workspace` |\n");
    match legs_from(&text) {
        Err(Malformed::Row { why, .. }) => assert!(
            why.contains("cargo test --workspace"),
            "the refusal quotes the command: {why}"
        ),
        other => panic!("a leg run by the default suite has to be refused, got {other:?}"),
    }
}

#[test]
fn a_leg_declared_twice_is_refused() {
    let mut rows = a_row("a-leg", "memory", "a lot of it");
    rows.push_str(&a_row("a-leg", "core count", "a lot of them"));
    match legs_from(&a_declaration(&rows)) {
        Err(Malformed::Row { row, why }) => {
            assert_eq!(row, 2);
            assert!(why.contains("twice"), "{why}");
        }
        other => panic!("a duplicate identifier has to be refused, got {other:?}"),
    }
}

#[test]
fn an_identifier_that_is_not_the_one_the_command_spells_is_refused() {
    let text = a_declaration(&a_row("A Leg", "memory", "a lot of it"));
    match legs_from(&text) {
        Err(Malformed::Row { why, .. }) => assert!(why.contains("A Leg"), "{why}"),
        other => panic!("a free-text identifier has to be refused, got {other:?}"),
    }
}

#[test]
fn a_row_with_the_wrong_number_of_cells_is_refused() {
    let text = a_declaration("| `a-leg` | memory | a lot of it |\n");
    match legs_from(&text) {
        Err(Malformed::Row { why, .. }) => {
            assert!(
                why.contains("cells"),
                "the refusal says what it counted: {why}"
            )
        }
        other => panic!("a short row has to be refused, got {other:?}"),
    }
}

#[test]
fn a_declaration_with_no_table_is_refused() {
    match legs_from("# A harness\n\nNo table here at all.\n") {
        Err(Malformed::NoTable) => {}
        other => panic!("a declaration with no table has to be refused, got {other:?}"),
    }
}

#[test]
fn a_table_with_no_rows_is_refused() {
    match legs_from(&a_declaration("")) {
        Err(Malformed::NoLegs) => {}
        other => panic!("an empty table has to be refused, got {other:?}"),
    }
}

/// The floor and the edition are written twice, once in each manifest, because
/// a package outside the workspace cannot inherit from it. Two copies of a
/// number are two things that have to move together, so this is the thing that
/// moves them.
#[test]
fn the_harness_manifest_carries_the_same_floor_as_the_workspace() {
    let root = workspace_root();
    let workspace = match fs::read_to_string(root.join("Cargo.toml")) {
        Ok(text) => text,
        Err(error) => panic!("the workspace manifest could not be read: {error}"),
    };
    let harness = match fs::read_to_string(root.join("harness/Cargo.toml")) {
        Ok(text) => text,
        Err(error) => panic!("the harness manifest could not be read: {error}"),
    };
    for key in ["edition", "rust-version"] {
        let declared = value_of(&workspace, key);
        let copied = value_of(&harness, key);
        assert!(
            declared.is_some(),
            "the workspace manifest declares no {key} to compare against"
        );
        assert_eq!(
            declared, copied,
            "harness/Cargo.toml has drifted from the workspace {key}"
        );
    }
}

/// The first `key = "value"` in a manifest, read as text rather than parsed.
/// This crate has no dependencies and a manifest parser would be one.
fn value_of(manifest: &str, key: &str) -> Option<String> {
    manifest
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix(key))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .find_map(|rest| {
            let value = rest.trim();
            value
                .strip_prefix('"')
                .and_then(|open| open.split('"').next())
                .map(str::to_owned)
        })
}

#[test]
fn the_harness_is_outside_this_workspace_by_the_manifest_rather_than_by_habit() {
    let root = workspace_root();
    let workspace = match fs::read_to_string(root.join("Cargo.toml")) {
        Ok(text) => text,
        Err(error) => panic!("the workspace manifest could not be read: {error}"),
    };
    assert!(
        !workspace.contains("\"harness\","),
        "harness must not be a workspace member, or its legs join the default test command"
    );
    assert!(
        workspace.contains("exclude"),
        "the workspace manifest has to exclude harness/, so a reader of it is told the directory \
         exists and was considered"
    );
    assert!(
        DECLARATION.starts_with("harness/"),
        "the declaration this suite reports from lives in the excluded directory"
    );
}
