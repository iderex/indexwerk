# 0006. Results do not depend on thread count

Status: accepted.

Issue: [#8](https://github.com/iderex/indexwerk/issues/8).

## The decision

For a given input and a given version, the canonical form is one value. It is
identical whether the run used one thread or sixteen, and identical between two
runs on the same machine.

Parallelism goes into the search inside the algorithm. The step that combines
partial results is a deterministic reduction with a fixed order, never
first-one-wins.

The rule is enforced by a test rather than by care. The test suite runs the
conformance vectors at several thread counts and compares the outputs byte for
byte.

## The reasons

### A canonicaliser whose answer varies is not a canonicaliser

The whole point of the operation is that two expressions are equal exactly when
their canonical forms are identical. A scheduling-dependent output destroys that
property silently, and it destroys it in the cases that are hardest to
reproduce, which are the large ones a user reaches after an hour of work.

### The speed argument makes this sharper rather than softer

Parallelism is the reason this project can beat the interpreted
implementations, so it will be used aggressively. That is precisely the setting
in which nondeterminism arrives, through unordered collection of partial results
and through racing early exits that take whichever answer finished first.

### A bug report has to be reproducible on a different machine

A user reporting a wrong canonical form from a sixteen-core machine must be
reproducible on a one-core machine, or the report cannot be acted on. Without
this decision the first question on every report is how many cores the reporter
had, and there is no good answer to it.

## The cost

Some parallel strategies are ruled out. Speculative search with a
first-answer-wins exit is the obvious one, and it would be faster.

The determinism is worth more than that. Where a nondeterministic strategy is
genuinely much faster it may be offered as an explicitly named non-canonical
fast path, which returns an equivalence witness rather than a canonical form, so
that a caller who takes it knows what they gave up.

## What follows from this record elsewhere

The test this record names reads the conformance vectors, which are fixed in
[0004](0004-normal-form-first.md) and written in
[#6](https://github.com/iderex/indexwerk/issues/6). Until those exist there is
nothing for it to read, so the test is owed by
[#8](https://github.com/iderex/indexwerk/issues/8) rather than delivered by this
record, and the gate does not run it today.
