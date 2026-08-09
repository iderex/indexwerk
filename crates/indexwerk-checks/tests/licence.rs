//! The licence check, run over this tree and proved to bite (#47).
//!
//! Each refusal below breaks one thing in an otherwise valid set, so a test that
//! passes for the wrong reason is visible: the fixtures differ from the accepted
//! one by the single fact the refusal is named after.

use std::fs;

use indexwerk_checks::licence::{
    Declaration, IDENTIFIER, LICENCE_FILE, LICENCE_TITLE, LICENCE_VERSION, ROOT_MANIFEST, Wrong,
    declaration_in, judge, judge_the_tree, manifest_paths,
};
use indexwerk_checks::workspace_root;

/// A licence text the check accepts.
fn the_licence() -> String {
    format!(
        "                    {LICENCE_TITLE}\n                       {LICENCE_VERSION}\n\nterms follow.\n"
    )
}

/// A root manifest declaring the identifier.
fn a_root() -> (String, Option<String>) {
    (
        ROOT_MANIFEST.to_owned(),
        Some(format!(
            "[workspace]\nmembers = [\"crates/a\"]\n\n[workspace.package]\nedition = \"2024\"\nlicense = \"{IDENTIFIER}\"\n"
        )),
    )
}

/// A member manifest inheriting it.
fn a_member(path: &str) -> (String, Option<String>) {
    (
        path.to_owned(),
        Some("[package]\nname = \"a\"\nlicense.workspace = true\n".to_owned()),
    )
}

#[test]
fn every_manifest_in_this_tree_declares_the_licence_and_the_file_carries_it() {
    let wrongs = judge_the_tree();
    if !wrongs.is_empty() {
        let report: Vec<String> = wrongs.iter().map(Wrong::message).collect();
        panic!(
            "the licence is declared inconsistently in {} place(s):\n{}",
            wrongs.len(),
            report.join("\n")
        );
    }
}

/// A judgement over nothing is a judgement that passes for the wrong reason.
/// This is the leg that separates a clean tree from an unwalked one.
#[test]
fn the_walk_reached_the_manifests_this_tree_has() {
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
        fs::read_to_string(root.join(LICENCE_FILE)).is_ok(),
        "{LICENCE_FILE} is what the identifier in every one of those manifests names"
    );
}

#[test]
fn a_valid_set_is_accepted() {
    let manifests = vec![a_root(), a_member("crates/a/Cargo.toml")];
    assert_eq!(judge(&manifests, Some(&the_licence())), Vec::new());
}

#[test]
fn a_manifest_declaring_no_licence_is_refused() {
    let manifests = vec![
        a_root(),
        (
            "crates/a/Cargo.toml".to_owned(),
            Some("[package]\nname = \"a\"\nedition = \"2024\"\n".to_owned()),
        ),
    ];
    match judge(&manifests, Some(&the_licence())).as_slice() {
        [Wrong::Undeclared { manifest }] => assert_eq!(manifest, "crates/a/Cargo.toml"),
        other => panic!("a manifest with no licence key has to be refused, got {other:?}"),
    }
}

#[test]
fn a_manifest_declaring_another_identifier_is_refused() {
    let manifests = vec![
        a_root(),
        (
            "crates/a/Cargo.toml".to_owned(),
            Some("[package]\nname = \"a\"\nlicense = \"MIT\"\n".to_owned()),
        ),
    ];
    match judge(&manifests, Some(&the_licence())).as_slice() {
        [Wrong::Another { manifest, declared }] => {
            assert_eq!(manifest, "crates/a/Cargo.toml");
            assert_eq!(declared, "MIT");
            let message = judge(&manifests, Some(&the_licence()))[0].message();
            assert!(
                message.contains("MIT"),
                "the refusal names what it found: {message}"
            );
            assert!(
                message.contains(IDENTIFIER),
                "and what the rest of the tree says: {message}"
            );
        }
        other => panic!("a second identifier has to be refused, got {other:?}"),
    }
}

/// The mistake somebody will actually make: the `-only` spelling of the same
/// licence, in one manifest and not in the others. It is the exact string every
/// manifest here carried until the answer of 2026-08-09 widened the grant, so a
/// manifest copied from an older revision of this tree, or a crate whose key was
/// written from memory, arrives spelled this way and is a narrower grant than
/// the one offered.
#[test]
fn the_neighbouring_spelling_of_the_same_licence_is_refused() {
    let manifests = vec![
        a_root(),
        (
            "crates/a/Cargo.toml".to_owned(),
            Some("[package]\nname = \"a\"\nlicense = \"AGPL-3.0-only\"\n".to_owned()),
        ),
    ];
    match judge(&manifests, Some(&the_licence())).as_slice() {
        [Wrong::Another { declared, .. }] => assert_eq!(declared, "AGPL-3.0-only"),
        other => panic!("a neighbouring spelling has to be refused, got {other:?}"),
    }
}

#[test]
fn a_manifest_pointing_at_a_file_where_an_identifier_belongs_is_refused() {
    let manifests = vec![
        a_root(),
        (
            "crates/a/Cargo.toml".to_owned(),
            Some("[package]\nname = \"a\"\nlicense-file = \"LICENSE\"\n".to_owned()),
        ),
    ];
    match judge(&manifests, Some(&the_licence())).as_slice() {
        [Wrong::APathRatherThanAnIdentifier { path, .. }] => assert_eq!(path, "LICENSE"),
        other => panic!("a path where an identifier belongs has to be refused, got {other:?}"),
    }
}

#[test]
fn a_member_inheriting_from_a_root_that_declares_nothing_is_refused() {
    let manifests = vec![
        (
            ROOT_MANIFEST.to_owned(),
            Some("[workspace]\n\n[workspace.package]\nedition = \"2024\"\n".to_owned()),
        ),
        a_member("crates/a/Cargo.toml"),
    ];
    let wrongs = judge(&manifests, Some(&the_licence()));
    assert!(
        wrongs.contains(&Wrong::Undeclared {
            manifest: ROOT_MANIFEST.to_owned()
        }),
        "the root declaring nothing is itself a finding: {wrongs:?}"
    );
    assert!(
        wrongs.contains(&Wrong::InheritedFromNothing {
            manifest: "crates/a/Cargo.toml".to_owned()
        }),
        "and so is the member inheriting nothing from it: {wrongs:?}"
    );
}

/// A package outside the workspace inherits from no workspace, so the key that
/// works for a member declares nothing here. `harness/` is that case.
#[test]
fn a_package_with_its_own_workspace_may_not_inherit_the_key() {
    let manifests = vec![
        a_root(),
        (
            "harness/Cargo.toml".to_owned(),
            Some("[workspace]\n\n[package]\nname = \"h\"\nlicense.workspace = true\n".to_owned()),
        ),
    ];
    match judge(&manifests, Some(&the_licence())).as_slice() {
        [Wrong::InheritedFromNothing { manifest }] => assert_eq!(manifest, "harness/Cargo.toml"),
        other => panic!("inheriting outside a workspace has to be refused, got {other:?}"),
    }
}

#[test]
fn an_absent_licence_file_is_refused() {
    match judge(&[a_root()], None).as_slice() {
        [Wrong::NoLicenceFile] => {}
        other => panic!("an identifier naming an absent file has to be refused, got {other:?}"),
    }
}

#[test]
fn a_licence_file_carrying_another_text_is_refused() {
    let another = "MIT License\n\nPermission is hereby granted, free of charge.\n";
    match judge(&[a_root()], Some(another)).as_slice() {
        [Wrong::LicenceFileIsAnotherText { opening }] => {
            assert_eq!(opening, "MIT License");
            let message = judge(&[a_root()], Some(another))[0].message();
            assert!(
                message.contains("MIT License"),
                "the refusal says what it found: {message}"
            );
        }
        other => panic!("a file that is not the named text has to be refused, got {other:?}"),
    }
}

/// The title alone would accept version 2 of the same licence, which reads the
/// same at a glance and is a different set of terms.
#[test]
fn the_same_licence_at_another_version_is_refused() {
    let older = format!(
        "                    {LICENCE_TITLE}\n                       Version 2, 3 November 2004\n"
    );
    match judge(&[a_root()], Some(&older)).as_slice() {
        [Wrong::LicenceFileIsAnotherText { .. }] => {}
        other => panic!("another version of the same licence has to be refused, got {other:?}"),
    }
}

#[test]
fn a_manifest_that_cannot_be_read_is_not_a_manifest_that_was_found_clean() {
    let manifests = vec![a_root(), ("crates/a/Cargo.toml".to_owned(), None)];
    match judge(&manifests, Some(&the_licence())).as_slice() {
        [Wrong::Unreadable { manifest }] => assert_eq!(manifest, "crates/a/Cargo.toml"),
        other => panic!("an unreadable manifest has to be refused, got {other:?}"),
    }
}

#[test]
fn both_spellings_of_inheritance_are_read_and_a_commented_key_is_not() {
    assert_eq!(
        declaration_in("license.workspace = true\n"),
        Declaration::Inherited
    );
    assert_eq!(
        declaration_in("license = { workspace = true }\n"),
        Declaration::Inherited
    );
    assert_eq!(
        declaration_in(&format!("license = \"{IDENTIFIER}\"\n")),
        Declaration::Literal(IDENTIFIER.to_owned())
    );
    assert_eq!(
        declaration_in("# license = \"MIT\"\n"),
        Declaration::Absent,
        "a key inside a comment is a sentence about the manifest, not a declaration"
    );
}
