# The rules a performance number follows

The whole argument for this project is a performance claim, so the rules for
making one are written down before any number is published. They apply to
`README.md` as much as to a paper, and a claim made anywhere in this repository
points here.

## The rules

**Every number carries what produced it.** The command it came from, the
revision it was produced at, the machine it ran on, and the versions of
everything being compared. A number without those is not published, because a
number nobody can produce again is a story about a machine somebody once had.

**Measured and published are different words, and they are not
interchangeable.** A number this project produced on a machine it can describe
is measured. A number taken from somebody else's paper, port or README is
published, and it is labelled that way every time it appears, including in
`README.md`. The distinction is not a formality: the second kind carries a
machine, a version and a workload this project did not choose and cannot
inspect.

**A comparison states what was compared.** Timing a compiled library against an
interpreted one is fair only where the same computation is on both sides, so a
result says what was inside the measurement and what was not: parsing, symmetry
setup, the canonicalisation itself, and the marshalling across a language
boundary, each of them separately rather than as one figure.

**Repetition and spread are stated rather than assumed.** How many times the
work ran, whether it was warmed up first, and how far the runs spread around the
central value. A single run is not a measurement, and a central value with no
spread beside it hides whether the machine was busy.

**The gate publishes no performance number.** A shared runner has an unknown
core count, an unknown neighbour and an unknown memory bandwidth, so a number
taken there is either flaky or dishonest. Where a number needs hardware or a
licence, it comes from the harness in `harness/`, which is outside the workspace
for that reason.

## Which half of this a machine holds

`render` in `harness/src/lib.rs` refuses a result that is missing any of the
fields above and prints nothing in its place. There is no second path and no
flag that skips it. The set of refusals is the `Refused` enum in that file
rather than a list here, because a list in a document drifts against the thing it
describes:

    git grep -n 'Refused::' -- harness/src/lib.rs | grep return

Two of them are sharper than a blank check. A revision has to look like an
object name, so a branch or a tag is refused: such a name moves and the number
does not move with it, and the pair stops being true without either being
edited. And a single run is refused rather than a repetition count somebody
preferred, which is the fourth rule above and not a threshold chosen in the
code.

The refusals are exercised in `harness/tests/refusal.rs`, and how each one was
proved to bite is in `harness/README.md` rather than repeated here:

    cargo test --manifest-path harness/Cargo.toml --all-targets

That suite is run by no leg of the gate, which `docs/required-checks.md` states
in the section on what reads each part of the tree. Running it is a line in
CONTRIBUTING.md and a person's job.

The other machine-held half is greppable rather than structural. One invariant
of `docs/invariants.md` refuses a performance number in tracked documentation
with no source nearby, over `README.md` and `docs/` and nothing else. It reads a
spelling rather than a parse tree and is crude on purpose, and `harness/README.md`
is outside its reach, which that file says of itself.

## What no machine here holds

Whether a comparison timed the same computation on both sides. Whether a machine
description says enough for somebody else to reproduce the number on comparable
hardware. Whether a figure labelled published really is somebody else's, and
whether a figure labelled measured really was produced by the command written
beside it. Those are judgements about meaning, and the review is where a wrong
one is caught.

## Where the numbers in this tree stand

`README.md` carries the only performance figures in this repository, and they
are labelled published rather than measured: they are the figures of the author
of the port they describe, quoted here. Nothing in this repository has been
measured by it. The legs that would produce a measured number are declared in
`harness/README.md` and none of them has run, and the ordinary suite reports
every one of them as a leg that did not run.

So the word published appears in this tree next to a number and the word
measured does not yet. That asymmetry is the current state rather than an
oversight, and the first genuinely measured number is what removes it.

## What this document does not settle

Which central value a result reports, how long a warm-up is, and how many
repetitions are enough beyond the floor of more than one. Those belong to the
leg that produces the first measurement, argued where that leg is written, and a
number chosen here in advance would be a rule with no run behind it.
