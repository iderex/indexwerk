# 0004. The normal form is specified before it is implemented

Status: accepted.

Issue: [#6](https://github.com/iderex/indexwerk/issues/6).

## The decision

A document fixes the normal form before the canonicaliser is written. It fixes
five things: the total order on index names, the order on tensor factors within
a monomial, where free indices sit relative to dummy pairs, how variance is
ordered, and where the overall sign is placed.

Alongside it lives a machine-readable file of conformance vectors, each one an
input expression and its canonical output, which any implementation can be run
against.

The document and the vectors exist before the canonicaliser exists, and the
document is the authority when the two disagree with the code.

## The reasons

### Two canonicalisers can both be correct and disagree

A canonical form is a choice of representative, and different systems choose
differently. Without a written normal form the differential testing in M6 cannot
tell a bug from a convention difference, and every disagreement with SymPy or
Cadabra turns into an argument about which one is right instead of a verdict
about this implementation.

### The vectors are what make the specification refusable

A document alone is prose. A file of inputs and expected outputs that the test
suite reads is a thing the build fails on, and the difference between those two
states is the difference between a rule and an explanation of one.

### A translation layer needs both forms written down

Comparing against another system means mapping its normal form onto this one.
That mapping is a small, testable piece of code once both forms are written
down, and it cannot be written correctly against an unwritten one.

## What the vector file has to be

The format is fixed here because the directory that holds the vectors is created
before the vectors are, and a directory whose readme cannot say what it will
hold is a directory that invites a guess.

Line-oriented UTF-8 text, one case per record, each record carrying a case
identifier, the input expression and the expected canonical output. Comment
lines are permitted and are what a case uses to say what it is for. Nothing else
is in the file: no generated timestamps, no counts, no ordering that a writer
has to maintain by hand.

Text rather than a binary or a serialised object graph, because a change to a
vector has to be readable in a diff by somebody who is deciding whether the
normal form moved or a typo landed.

Nothing in a vector is byte-exact in the sense that the line-ending policy is
about. An index expression contains no carriage return and no trailing
whitespace that carries meaning, so the vectors are ordinary text under
`.gitattributes` and need no exemption from it. The comment in that file
reserving the question for this record is answered by this paragraph, and the
answer is that no pattern is owed.

The concrete syntax of the input and output expressions is not fixed here. It is
fixed in the normal form document itself, because it is the same syntax that
document uses to quote a case, and splitting the two across two files is how
they drift.

## The consequence for the code

Changing the normal form after the first release is a breaking change and is
treated as one. The vectors file carries a version, and a change to the form
moves that version in the same change that moves the vectors.

## What follows from this record elsewhere

The document and the vectors are written in
[#6](https://github.com/iderex/indexwerk/issues/6) and
[#28](https://github.com/iderex/indexwerk/issues/28). The total order on
permutations in M4 refers to the order this document fixes rather than defining
its own, so [#20](https://github.com/iderex/indexwerk/issues/20) reads the same
fixture the document quotes.
