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
harness in `harness/` and never into the default suite.

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

The scan reads `harness/` as well, for this rule and for no other. The legs
there are allowed to want cores, a processor feature, memory and a licence, and
they are not allowed to want elevation, so a test moved out of the default suite
into the harness meets the same refusal it met before it moved. That was a
sentence rather than a check until the scope was widened, and the fixtures
proving it fires there are in `crates/indexwerk-checks/tests/bites.rs`.

The other invariants stop at `crates/`, `docs/` and `README.md`. The harness is
not a shipped crate and nothing in it reaches a consumer, so the unsafe, the
floating point, the egress and the panic-path rules are not about it. The
documentation rule is a different case: `harness/README.md` is neither
`README.md` nor under `docs/`, so a performance number written there is read by
nothing, and that gap is real rather than covered.

## The hardware-bound harness

Some things worth measuring cannot be measured on a shared runner without the
number becoming either flaky or dishonest: how the work scales with core count,
a path selected by a processor feature the runner lacks, a case that needs more
memory than the runner has, and any comparison against a product that needs a
licence. Those legs live in `harness/`, which is a package in this tree and
deliberately not a member of the workspace.

That is what keeps them out of the default test command. `cargo test
--workspace` cannot reach a package the workspace excludes, so the separation
holds whether or not anybody remembers it:

    cargo metadata --format-version 1 --no-deps

`harness/README.md` is the list. One row per leg, saying what the leg requires
before it says what it measures, and the requirement kind is one of the four the
harness is for rather than free prose. That table is parsed by
`crates/indexwerk-checks`, so a row with an invented requirement kind, an empty
requirement, a duplicate identifier or a command belonging to something other
than the harness reds the ordinary suite.

The ordinary suite also prints every leg in that table as one that did not run,
with what running it would require, so a green run cannot be read as covering
them.

Running the harness, and its own suite, from a clone:

    cargo run --manifest-path harness/Cargo.toml -- list
    cargo test --manifest-path harness/Cargo.toml --all-targets

Neither is run by the gate. That has a cost worth knowing about: a change to the
workspace that stops the harness compiling is not caught until somebody builds
it, and the second command above is what catches it.

A result the harness prints carries the machine it came from and the version of
everything it compared. That is enforced in `harness/src/lib.rs` rather than
left to whoever reads the output, and a measurement missing either is refused
and nothing is printed.

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

## Formatting and lint

Two check runs, separate from the build, so that a formatting failure does not
hide a compile failure. Both reproduce locally, and on a clean clone both
produce no output and exit zero.

The format leg:

    cargo fmt --all -- --check
    cargo test -p indexwerk-checks --locked --test formatting

The lint leg:

    cargo clippy --workspace --locked --all-targets -- -D warnings

Denied rather than warned. A lint that only warns is a lint nobody fixes, and
`--all-targets` reaches the tests, which are most of the code here.

### Why the second format command is not a formatter

`cargo fmt` reads Rust and nothing else, and this tree is mostly Markdown. The
rest of the format leg is `crates/indexwerk-checks`, in the tree rather than in
a workflow file, so a contributor runs exactly what the gate runs.

It is not a Markdown formatter and it does not reflow anything. It judges
whitespace: a tab, trailing blanks that are not the two spaces of a hard line
break, a line that looks empty and is not, a missing final newline, a blank line
at the end. Those are worth judging because they are invisible, which is also
why nothing else catches them.

Two things it deliberately does not judge, each because judging it would cost
more than it is worth.

Line endings. `.gitattributes` stores and checks out LF everywhere and is the
authority for it. A working tree can carry carriage returns while the committed
bytes are LF, because a checkout predating a rule in that file is not rewritten,
and a check that judged line endings would report exactly that clean tree as
failing. So a carriage return is removed from each line before the line is
judged. `format (windows)` runs the same check on the other platform, which is
how that is demonstrated rather than asserted.

Line length. This tree wraps prose at eighty columns by hand, and twenty lines
do not: a generated document, a paragraph written as one line on purpose, and
links that cannot be broken. Reflowing those is a different change with a
different argument.

### Python

There is no Python source in this tree and no Python formatter configured. Those
two facts are only consistent together, so the format leg lists the Python
sources and reds if it finds any, naming the files. The change that brings the
first `.py` file chooses a formatter for it and adds it to the leg and to the
commands above.

## The checks a change runs

`docs/required-checks.md` lists them by their exact names, with the workflow file
that produces each one, and says which ones are not required and why. The list is
not restated here, because a list in two places drifts.
