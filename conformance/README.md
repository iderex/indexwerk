# Conformance vectors

Data, not prose. What is in this directory is read by the test suite, and it is
also what another implementation is run against to say whether it agrees with
this one's normal form.

The directory exists before the vectors do, created by
[#14](https://github.com/iderex/indexwerk/issues/14), so that the first vector
file does not also have to invent where vector files live.

## The format

Fixed in [`../docs/adr/0004-normal-form-first.md`](../docs/adr/0004-normal-form-first.md).

Line-oriented UTF-8 text, one case per record, each record carrying a case
identifier, the input expression and the expected canonical output. Comment
lines are permitted and are what a case uses to say what it is for. Nothing else
is in the file: no generated timestamps, no counts, and no ordering a writer has
to maintain by hand.

Text rather than a binary or a serialised object graph, because a change to a
vector has to be readable in a diff by somebody deciding whether the normal form
moved or a typo landed.

Nothing here is byte-exact in the sense the line-ending policy is about. An
index expression carries no carriage return and no meaningful trailing
whitespace, so these files are ordinary text under `.gitattributes` and are not
exempted from it.

The concrete syntax of the expressions is part of the normal form document
rather than of this readme, because it is the same syntax that document uses to
quote a case, and a syntax described in two places drifts.

## What reads it

The test suite reads every vector file in this directory and fails on any
mismatch between the expected output and what the canonicaliser produces. That
test is owed by [#6](https://github.com/iderex/indexwerk/issues/6) along with
the vectors themselves.

A second reader is owed by [#8](https://github.com/iderex/indexwerk/issues/8),
which runs the same set at several thread counts and compares the outputs byte
for byte.

Neither reader exists today, and neither does a vector file. This directory
holds this readme and nothing else, so a green test run says nothing at all
about conformance yet.
