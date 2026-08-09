//! The dependency-level half of the offline guarantee, run over this tree and
//! proved to bite (#36).

use indexwerk_checks::dependencies::locked_in;
use indexwerk_checks::egress_dependencies::{
    ALLOWED, Brings, FORBIDDEN, Wrong, judge, judge_the_tree,
};
use indexwerk_checks::workspace_root;

/// A lock entry, written the way cargo writes one.
fn locked(name: &str, version: &str) -> String {
    format!("[[package]]\nname = \"{name}\"\nversion = \"{version}\"\n\n")
}

#[test]
fn nothing_in_this_tree_brings_a_route_off_this_host() {
    let wrongs = judge_the_tree();
    if !wrongs.is_empty() {
        let report: Vec<String> = wrongs.iter().map(Wrong::message).collect();
        panic!(
            "{} locked crate(s) bring a route off this host:\n{}",
            wrongs.len(),
            report.join("\n")
        );
    }
}

/// A judgement over nothing passes for the wrong reason, and while this tree has
/// no third-party dependency at all that is the failure mode with teeth. This
/// leg reads the same file the check reads and says what is actually in it.
#[test]
fn the_lock_file_this_judges_is_the_one_in_the_tree_and_it_holds_this_repository_only() {
    let text = std::fs::read_to_string(workspace_root().join("Cargo.lock"))
        .expect("Cargo.lock is what this check reads");
    let names: Vec<String> = locked_in(&text)
        .into_iter()
        .map(|package| package.name)
        .collect();
    assert_eq!(
        names,
        vec![
            "indexwerk-checks",
            "indexwerk-core",
            "indexwerk-ffi",
            "indexwerk-python"
        ],
        "the locked tree is four packages, all of them in this repository"
    );
}

/// The one the clause names: a networking dependency arrives and the check reds.
#[test]
fn a_networking_dependency_is_refused() {
    let lock = format!(
        "{}{}",
        locked("indexwerk-core", "0.0.0"),
        locked("reqwest", "0.12.9")
    );
    match judge(&lock).as_slice() {
        [wrong] => {
            assert_eq!(wrong.name, "reqwest");
            assert_eq!(wrong.version, "0.12.9");
            assert_eq!(wrong.brings, Brings::ANetworkStack);
            let message = wrong.message();
            assert!(
                message.contains("0008-nothing-leaves-the-host"),
                "the refusal names where the rule comes from: {message}"
            );
        }
        other => panic!("a networking dependency has to be refused, got {other:?}"),
    }
}

/// The real risk is not the direct edge. A lock file carries the whole
/// transitive set, so a stack arriving under something arriving under something
/// is in what is read, and this is the leg that says so.
#[test]
fn a_stack_three_levels_down_is_refused_the_same_as_a_direct_one() {
    let lock = format!(
        "{}{}{}",
        locked("convenient", "1.0.0"),
        locked("innocuous", "2.1.0"),
        locked("mio", "1.0.2")
    );
    match judge(&lock).as_slice() {
        [wrong] => assert_eq!(wrong.name, "mio"),
        other => panic!("a transitive network stack has to be refused, got {other:?}"),
    }
}

#[test]
fn a_telemetry_client_and_a_crash_reporter_are_refused_and_are_told_apart() {
    let lock = format!(
        "{}{}",
        locked("opentelemetry", "0.27.0"),
        locked("sentry", "0.34.0")
    );
    let wrongs = judge(&lock);
    let kinds: Vec<Brings> = wrongs.iter().map(|wrong| wrong.brings).collect();
    assert_eq!(
        kinds,
        vec![Brings::ATelemetryClient, Brings::ACrashReporter]
    );
}

/// A crate whose name merely starts or ends like a forbidden one is not that
/// crate. The lock file holds exact names, so this compares exactly, and the
/// leg exists because a substring comparison is the shortcut somebody takes.
#[test]
fn a_name_that_merely_contains_a_forbidden_one_is_not_refused() {
    let lock = format!(
        "{}{}{}",
        locked("tokio-console-subscriber-lookalike", "1.0.0"),
        locked("openssl-probe", "0.1.5"),
        locked("miori", "0.3.0")
    );
    assert_eq!(judge(&lock), Vec::new());
}

/// The allow list is empty at the first release, which is the clause's own
/// wording. A test rather than a sentence, because an entry added quietly is
/// exactly how this check stops meaning anything.
#[test]
fn the_allow_list_is_empty() {
    assert_eq!(
        ALLOWED,
        &[] as &[&str],
        "an entry here needs an issue arguing for it, and the issue is named in the entry"
    );
}

/// Every name is listed once. A duplicate would report one crate twice and read
/// as two findings.
#[test]
fn no_name_is_listed_twice() {
    let mut names: Vec<&str> = FORBIDDEN.iter().map(|entry| entry.name).collect();
    let before = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), before, "a name is listed twice in FORBIDDEN");
}
