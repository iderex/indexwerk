# The hardware-bound harness

Some things worth measuring cannot be measured in the gate. A shared runner has
an unknown core count, an unknown memory bandwidth and an unknown neighbour, it
carries whatever processor features the pool happened to give it, and it holds
no licence for a closed product. A measurement taken there is either flaky or
dishonest, and neither is worth having.

So those legs live here instead, outside the workspace, out of the default test
command and off the required check list. This directory is named for what it
needs rather than for what it is: it is not an integration test suite, because
what separates it from the ordinary suite is hardware and licences.

Nothing here has been measured yet. The engine the legs would measure does not
exist, so every leg below is a declaration of what it will require, not a
result. What does exist is the part that decides whether a result may be
printed at all, which is the subject of the section after the table.

## The legs, and what each one needs

This table is the authority. `crates/indexwerk-checks` parses it, refuses a row
whose requirement kind is not one of the four below and refuses a row with an
empty requirement, and the ordinary suite reports every row in it as a leg that
did not run. A leg added here without a report reds that suite, and a report
naming a leg that is not here reds it too.

| Leg | Kind of requirement | What it requires | How to run it |
| --- | --- | --- | --- |
| `parallel-scaling` | core count | a machine whose core count, memory size and processor model are recorded beside the number, held by nobody else while it runs | `cargo run --manifest-path harness/Cargo.toml -- run parallel-scaling` |
| `feature-gated-paths` | processor feature | hardware carrying the processor feature the path is selected by, named in the result, so the path the gate runner lacks is exercised somewhere | `cargo run --manifest-path harness/Cargo.toml -- run feature-gated-paths` |
| `large-memory-cases` | memory | memory beyond what a shared runner has, declared per case before the case runs rather than discovered when it is killed | `cargo run --manifest-path harness/Cargo.toml -- run large-memory-cases` |
| `closed-product-comparison` | external licence | a licence for the closed product the comparison is against, and the one machine both sides ran on | `cargo run --manifest-path harness/Cargo.toml -- run closed-product-comparison` |

The four kinds are the four in issue #18: core count, processor feature, memory,
and an external licence. A fifth kind is a change to the parser as well as to
this table, which is what keeps the column from becoming free prose.

`parallel-scaling` is the leg issue #33 asks for. `closed-product-comparison` is
the one issue #32 asks for, and it is the leg that carries the distinction
between a figure this project measured and a figure it quoted: where no licence
is available the closed side stays labelled published, and the result says which
side was measured here.

## A result that cannot be produced again is refused

The rule this harness exists for is that a number means nothing on its own. What
it has to carry with it is the command that produced it, the revision it was
produced at, the machine it ran on, the versions of everything compared, how many
times it ran and how far those runs spread. Those are the rules issue #31 fixes,
and this is the half of them a machine can hold.

`render` in `src/lib.rs` returns a refusal and prints nothing where any of them
is missing. There is no second path and no flag that skips the checks. The set is
the `Refused` enum in that file rather than a list here, because a list in a
document drifts against the thing it describes.

Two of those refusals are sharper than a blank check and are worth knowing about
before a result is written. A revision has to look like an object name: `main`
and `v0.1` are refused, because both move and the number does not move with them.
And a single run is refused, because a spread cannot be taken across one sample,
which is #31's sentence rather than a threshold chosen here.

The refusals are exercised in `tests/refusal.rs`, one test per refusal. Each was
proved by deleting the clause it names and watching that test, and only that
test, turn red.

Because no leg produces a result yet, the refusal is the whole of what this
crate does today. That is the honest state and it is stated rather than dressed
up: running any leg exits non-zero and says the measurement does not exist.

## Why it is not in the workspace

The root `Cargo.toml` excludes this directory, so `cargo test --workspace` and
`cargo build --workspace` cannot reach it. That is a stronger separation than a
convention about which tests to run, and it is checkable in one command:

    cargo metadata --format-version 1 --no-deps

The cost of the separation is stated because it is real. The gate does not
compile this crate, so a change to the workspace that breaks it is not caught
until somebody builds it here. Its own suite is what catches that, and running
it is the line at the top of the table's last column.

The greppable invariants of `docs/invariants.md` do not read this directory
either. Their scopes name `crates/`, `docs/` and `README.md`, and none of the
three reaches `harness/`. The headless and unelevated requirement is the one
that matters most here, because this is the directory whose legs are allowed to
want hardware: they are allowed to want cores, features, memory and a licence,
and they are still not allowed to want elevation. Nothing scans this directory
for that today, and that gap is real rather than covered.
