//! Proof that the check bites, one class at a time.
//!
//! This file is excluded from the scan it exercises, for the same reason the
//! term table is: it has to contain the constructs to feed them to the scanner.
//! The exclusion is by name, in `terms::EXCLUDED`, and the test at the bottom
//! reads that list rather than restating it.

use indexwerk_checks::terms::{Class, EXCLUDED};
use indexwerk_checks::{scan_text, scan_workspace};

fn classes_found(sample: &str) -> Vec<Class> {
    let mut classes: Vec<Class> = scan_text("fixture.rs", sample)
        .into_iter()
        .map(|finding| finding.class)
        .collect();
    classes.dedup();
    classes
}

#[test]
fn a_bind_off_loopback_is_refused() {
    let sample = "let listener = TcpListener::bind(\"0.0.0.0:8080\")?;";
    assert_eq!(classes_found(sample), vec![Class::OffLoopbackBind]);
}

#[test]
fn a_bind_to_loopback_is_not_refused() {
    // The compliant form of the same construct. If this ever fails, the check
    // has started refusing the thing the rule permits, which would push people
    // to disable it rather than to fix anything.
    let sample = "let listener = TcpListener::bind(\"127.0.0.1:0\")?;";
    assert!(scan_text("fixture.rs", sample).is_empty());
}

#[test]
fn certificate_store_access_is_refused() {
    let sample = "    Command::new(\"certutil\").arg(\"-addstore\").status()?;";
    assert_eq!(classes_found(sample), vec![Class::CertificateStore]);
}

#[test]
fn service_installation_is_refused() {
    let sample = "    Command::new(\"sc.exe\").args([\"create\", \"indexwerk\"]).status()?;";
    assert_eq!(classes_found(sample), vec![Class::ServiceInstall]);
}

#[test]
fn elevation_is_refused() {
    let sample = "    Command::new(\"powershell\").arg(\"Start-Process -Verb RunAs\").status()?;";
    assert_eq!(classes_found(sample), vec![Class::Elevation]);
}

#[test]
fn a_finding_names_the_file_and_the_line() {
    let sample = "// first line\n// second line\nlet l = TcpListener::bind(\"0.0.0.0:1\")?;\n";
    let findings = scan_text("crates/indexwerk-core/src/lib.rs", sample);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "crates/indexwerk-core/src/lib.rs");
    assert_eq!(findings[0].line, 3);
    let rendered = findings[0].to_string();
    assert!(rendered.starts_with("crates/indexwerk-core/src/lib.rs:3: "));
    // The message says where the rule comes from, so somebody meeting a red
    // check learns why rather than how to silence it.
    assert!(rendered.contains("issue #17"));
}

#[test]
fn every_class_has_at_least_one_term_that_bites() {
    // A class with no working term would be a rule that is written down and
    // enforces nothing, which is the shape this project refuses.
    for (class, sample) in [
        (Class::OffLoopbackBind, "bind(\"0.0.0.0:0\")"),
        (Class::CertificateStore, "CertOpenStore(...)"),
        (Class::ServiceInstall, "New-Service -Name x"),
        (Class::Elevation, "AdjustTokenPrivileges(...)"),
    ] {
        let findings = scan_text("fixture.rs", sample);
        assert!(
            findings.iter().any(|finding| finding.class == class),
            "no term bites for {}",
            class.name()
        );
    }
}

#[test]
fn the_exclusion_list_is_exactly_the_two_files_that_hold_the_literals() {
    // A third entry here would be a hole in the scan, so it is asserted rather
    // than trusted. Widening it means editing this assertion in the same diff.
    assert_eq!(
        EXCLUDED,
        [
            "crates/indexwerk-checks/src/terms.rs",
            "crates/indexwerk-checks/tests/bites.rs",
        ]
    );
}

#[test]
fn this_file_would_be_refused_if_it_were_not_excluded() {
    // The proof that the exclusion is load bearing rather than decorative: the
    // text of this very file, fed to the scanner under a name that is not
    // excluded, is refused.
    let own_text = include_str!("bites.rs");
    let findings = scan_text("crates/indexwerk-core/src/pretend.rs", own_text);
    assert!(
        !findings.is_empty(),
        "the fixtures in this file no longer trip the check"
    );
    // And the real scan, which honours the exclusion, does not report it.
    assert!(
        scan_workspace()
            .iter()
            .all(|finding| finding.file != "crates/indexwerk-checks/tests/bites.rs"),
        "the exclusion is not being honoured"
    );
}
