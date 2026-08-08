# Contributing

## Headless and unelevated, as a birth requirement

Every test this project plans runs on a machine with no display attached and
under an ordinary user account. This is a birth requirement rather than
something to fix later, and a change that makes the suite need elevation is a
defect rather than a step to document.

Three things follow from it.

No test opens a window, needs a graphical session, or depends on a display
server being present. Nothing planned here has a user interface, so the rule
costs nothing today and a great deal later, on the day a plotting dependency
arrives with a windowing requirement attached.

No test needs elevation. That means no test binds a listening socket on anything
but loopback, writes into a certificate store, installs a service, or shells out
to a tool that asks for administrator rights. Binding off loopback is the one
that catches people out, because on Windows it raises a firewall consent dialog
that only an administrator can answer, and the answer covers one executable path
rather than the project, so every new build directory asks again.

No test needs the network. That follows from the decision record on nothing
leaving the host, issue #10, and it is stated here as well because the test
suite is where it would first be violated.

Where a test genuinely needs one of these, it goes into the hardware-bound
harness described in issue #18 and never into the default suite.

### How the rule is checked

The rule is checked rather than trusted. `crates/indexwerk-checks` scans every
Rust source under `crates/` for the constructs that would break it and names the
file and the line when it fires. The search terms live in
`crates/indexwerk-checks/src/terms.rs`, in the check rather than in this
document, so widening them is a code change that shows up in a diff.

It runs as part of the ordinary suite:

    cargo test --workspace --locked

Two files are excluded from the scan, because a table of forbidden constructs
has to contain the constructs and a proof that it bites has to feed them in:
`crates/indexwerk-checks/src/terms.rs` and
`crates/indexwerk-checks/tests/bites.rs`. The exclusion list is asserted in
`bites.rs` rather than left to drift, and a test there feeds that file's own
text back to the scanner under a different name to show the exclusion is load
bearing rather than decorative.

## The toolchain, and the two floors

### The compiler you build with

`rust-toolchain.toml` pins it. rustup reads that file before its own default and
installs what it names on first use, so a clone builds with the same compiler
the gate builds with and nobody has to be told which one that is.

It is the only place the pinned version is written. Moving it is a one-line
change to that file.

### The oldest compiler this project supports

`rust-version` in the workspace manifest, and it is `1.85.0`.

That value is forced rather than preferred. The workspace is on edition 2024,
which no compiler below 1.85 accepts, so the floor cannot go lower while the
edition stands.

It is also not raised above the force, and that is the part worth stating,
because raising it is the easy direction. Every distribution that packages this
project ships whatever compiler its release carries, and a floor moved to pick
up a convenience feature locks those packagers out for as long as their release
lives. A language feature from the last three years is not worth that, and where
one genuinely is, the argument belongs in an issue rather than in a commit that
happens to need it.

Two gate legs exercise the floor, `build (floor toolchain)` and
`test (floor toolchain)`. Both read the value out of the workspace manifest, so
the floor that is declared and the floor that is exercised cannot drift apart.
Both also refuse to run if the compiler they ended up with is not the one the
manifest declares, which is not paranoia: `rust-toolchain.toml` outranks
`rustup default`, so a floor leg written the obvious way builds on the pinned
compiler and reports itself green as the floor.

Building on the floor locally:

    rustup toolchain install 1.85.0 --profile minimal
    RUSTUP_TOOLCHAIN=1.85.0 cargo test --workspace --locked

### The oldest Python the package supports

Python 3.10, and the wheels target the stable application binary interface at
that level, `abi3-py310`.

3.10 is the oldest version still receiving security fixes. 3.9 reached end of
life on 2025-10-31 and 3.10 does so in October 2026, dates published in the
Python developer guide's version table and quoted from it rather than measured
here. Supporting a version upstream has stopped fixing means shipping wheels for
an interpreter nobody should be running.

The stable-interface level is fixed here rather than in M7 because it is the
same number: a single wheel built against `abi3-py310` loads on 3.10 and every
version after it, so the floor and the interface level move together or the
wheel matrix stops making sense.

Neither number is declared in Python package metadata, because there is no
Python package metadata in this tree. `crates/indexwerk-python` is a Rust crate
with no `pyproject.toml`, no build backend and no binding code, and writing one
that cannot build a wheel would be a claim this tree cannot back. That half of
#12 is owed and is named there.

### Raising either floor

Raising the Rust floor or the Python floor needs an issue, argued and merged
like anything else. A floor is a promise to somebody who is not in the room when
the change is made, and the cost of breaking it lands on them rather than here.
Lowering one needs an issue for the same reason.

## Signing off your work

Every commit carries a `Signed-off-by:` trailer matching its author. Sign
with:

    git commit -s

The DCO check reads the whole commit range of a pull request and reds on any
commit that lacks a matching trailer. The text of the certificate itself is not
in this tree yet; the workflow that enforces it points at a file that has still
to be added.

## The checks a change runs

`docs/required-checks.md` lists them by their exact names, with the workflow file
that produces each one, and says which ones are not required and why. The list is
not restated here, because a list in two places drifts.
