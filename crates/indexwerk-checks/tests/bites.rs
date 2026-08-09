//! Proof that the check bites, one invariant at a time.
//!
//! This file is excluded from the scan it exercises, for the same reason the
//! term table is: it has to contain the constructs to feed them to the scanner.
//! The exclusion is by name, in `terms::EXCLUDED`, and a test at the bottom
//! reads that list rather than restating it.
//!
//! Every fixture is fed to `scan_text` under a path that decides the scope, so
//! each invariant is proved twice over: it fires where it reaches, and it does
//! not fire where it does not. The second half is the one that would otherwise
//! rot, because an invariant that quietly widened its reach would still pass a
//! test that only asked whether it fires.

use indexwerk_checks::terms::{Class, EXCLUDED, INVARIANTS, Invariant};
use indexwerk_checks::{catalogue_markdown, scan_text, scan_workspace};

const CORE: &str = "crates/indexwerk-core/src/lib.rs";
const FFI: &str = "crates/indexwerk-ffi/src/lib.rs";
const PYTHON: &str = "crates/indexwerk-python/src/lib.rs";
const CHECKS: &str = "crates/indexwerk-checks/src/walk.rs";
const HARNESS: &str = "harness/tests/refusal.rs";
const DOC: &str = "docs/benchmarks.md";

fn classes(path: &str, sample: &str) -> Vec<Class> {
    let mut found: Vec<Class> = scan_text(path, sample)
        .into_iter()
        .map(|finding| finding.class)
        .collect();
    found.dedup();
    found
}

fn invariants(path: &str, sample: &str) -> Vec<Invariant> {
    let mut found: Vec<Invariant> = scan_text(path, sample)
        .into_iter()
        .map(|finding| finding.invariant())
        .collect();
    found.dedup();
    found
}

// No unsafe code outside the one declared crate.

#[test]
fn an_unsafe_block_outside_the_declared_crate_is_refused() {
    let sample = "    let value = unsafe { *pointer };";
    assert_eq!(classes(CORE, sample), vec![Class::UnsafeConstruct]);
}

#[test]
fn an_unsafe_attribute_outside_the_declared_crate_is_refused() {
    let sample = "#[unsafe(no_mangle)]\npub extern \"C\" fn thing() {}\n";
    assert_eq!(classes(PYTHON, sample), vec![Class::UnsafeConstruct]);
}

#[test]
fn the_same_unsafe_inside_the_declared_crate_is_not_refused() {
    // The near-miss worth spending on. The exception granted by
    // docs/adr/0005-layering.md is one crate wide, and a check that refused it
    // everywhere would refuse the C interface itself.
    let sample = "#[unsafe(no_mangle)]\npub extern \"C\" fn thing() { unsafe { core() } }\n";
    assert!(scan_text(FFI, sample).is_empty());
}

#[test]
fn the_forbid_attribute_does_not_trip_the_unsafe_terms() {
    // The line that enforces the rule contains the word the rule is about. If
    // this ever fails, every crate that obeys the rule is refused for obeying
    // it, which is the shape that teaches people to delete the check.
    let sample = "#![forbid(unsafe_code)]\n";
    assert!(scan_text(CORE, sample).is_empty());
}

// No floating point in the core.

#[test]
fn a_float_in_the_core_is_refused() {
    let sample = "    let coefficient: f64 = 0.5;";
    assert_eq!(classes(CORE, sample), vec![Class::FloatingPointType]);
}

#[test]
fn a_float_outside_the_core_is_not_refused_by_this_invariant() {
    // The record is about the core. The layers above it marshal whatever the
    // application hands them, and refusing a float there would be a different
    // rule that nobody has argued for.
    let sample = "    let coefficient: f64 = 0.5;";
    assert!(!invariants(PYTHON, sample).contains(&Invariant::NoFloatingPointInTheCore));
}

#[test]
fn a_name_that_merely_ends_in_a_float_type_is_not_refused() {
    // Token matching rather than substring, so `buf64` is not arithmetic.
    let sample = "    let buf64 = make_buffer(); let xf32y = 1;";
    assert!(scan_text(CORE, sample).is_empty());
}

// No egress from a shipped crate.

#[test]
fn a_socket_in_a_shipped_crate_is_refused() {
    let sample = "    let stream = TcpStream::connect(address)?;";
    assert_eq!(classes(CORE, sample), vec![Class::Egress]);
}

#[test]
fn a_name_resolution_in_a_shipped_crate_is_refused() {
    let sample = "    let addresses = host.to_socket_addrs()?;";
    assert_eq!(classes(PYTHON, sample), vec![Class::Egress]);
}

#[test]
fn a_process_spawn_in_a_shipped_crate_is_refused() {
    let sample = "    Command::new(\"git\").arg(\"fetch\").status()?;";
    assert_eq!(classes(FFI, sample), vec![Class::Egress]);
}

#[test]
fn the_same_process_spawn_in_the_checks_crate_is_not_egress() {
    // The checks crate ships to nobody, so the offline guarantee of
    // docs/adr/0008-nothing-leaves-the-host.md is not about it. The bound is
    // stated here rather than left for somebody to infer from a green run.
    let sample = "    let output = Command::new(\"git\").arg(\"status\").output()?;";
    assert!(!invariants(CHECKS, sample).contains(&Invariant::NoEgressFromAShippedCrate));
}

// No panic path in a library crate.

#[test]
fn an_unwrap_in_a_library_source_is_refused() {
    let sample = "    let value = maybe.unwrap();";
    assert_eq!(classes(CORE, sample), vec![Class::PanicPath]);
}

#[test]
fn a_panic_macro_in_a_library_source_is_refused() {
    let sample = "    panic!(\"the slot count did not match\");";
    assert_eq!(classes(CHECKS, sample), vec![Class::PanicPath]);
}

#[test]
fn a_panic_inside_a_test_module_is_not_refused() {
    // A panic in a test is how a test reports. Refusing it would push the
    // suite towards reporting failure some other way, which is worse than the
    // thing being refused.
    let sample = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        \
                  let v = maybe.unwrap();\n    }\n}\n";
    assert!(scan_text(CORE, sample).is_empty());
}

#[test]
fn a_panic_after_a_test_module_closes_is_refused_again() {
    // The near-miss the brace counting exists for: a region that never ends
    // would silence every panic written below the first test module in a file.
    let sample = "#[cfg(test)]\nmod tests {\n    fn t() {\n        let v = a.unwrap();\n    \
                  }\n}\n\npub fn later() {\n    let w = b.unwrap();\n}\n";
    let findings = scan_text(CORE, sample);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 9);
}

#[test]
fn an_admitted_abort_is_not_refused() {
    let sample = "    // aborts on a violated internal invariant: the table is built \
                  above\n    let entry = table.get(id).unwrap();\n";
    assert!(scan_text(CORE, sample).is_empty());
}

#[test]
fn the_admission_does_not_reach_past_the_statement_it_admits() {
    // A marker written once at the top of a file must not admit everything
    // below it, or the named list stops being a list.
    let sample = "// aborts on a violated internal invariant: stated once, wrongly\n\n\
                  pub fn one() {\n    let a = x.unwrap();\n}\n";
    let findings = scan_text(CORE, sample);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4);
}

// Headless and unelevated.

#[test]
fn a_bind_off_loopback_is_refused() {
    let sample = "let listener = TcpListener::bind(\"0.0.0.0:8080\")?;";
    assert!(classes(CHECKS, sample).contains(&Class::OffLoopbackBind));
}

#[test]
fn a_bind_to_loopback_is_not_refused() {
    // The compliant form of the same construct. If this ever fails, the check
    // has started refusing the thing the rule permits, which would push people
    // to disable it rather than to fix anything.
    let sample = "let listener = Listener::bind(\"127.0.0.1:0\")?;";
    assert!(scan_text(CHECKS, sample).is_empty());
}

#[test]
fn certificate_store_access_is_refused() {
    let sample = "    let store = X509Store::open()?;";
    assert_eq!(classes(CHECKS, sample), vec![Class::CertificateStore]);
}

#[test]
fn service_installation_is_refused() {
    let sample = "    run(\"schtasks /create /tn indexwerk\")?;";
    assert_eq!(classes(CHECKS, sample), vec![Class::ServiceInstall]);
}

#[test]
fn elevation_is_refused() {
    let sample = "    run(\"powershell Start-Process -Verb RunAs\")?;";
    assert_eq!(classes(CHECKS, sample), vec![Class::Elevation]);
}

#[test]
fn elevation_in_the_harness_is_refused_too() {
    // The near-miss worth spending on. A leg that wants hardware belongs in
    // `harness/`, which is the one directory in this tree allowed to want
    // something the gate cannot give it. Elevation is not on that list, and a
    // test moved there to get around a red check is the move this leg refuses.
    let sample = "    run(\"powershell Start-Process -Verb RunAs\")?;";
    assert_eq!(classes(HARNESS, sample), vec![Class::Elevation]);
}

#[test]
fn a_bind_off_loopback_in_the_harness_is_refused_too() {
    let sample = "let listener = TcpListener::bind(\"0.0.0.0:8080\")?;";
    assert!(classes(HARNESS, sample).contains(&Class::OffLoopbackBind));
}

#[test]
fn a_service_install_in_the_harness_is_refused_too() {
    let sample = "    run(\"schtasks /create /tn indexwerk\")?;";
    assert_eq!(classes(HARNESS, sample), vec![Class::ServiceInstall]);
}

#[test]
fn a_certificate_store_in_the_harness_is_refused_too() {
    let sample = "    let store = X509Store::open()?;";
    assert_eq!(classes(HARNESS, sample), vec![Class::CertificateStore]);
}

#[test]
fn a_bind_to_loopback_in_the_harness_is_not_refused() {
    let sample = "let listener = Listener::bind(\"127.0.0.1:0\")?;";
    assert!(scan_text(HARNESS, sample).is_empty());
}

#[test]
fn the_other_invariants_stop_at_the_crates_and_do_not_read_the_harness() {
    // The half that would otherwise rot. Widening one invariant to a directory
    // must not widen the other five, and the harness is deliberately outside
    // them: it is not a shipped crate, it is not the core, and nothing in it
    // reaches a consumer. A fixture that breaks four of them at once is silent
    // under this path and refused under a crate path.
    let sample = "use std::net::TcpListener;\nlet x: f64 = unsafe { table.get(0).unwrap() };\n";
    assert!(scan_text(HARNESS, sample).is_empty(), "{sample}");
    let under_a_crate = invariants(CORE, sample);
    assert!(under_a_crate.contains(&Invariant::NoEgressFromAShippedCrate));
    assert!(under_a_crate.contains(&Invariant::NoFloatingPointInTheCore));
    assert!(under_a_crate.contains(&Invariant::NoUnsafeOutsideTheDeclaredCrate));
    assert!(under_a_crate.contains(&Invariant::NoPanicPathInALibraryCrate));
}

// No performance number without its source.

#[test]
fn a_time_figure_with_no_source_near_it_is_refused() {
    let sample = "# Speed\n\nThe canonicaliser takes 270 ms on a Riemann monomial.\n";
    assert_eq!(
        classes(DOC, sample),
        vec![Class::UnsourcedPerformanceNumber]
    );
}

#[test]
fn a_time_figure_with_a_command_near_it_is_not_refused() {
    let sample = "# Speed\n\n    cargo bench --bench riemann\n\nIt takes 270 ms.\n";
    assert!(scan_text(DOC, sample).is_empty());
}

#[test]
fn a_time_figure_labelled_as_somebody_elses_is_not_refused() {
    // The distinction #31 fixes. A number this project produced is measured and
    // carries its command; a number out of somebody else's paper is published
    // and is labelled that way every time it appears.
    let sample = "The port took 2400 ms, a figure published by its author rather than \
                  measured here.\n";
    assert!(scan_text(DOC, sample).is_empty());
}

#[test]
fn a_plain_number_in_documentation_is_not_a_performance_number() {
    // The crude check has to stay off ordinary prose or nobody will keep it.
    let sample = "There are 3 layers and 9 decision records, and the floor is 1.90.0.\n";
    assert!(scan_text(DOC, sample).is_empty());
}

#[test]
fn a_time_figure_in_a_source_file_is_not_this_invariant() {
    let sample = "// the benchmark took 270 ms\n";
    assert!(!invariants(CORE, sample).contains(&Invariant::NoPerformanceNumberWithoutItsSource));
}

// What a finding says, and what holds the set together.

#[test]
fn a_finding_names_the_file_the_line_and_where_the_rule_comes_from() {
    let sample = "// first line\n// second line\nlet v = TcpStream::connect(a)?;\n";
    let findings = scan_text(CORE, sample);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, CORE);
    assert_eq!(findings[0].line, 3);
    let rendered = findings[0].to_string();
    assert!(rendered.starts_with("crates/indexwerk-core/src/lib.rs:3: "));
    // The message says where the rule comes from, so somebody meeting a red
    // check learns why rather than how to silence it.
    assert!(rendered.contains("docs/adr/0008-nothing-leaves-the-host.md"));
    assert!(rendered.contains("issue #36"));
}

#[test]
fn every_invariant_names_a_source_and_bites_at_least_once() {
    // An invariant listed with no working term would be a rule that is written
    // down and enforces nothing, which is the shape this project refuses.
    for invariant in INVARIANTS {
        assert!(
            !invariant.source().is_empty(),
            "{} names no source",
            invariant.title()
        );
    }
    for (invariant, path, sample) in [
        (
            Invariant::NoUnsafeOutsideTheDeclaredCrate,
            CORE,
            "let v = unsafe { read() };",
        ),
        (Invariant::NoFloatingPointInTheCore, CORE, "let x: f32 = 1;"),
        (
            Invariant::NoEgressFromAShippedCrate,
            CORE,
            "use std::net::TcpListener;",
        ),
        (Invariant::NoPanicPathInALibraryCrate, CORE, "a.unwrap();"),
        (
            Invariant::HeadlessAndUnelevated,
            CHECKS,
            "bind(\"0.0.0.0:0\")",
        ),
        (
            Invariant::NoPerformanceNumberWithoutItsSource,
            DOC,
            "it took 5 ms",
        ),
    ] {
        assert!(
            invariants(path, sample).contains(&invariant),
            "no term bites for {}",
            invariant.title()
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

#[test]
fn the_rendered_catalogue_carries_every_invariant_with_its_source() {
    let rendered = catalogue_markdown();
    for invariant in INVARIANTS {
        assert!(
            rendered.contains(invariant.title()),
            "{} is not in the rendered catalogue",
            invariant.title()
        );
        assert!(
            rendered.contains(invariant.source()),
            "{} is listed without its source",
            invariant.title()
        );
    }
}
