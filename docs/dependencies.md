# Why each dependency is here

One row per direct dependency on something outside this repository, saying why
it is present. A dependency list with no reasons is a list nobody can prune:
after a few years every entry looks load bearing, and the one that was needed
once cannot be told from the one that is needed now.

`crates/indexwerk-checks/src/dependencies.rs` reads this file and reds the
ordinary suite in both directions. A direct dependency with no row here is
refused, and a row here naming a crate no manifest depends on is refused too, so
the register cannot drift away from the tree in either direction.

## The rows

There are none. No manifest in this tree depends on anything outside it:

    $ cat Cargo.lock
    [[package]] indexwerk-checks 0.0.0
    [[package]] indexwerk-core   0.0.0
    [[package]] indexwerk-ffi    0.0.0  dependencies = ["indexwerk-core"]
    [[package]] indexwerk-python 0.0.0  dependencies = ["indexwerk-ffi"]

The three entries with dependencies name each other, which is the layering of
[`adr/0005-layering.md`](adr/0005-layering.md) rather than anything fetched.

An empty register is the reason to write the rule down now rather than later.
The check that refuses a dependency arriving without a row is easier to add
against nothing than against a list somebody has to reconstruct the reasons for,
and the first dependency this tree takes is the one whose argument matters most.

## The shape of a row

A list item: a hyphen, a space, the crate name in backticks, then a sentence.
This file carries no example of one, because an example would be a row, and the
check reads rows wherever they are written rather than only under a heading.

That shape is fixed in the check rather than here, so a row written another way
is a row the check does not see and the dependency it describes still reds. A
name with nothing after it does not count as a row, because a register that
accepts a bare name accepts the shape somebody reaches for to make a red check
go away.

## What a row is expected to say

What the crate does here, and what was weighed against taking it. Not its
description, which is already in its own manifest.

Adding one is a change to this file in the same commit as the change to the
manifest. The two apart is how a reason gets written from memory by somebody who
was not there.

## What this does not cover

Licences of transitive dependencies, and known advisories against them. Both are
[#38](https://github.com/iderex/indexwerk/issues/38) as well and neither is in
the tree. The workspace licence is `AGPL-3.0-or-later`, answered on 2026-08-09
in [#2](https://github.com/iderex/indexwerk/issues/2), so the allowed list a
licence check needs can now be written; nothing here writes it.

What a crate brings with it is covered, and it is a separate question from why it
is here. `crates/indexwerk-checks/src/egress_dependencies.rs` reads the same lock
file and refuses anything in the whole transitive set that carries a route off
this host, which is [#36](https://github.com/iderex/indexwerk/issues/36). A row
in this register does not admit such a crate, and no row can: that check has an
allow list of its own, it is empty, and adding to it needs an issue.
