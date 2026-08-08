# 0002. The means the core is made of

Status: accepted.

Issue: [#4](https://github.com/iderex/indexwerk/issues/4).

## The decision

The core is written in Rust, as a library crate with no input or output of its
own, exposing a C interface. The Python package is built on that interface.
Nothing in the core depends on Python being present.

## The reasons

### The properties have to be refusable by a machine

A property this project needs has to be something a machine refuses rather than
something a document asserts. Rust refuses whole classes at compile time, and
`#![forbid(unsafe_code)]` on every crate except the declared foreign-interface
one turns "the core contains no unsafe code" from a claim into a build that does
not produce a binary. That single line is worth more than a review policy,
because a review policy is applied by whoever is reading that day.

### Proof has to be executed, not described

One toolchain gives the unit tests, the property tests, the benchmark harness
and the fuzzing targets, with one command each and no separate apparatus to keep
alive. A suite assembled from four unrelated tools is a suite somebody stops
running, and the legs that stop first are the slow ones, which are the ones that
find things.

### The parallelism has to be real

The measured factor of nine has two parts, interpretation and the global
interpreter lock, and only a compiled language without a lock addresses the
second. The work is a search over a group, which is the shape that parallelises.
A means that leaves the second part in place gives up the reason the project
exists.

### The interface has to reach more than Python

A C interface is what Python, Julia, a Mathematica link and a C++ system such as
Cadabra can all consume. Choosing a language whose only comfortable export is a
Python module would close that door on the first day, and reopening it later is
a rewrite of the boundary rather than an addition to it.

## The alternative that was considered

C++, and it has one real advantage. xPerm is C and Cadabra is C++, so reuse and
adoption are both easier, and the offer to link xPerm is on the table.

It is not chosen for three reasons. The refusal properties are weaker: there is
no single line that makes unsafe code outside one module fail to compile. The
test, benchmark and fuzz apparatus has to be assembled from separate tools that
each need maintaining, which is the second reason above with the answer
reversed. And the memory safety of a library that other people link into
long-running interactive sessions is exactly the property worth paying for,
because a fault there lands in somebody else's process during somebody else's
work.

If the answer to the xPerm reuse question is that the C is taken directly, this
decision has to be reopened. That question is entry 2 of the open decisions
issue, [#2](https://github.com/iderex/indexwerk/issues/2), and it is not
answered here.

## The costs, paid knowingly

A toolchain the tree does not carry today, and a minimum supported version that
has to be declared and held. Declaring it and exercising it is
[#12](https://github.com/iderex/indexwerk/issues/12).

A wheel build matrix, because a compiled extension does not install from source
on a machine with no compiler. M7 and M10 carry that cost.

No reuse of xPerm's C without a rewrite, so the algorithm is written from the
papers and earns trust through the differential testing in M6 instead of through
inheritance.

A dependency on a small number of crates. Each one added is a supply chain entry
that M8 has to account for, so the core keeps its dependency list short and
states why each entry is there.

## What follows from this record elsewhere

The workspace manifests carry the shape this record chose, and the layer
boundaries it implies are [0005](0005-layering.md) rather than this record.
