# Decision records

Every decision that shapes the architecture is written down here with its
reasons before the code that depends on it exists. A record is the place an
argument was settled, so a later reader can reopen the argument rather than
guess at it from the code.

## The format

One record per file, Markdown, wrapped at eighty columns like the rest of the
prose in this tree. The file name is the number, a hyphen, and a short slug:
`0004-normal-form-first.md`.

A record opens with four lines and then its sections:

    # 0004. The normal form is specified before it is implemented

    Status: accepted.

    Issue: [#6](https://github.com/iderex/indexwerk/issues/6).

The issue link is not decoration. The issue is where the decision was argued and
where its done-condition lives, and a record with no issue behind it is a
decision nobody was able to object to.

After that come the decision, the reasons, and whatever else the decision
actually needs. Most records carry a cost or a rejected alternative, and a
record that names neither is usually a record that has not been argued. A
closing section saying what follows from the record elsewhere is where the
obligations it creates in other milestones are named, so that a reader can tell
an unenforced statement from an enforced one.

## The numbering

Four digits, allocated when the record is written, never reused.

The numbers here are not consecutive by accident. Each of the eight M1 decision
records takes the number two below its issue, so record `000N` belongs to issue
`#N+2`, and that mapping is what fixes the number rather than a counter somebody
has to increment. Records after M1 take the next free number.

## Superseding

A record is never deleted and never rewritten to say something else. What
happened is part of what the record is for.

When a decision is replaced, the old record's `Status:` line becomes
`superseded by 00NN` and the new record says which record it supersedes and what
changed. A reader arriving at the old number lands on the old argument and is
sent forward, which is what an incoming link from outside this repository will
do.

A record whose decision turned out to be wrong is superseded, not corrected. The
correction is the new record.

## The records

This list is written by hand and it is short. A long index is a sign the records
are too small. It is also the one thing here that can drift against the
directory, so if the two disagree, the directory is right.

| Record | Decision | Issue |
| --- | --- | --- |
| [0001](0001-core-before-layer.md) | The core is built before the application layer | [#3](https://github.com/iderex/indexwerk/issues/3) |
| [0002](0002-the-means.md) | The means the core is made of | [#4](https://github.com/iderex/indexwerk/issues/4) |
| [0003](0003-single-term-only.md) | The first core handles single-term symmetries only | [#5](https://github.com/iderex/indexwerk/issues/5) |
| [0004](0004-normal-form-first.md) | The normal form is specified before it is implemented | [#6](https://github.com/iderex/indexwerk/issues/6) |
| [0005](0005-layering.md) | A core with no input or output, a C interface, a Python package | [#7](https://github.com/iderex/indexwerk/issues/7) |
| [0006](0006-determinism.md) | Results do not depend on thread count | [#8](https://github.com/iderex/indexwerk/issues/8) |
| [0007](0007-exact-arithmetic.md) | Exact arithmetic only, no floating point in the core | [#9](https://github.com/iderex/indexwerk/issues/9) |
| [0008](0008-nothing-leaves-the-host.md) | Nothing leaves the host unless the operator deliberately federates | [#10](https://github.com/iderex/indexwerk/issues/10) |
| [0009](0009-first-application-slice.md) | The first application computation | [#43](https://github.com/iderex/indexwerk/issues/43) |
