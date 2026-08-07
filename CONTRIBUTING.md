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
