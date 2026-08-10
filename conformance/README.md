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

The concrete syntax of the expressions is settled by
[`../docs/normal-form.md`](../docs/normal-form.md) rather than by this readme,
because it is the same syntax that document uses to quote a case. That document
names where the grammar is written rather than carrying a second copy of it, for
the reason a syntax described in two places drifts.

## The order fixtures

`order/` is the other kind of file here, and it is not a vector file. It holds
the orderings of [`../docs/normal-form.md`](../docs/normal-form.md) written out
as data: a list in ascending order, which an implementation of that order is
sorted against. It exists because an order stated only in prose is an order two
people read two ways, and because the document quoting a fixture and the suite
reading a different one is the drift both are meant to prevent.

`order/permutations.txt` is the first of them, quoted by section 7 of that
document.

## What reads it

The test suite reads every vector file in this directory and fails on any
mismatch between the expected output and what the canonicaliser produces. That
test is owed by [#6](https://github.com/iderex/indexwerk/issues/6) along with
the vectors themselves.

A second reader is owed by [#8](https://github.com/iderex/indexwerk/issues/8),
which runs the same set at several thread counts and compares the outputs byte
for byte.

Neither reader exists today, and neither does a vector file. So a green test run
says nothing at all about conformance yet.

The order fixtures are read separately, by the crate that implements the order
they describe. `order/permutations.txt` is owed a reader by
[#20](https://github.com/iderex/indexwerk/issues/20), which is where the
permutation type and its order are built. That reader is a different one from
the two above and it says nothing about conformance either: it compares an order
against a written order, not a canonicaliser against an expected output.
