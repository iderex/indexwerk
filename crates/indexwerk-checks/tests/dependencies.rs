//! The dependency policy, run over this tree and proved to bite (#38).
//!
//! Each refusal below breaks one thing in an otherwise accepted set, so a test
//! passing for the wrong reason is visible: the fixtures differ from the
//! accepted one by the single fact the refusal is named after.
//!
//! The register this tree carries is empty, which is what makes the fixtures
//! carry the weight here. A check whose only evidence is a clean tree with
//! nothing in it is a check that has never been shown to fire.

use indexwerk_checks::dependencies::{
    Direct, LOCK_FILE, Locked, REASONS_FILE, Wrong, accounted_in, directs_in, judge,
    judge_the_tree, locked_in,
};
use indexwerk_checks::licence::manifest_paths;
use indexwerk_checks::workspace_root;

/// A lock file with nothing duplicated.
fn a_lock() -> String {
    "[[package]]\nname = \"alpha\"\nversion = \"1.2.3\"\n".to_owned()
}

/// A manifest depending on one crate from outside the tree.
fn a_manifest_wanting(name: &str) -> (String, Option<String>) {
    (
        "crates/a/Cargo.toml".to_owned(),
        Some(format!(
            "[package]\nname = \"a\"\n\n[dependencies]\n{name} = \"1\"\n"
        )),
    )
}

#[test]
fn this_tree_accounts_for_every_direct_dependency_it_has() {
    let wrongs = judge_the_tree();
    if !wrongs.is_empty() {
        let report: Vec<String> = wrongs.iter().map(Wrong::message).collect();
        panic!(
            "the dependency policy is broken in {} place(s):\n{}",
            wrongs.len(),
            report.join("\n")
        );
    }
}

/// A judgement over nothing passes for the wrong reason. This is the leg that
/// separates a tree with no outside dependencies from a walk that reached no
/// manifest, and it is the one that matters most while the register is empty.
#[test]
fn the_walk_reached_the_manifests_and_the_two_files_it_judges_against() {
    let root = workspace_root();
    let paths = manifest_paths(&root);
    for expected in [
        "Cargo.toml",
        "crates/indexwerk-checks/Cargo.toml",
        "crates/indexwerk-core/Cargo.toml",
        "crates/indexwerk-ffi/Cargo.toml",
        "crates/indexwerk-python/Cargo.toml",
        "harness/Cargo.toml",
    ] {
        assert!(
            paths.iter().any(|found| found == expected),
            "the walk did not reach {expected}; it reached {paths:?}"
        );
    }
    assert!(
        root.join(REASONS_FILE).is_file(),
        "{REASONS_FILE} is what a direct dependency is judged against"
    );
    assert!(
        root.join(LOCK_FILE).is_file(),
        "{LOCK_FILE} is what the duplicate series check reads"
    );
}

/// The in-tree edges are the layering rather than the supply chain, and asking
/// them for a reason would fill the register with rows about this repository.
#[test]
fn the_path_edges_between_the_layers_of_this_tree_are_read_and_are_not_entries() {
    let root = workspace_root();
    let text =
        std::fs::read_to_string(root.join("crates/indexwerk-ffi/Cargo.toml")).unwrap_or_default();
    let directs = directs_in("crates/indexwerk-ffi/Cargo.toml", &text);
    assert_eq!(
        directs,
        vec![Direct {
            manifest: "crates/indexwerk-ffi/Cargo.toml".to_owned(),
            name: "indexwerk-core".to_owned(),
            inside_the_tree: true,
        }],
        "the C interface depends on the core by path, and the reader has to see \
         both the edge and that it is inside this tree"
    );
}

#[test]
fn an_accounted_dependency_is_taken() {
    let manifests = vec![a_manifest_wanting("alpha")];
    let reasons = "- `alpha` it does the thing, and writing it here would have cost more.\n";
    assert_eq!(
        judge(&manifests, Some(reasons), Some(&a_lock())),
        Vec::new()
    );
}

#[test]
fn a_direct_dependency_with_no_reason_is_refused() {
    let manifests = vec![a_manifest_wanting("alpha")];
    match judge(&manifests, Some("nothing here.\n"), Some(&a_lock())).as_slice() {
        [Wrong::Unaccounted { manifest, name }, ..] => {
            assert_eq!(manifest, "crates/a/Cargo.toml");
            assert_eq!(name, "alpha");
            let message = judge(&manifests, Some("nothing here.\n"), Some(&a_lock()))[0].message();
            assert!(
                message.contains(REASONS_FILE),
                "the refusal names where the reason belongs: {message}"
            );
        }
        other => panic!("a dependency with no reason has to be refused, got {other:?}"),
    }
}

/// The other direction. A register that keeps rows for crates nobody depends on
/// stops being read, and then the rows that do matter are not read either.
#[test]
fn a_reason_for_a_dependency_no_manifest_wants_is_refused() {
    let reasons = "- `departed` it was needed for the old parser.\n";
    match judge(&[], Some(reasons), Some(&a_lock())).as_slice() {
        [Wrong::AccountedForNothing { name }] => assert_eq!(name, "departed"),
        other => panic!("a row naming nothing has to be refused, got {other:?}"),
    }
}

/// The shape somebody reaches for to make a red check go away: the name, and
/// nothing said about it.
#[test]
fn a_row_carrying_a_name_and_no_reason_does_not_count_as_a_row() {
    assert_eq!(accounted_in("- `alpha`\n"), Vec::<String>::new());
    assert_eq!(accounted_in("- `alpha` .\n"), Vec::<String>::new());
    assert_eq!(
        accounted_in("- `alpha` it parses the vectors.\n"),
        vec!["alpha".to_owned()]
    );
}

#[test]
fn a_missing_register_is_refused_rather_than_read_as_an_empty_one() {
    match judge(&[], None, Some(&a_lock())).as_slice() {
        [Wrong::NoReasonsFile] => {}
        other => panic!("an absent register has to be refused, got {other:?}"),
    }
}

#[test]
fn a_manifest_that_cannot_be_read_is_not_a_manifest_that_was_found_to_want_nothing() {
    let manifests = vec![("crates/a/Cargo.toml".to_owned(), None)];
    match judge(&manifests, Some(""), Some(&a_lock())).as_slice() {
        [Wrong::UnreadableManifest { manifest }] => assert_eq!(manifest, "crates/a/Cargo.toml"),
        other => panic!("an unreadable manifest has to be refused, got {other:?}"),
    }
}

#[test]
fn a_lock_file_that_cannot_be_read_is_refused() {
    match judge(&[], Some(""), None).as_slice() {
        [Wrong::UnreadableLockFile] => {}
        other => panic!("an absent lock file has to be refused, got {other:?}"),
    }
}

/// Two majors of one crate, which is the shape that puts an audited copy and an
/// unaudited one in the same binary. No manifest asked for it: it arrives
/// because two dependencies disagree about a third.
#[test]
fn one_crate_locked_at_two_majors_is_refused() {
    let lock = "[[package]]\nname = \"alpha\"\nversion = \"1.2.3\"\n\n\
                [[package]]\nname = \"alpha\"\nversion = \"2.0.1\"\n";
    match judge(&[], Some(""), Some(lock)).as_slice() {
        [Wrong::SeveralSeries { name, versions }] => {
            assert_eq!(name, "alpha");
            assert_eq!(versions, &["1.2.3".to_owned(), "2.0.1".to_owned()]);
        }
        other => panic!("two majors of one crate have to be refused, got {other:?}"),
    }
}

/// Below one, the second component is the compatibility boundary. Reading only
/// the leading zero would call these one series and miss the case entirely,
/// which is the one-character mistake somebody writing this check makes.
#[test]
fn two_pre_one_minors_of_one_crate_are_two_series() {
    let lock = "[[package]]\nname = \"alpha\"\nversion = \"0.1.9\"\n\n\
                [[package]]\nname = \"alpha\"\nversion = \"0.2.0\"\n";
    match judge(&[], Some(""), Some(lock)).as_slice() {
        [Wrong::SeveralSeries { versions, .. }] => {
            assert_eq!(versions, &["0.1.9".to_owned(), "0.2.0".to_owned()]);
        }
        other => panic!("0.1 and 0.2 are incompatible and have to be refused, got {other:?}"),
    }
}

/// And two patches of one series are one series, so the check does not fire on
/// the case cargo resolves by itself.
#[test]
fn two_patches_of_one_series_are_not_a_finding() {
    let lock = "[[package]]\nname = \"alpha\"\nversion = \"1.2.3\"\n\n\
                [[package]]\nname = \"alpha\"\nversion = \"1.4.0\"\n";
    assert_eq!(judge(&[], Some(""), Some(lock)), Vec::new());
}

#[test]
fn the_lock_reader_takes_a_name_and_a_version_together() {
    assert_eq!(
        locked_in(&a_lock()),
        vec![Locked {
            name: "alpha".to_owned(),
            version: "1.2.3".to_owned(),
        }]
    );
}

/// The four table shapes cargo takes. A dependency written in one this reader
/// does not know is a dependency that walks through the register, so each is
/// exercised rather than assumed.
#[test]
fn every_table_shape_a_dependency_can_be_declared_in_is_read() {
    let text = "[package]\nname = \"a\"\n\n\
                [dependencies]\nalpha = \"1\"\nbeta = { version = \"2\" }\n\
                gamma.workspace = true\n\n\
                [dev-dependencies]\ndelta = \"1\"\n\n\
                [build-dependencies]\nepsilon = \"1\"\n\n\
                [target.'cfg(unix)'.dependencies]\nzeta = \"1\"\n\n\
                [dependencies.eta]\nversion = \"1\"\n";
    let names: Vec<String> = directs_in("Cargo.toml", text)
        .into_iter()
        .map(|direct| direct.name)
        .collect();
    assert_eq!(
        names,
        vec!["alpha", "beta", "delta", "epsilon", "eta", "gamma", "zeta"]
    );
}

/// A sub-table naming a path is the in-tree edge written the long way. Reading
/// it as an outside dependency would demand a reason for this repository's own
/// layering.
#[test]
fn a_path_dependency_written_as_a_sub_table_is_seen_as_inside_the_tree() {
    let text = "[package]\nname = \"a\"\n\n\
                [dependencies.indexwerk-core]\npath = \"../indexwerk-core\"\n";
    assert_eq!(
        directs_in("Cargo.toml", text),
        vec![Direct {
            manifest: "Cargo.toml".to_owned(),
            name: "indexwerk-core".to_owned(),
            inside_the_tree: true,
        }]
    );
}

/// Keys outside a dependency table are not dependencies, and a commented one is
/// a sentence about the manifest rather than a declaration.
#[test]
fn keys_outside_a_dependency_table_are_not_read_as_dependencies() {
    let text = "[package]\nname = \"a\"\nedition = \"2024\"\n\n\
                [lib]\ncrate-type = [\"rlib\"]\n\n\
                [dependencies]\n# alpha = \"1\"\n";
    assert_eq!(directs_in("Cargo.toml", text), Vec::new());
}
