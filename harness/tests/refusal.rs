//! What this harness refuses, one test per refusal.
//!
//! The point of the harness is that a number carries everything a reader needs
//! in order to produce it again. That is a property of this code rather than of
//! whoever reads the output, so each clause of it is a test here, and each test
//! is written so that deleting the clause it names turns it red rather than
//! leaving it passing because some other clause fired first.
//!
//! Order matters for that last part. `render` checks its clauses in a fixed
//! order, so a fixture that is blank in two places proves only the earlier
//! check. Every fixture below is complete apart from the one field its test is
//! about.

use indexwerk_harness::{
    Compared, FEWEST_REPETITIONS, Machine, Measurement, Outcome, Refused, dispatch, render,
};

fn a_machine() -> Machine {
    Machine {
        description: "16 cores, 64 GiB, an ordinary desktop under an ordinary user".to_owned(),
    }
}

fn a_measurement() -> Measurement {
    Measurement {
        leg: "parallel-scaling".to_owned(),
        value: "a speedup of 11.4 at 16 threads".to_owned(),
        command: "cargo run --manifest-path harness/Cargo.toml -- run parallel-scaling".to_owned(),
        revision: "0755432b19a392a88596387cb23ae39428cb9765".to_owned(),
        repetitions: 30,
        spread: "11.1 to 11.6 across the runs".to_owned(),
        compared: vec![Compared {
            name: "indexwerk".to_owned(),
            version: "0.0.0".to_owned(),
        }],
    }
}

#[test]
fn a_complete_measurement_renders_and_carries_every_field() {
    let rendered = render(&a_machine(), &a_measurement()).expect("a complete measurement renders");
    for expected in [
        "machine: 16 cores, 64 GiB",
        "command: cargo run --manifest-path harness/Cargo.toml -- run parallel-scaling",
        "revision: 0755432b19a392a88596387cb23ae39428cb9765",
        "indexwerk 0.0.0",
        "runs: 30, spread 11.1 to 11.6 across the runs",
        "result: a speedup of 11.4 at 16 threads",
    ] {
        assert!(
            rendered.contains(expected),
            "a refusal that does not reach the output is a check about a struct rather than \
             about a published number; {expected:?} is missing from:\n{rendered}"
        );
    }
}

#[test]
fn a_measurement_with_no_command_is_refused() {
    let mut unrepeatable = a_measurement();
    unrepeatable.command = "   ".to_owned();
    assert_eq!(render(&a_machine(), &unrepeatable), Err(Refused::NoCommand));
}

#[test]
fn a_measurement_with_no_revision_is_refused() {
    let mut floating = a_measurement();
    floating.revision = String::new();
    assert_eq!(render(&a_machine(), &floating), Err(Refused::NoRevision));
}

/// The mistake somebody makes when the number is wanted quickly: a branch name
/// where a revision belongs. It reads as an answer and it moves.
#[test]
fn a_revision_that_is_a_name_rather_than_an_object_is_refused() {
    for moving in [
        "main",
        "v0.1.0",
        "HEAD",
        "0755432b19a392a88596387cb23ae39428cb976z",
    ] {
        let mut named = a_measurement();
        named.revision = moving.to_owned();
        assert_eq!(
            render(&a_machine(), &named),
            Err(Refused::RevisionThatMoves),
            "{moving} is not an object name"
        );
    }
}

/// The short object name is accepted, because refusing it would refuse what a
/// person actually pastes.
#[test]
fn a_short_object_name_is_accepted() {
    let mut short = a_measurement();
    short.revision = "0755432".to_owned();
    assert!(render(&a_machine(), &short).is_ok());
    short.revision = "075543".to_owned();
    assert_eq!(
        render(&a_machine(), &short),
        Err(Refused::RevisionThatMoves),
        "one character shorter is no longer distinguishable from a name"
    );
}

#[test]
fn a_single_run_is_refused_because_it_is_not_a_measurement() {
    let mut once = a_measurement();
    once.repetitions = 1;
    assert_eq!(render(&a_machine(), &once), Err(Refused::TooFewRepetitions));
    once.repetitions = FEWEST_REPETITIONS;
    assert!(
        render(&a_machine(), &once).is_ok(),
        "the floor itself is accepted, so the refusal is about a single run rather than about \
         a number somebody preferred"
    );
}

#[test]
fn a_central_value_with_no_spread_is_refused() {
    let mut alone = a_measurement();
    alone.spread = String::new();
    assert_eq!(render(&a_machine(), &alone), Err(Refused::NoSpread));
}

#[test]
fn a_measurement_with_no_machine_description_is_refused() {
    let nameless = Machine {
        description: String::new(),
    };
    assert_eq!(
        render(&nameless, &a_measurement()),
        Err(Refused::NoMachineDescription)
    );
}

#[test]
fn a_machine_description_of_whitespace_is_refused_like_an_empty_one() {
    let nearly = Machine {
        description: "   \t  ".to_owned(),
    };
    assert_eq!(
        render(&nearly, &a_measurement()),
        Err(Refused::NoMachineDescription)
    );
}

#[test]
fn a_measurement_comparing_nothing_is_refused() {
    let mut versionless = a_measurement();
    versionless.compared.clear();
    assert_eq!(
        render(&a_machine(), &versionless),
        Err(Refused::NothingCompared)
    );
}

#[test]
fn a_measurement_with_no_leg_is_refused() {
    let mut anonymous = a_measurement();
    anonymous.leg = "  ".to_owned();
    assert_eq!(render(&a_machine(), &anonymous), Err(Refused::NoLeg));
}

#[test]
fn a_measurement_with_no_value_is_refused() {
    let mut empty = a_measurement();
    empty.value = String::new();
    assert_eq!(render(&a_machine(), &empty), Err(Refused::NoValue));
}

#[test]
fn every_refusal_says_why_rather_than_only_that() {
    for refusal in [
        Refused::NoLeg,
        Refused::NoValue,
        Refused::NoCommand,
        Refused::NoRevision,
        Refused::RevisionThatMoves,
        Refused::TooFewRepetitions,
        Refused::NoSpread,
        Refused::NoMachineDescription,
        Refused::NothingCompared,
    ] {
        assert!(
            refusal.reason().len() > 20,
            "{refusal:?} has to say what is missing, not only that something is"
        );
    }
}

#[test]
fn running_a_declared_leg_refuses_because_there_is_nothing_to_measure() {
    let outcome = dispatch(&["run".to_owned(), "parallel-scaling".to_owned()]);
    match outcome {
        Outcome::Refuse(message, code) => {
            assert_eq!(code, 1, "a declared leg with no measurement exits 1");
            assert!(
                message.contains("does not exist in this tree yet"),
                "the refusal has to say why, got: {message}"
            );
        }
        other => panic!("a declared leg must not report a result: {other:?}"),
    }
}

#[test]
fn running_an_undeclared_leg_is_a_different_refusal() {
    let outcome = dispatch(&["run".to_owned(), "not-a-leg".to_owned()]);
    match outcome {
        Outcome::Refuse(message, code) => {
            assert_eq!(code, 2, "an undeclared leg is a usage error");
            assert!(
                message.contains("not a declared leg"),
                "the refusal has to name the table, got: {message}"
            );
        }
        other => panic!("an undeclared leg must not be accepted: {other:?}"),
    }
}

#[test]
fn listing_prints_the_declared_legs_and_what_they_need() {
    match dispatch(&[]) {
        Outcome::Print(text) => {
            for leg in [
                "parallel-scaling",
                "feature-gated-paths",
                "large-memory-cases",
                "closed-product-comparison",
            ] {
                assert!(text.contains(leg), "the list has to name {leg}");
            }
            assert!(
                text.contains("external licence"),
                "the list has to say what each leg requires, not only its name"
            );
        }
        other => panic!("listing must print: {other:?}"),
    }
}
