# The normal form

This document is a placeholder. It does not specify the normal form, and
nothing may be quoted from it as if it did.

The specification is owed by
[#6](https://github.com/iderex/indexwerk/issues/6), which fixes five things: the
total order on index names, the order on tensor factors within a monomial, where
free indices sit relative to dummy pairs, how variance is ordered, and where the
overall sign is placed. It is written and the conformance vectors are filled in
before the canonicaliser is written, which is the decision recorded in
[0004](adr/0004-normal-form-first.md).

The file exists now so that the documents pointing here point at something. The
first of those is [`../conformance/README.md`](../conformance/README.md), which
describes the vector file this document will define the syntax of.

Until #6 lands, the authority for the normal form is nowhere, and code that
needs one is code that is being written too early.
