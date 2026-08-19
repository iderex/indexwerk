# Security policy

## What this repository is

Taken from the tree rather than from the description. `indexwerk` is a Cargo
workspace of four Rust crates plus one package deliberately kept out of the
workspace:

- `crates/indexwerk-core`, the canonicalisation core. It forbids unsafe code,
  has no dependencies, and by its own rule performs no input or output: no
  file, no environment variable, no thread pool the caller did not ask for.
- `crates/indexwerk-ffi`, the C interface, and the only crate in the tree
  allowed to contain unsafe code.
- `crates/indexwerk-python`, which reaches the core through the C interface and
  has no privileged path to it.
- `crates/indexwerk-checks`, which ships to nobody. It scans the tree for the
  constructs the invariants forbid.
- `harness/`, excluded from the workspace so `cargo test --workspace` cannot
  reach it, for measurements that need hardware or a licence.

The description calls this fast canonicalisation of tensor index expressions.
The tree does not contain that yet. `indexwerk_core::layers()` returns the
number 3 and is documented as a placeholder, and the crate documentation names
the base and strong generating set, the orbit and stabiliser operations and the
double coset search as not present. What does exist is the index expression
model in `expression.rs`, the permutation and sign type in `permutation.rs`,
and the exact rational coefficient in `rational.rs`. `Cargo.lock` lists four
packages and all four are in this tree, so there is no third party dependency
today.

That state is most of this policy, and I would rather say so than write a
threat model for a program that has not been built yet. An engine that does not
exist cannot return a wrong answer, and a workspace with no external
dependencies has no transitive supply chain to compromise.

## Reporting

Report privately through GitHub Security Advisories:

    https://github.com/iderex/indexwerk/security/advisories/new

That channel is open. The reading, taken today:

    $ gh api repos/iderex/indexwerk/private-vulnerability-reporting
    {"enabled":true}

Please use it rather than the issue tracker. The tracker is public and every
issue on it is readable while it is being fixed.

I promise no acknowledgement deadline and no fix deadline. This is one person's
project with no rota behind it, and a deadline that cannot be kept is worse
than no deadline at all: a reporter who is told to expect an answer within a
stated time and then hears nothing spends that silence wondering whether the
report arrived, rather than whether it is being worked on. You will get an
answer when there is one, and if a report turns out to be something I will not
fix I will say that instead of leaving it open.

## What this program does with input it did not produce

There is one such entry point today. `Monomial` implements `FromStr` over the
one-line text form whose grammar is documented at the top of
`crates/indexwerk-core/src/expression.rs`, and `parse_factor` and `parse_slot`
below it do the work. The grammar has no nesting and the parser is flat and
iterative, so there is no recursion for a hostile line to drive into the stack,
and every failure returns a `TextError` rather than panicking. If you can make
that path panic, abort, allocate without bound or loop forever on a line of
text, that is a vulnerability in this repository and I want the line that does
it.

Everything else in the core is a constructor a caller reaches with values it
built itself.

Across the trust boundary there is currently one exported symbol,
`indexwerk_layers()`, which takes no pointer and owns nothing. The ownership,
nullability and thread safety rules, and the generated header, are named in the
crate as not written yet. So the usual C interface surface, a pointer freed
twice or a length trusted from the caller, is absent rather than handled well.
There is also no CPython binding, no build backend and no wheel, so nothing
here loads into somebody else's Python process today.

Nothing in the three shipped crates opens a socket, resolves a name, spawns a
process or reports anything anywhere. That is decided in
`docs/adr/0008-nothing-leaves-the-host.md`, and it is checked rather than
asserted: `crates/indexwerk-checks` scans the shipped crates for those
constructs, and the invariant table it holds is rendered into
`docs/invariants.md`. The check is a line scan and says so about itself, so
read the guarantee as no shipped source spelling a network construct, which an
empty dependency list makes easy to hold today and harder on the day the first
dependency arrives.

One honest cost, since somebody will find it. `Monomial::check_indices` scans
the names it has already seen once per slot, which is quadratic in the number
of distinct index names in a single monomial. On expressions a physicist writes
that is nothing. If you put this library behind a service that parses lines
from strangers, bounding that cost is yours, and I would rather have that
report on the public tracker as a performance defect.

## What is not a vulnerability here

**Something that is missing.** The placeholder return value, the absent
canonicaliser, the missing C header, the absent `pyproject.toml`, the DCO
certificate text the workflow points at and the tree does not contain. Each of
those is owed and named as owed in the tree. A gap is not an exposure.

**A wrong canonical form, once there is one.** A wrong answer to a question the
caller asked crosses no boundary. It is a correctness defect and it belongs on
the public tracker, where it can be argued with the fixture that shows it. The
exception is worth stating: if the wrong answer can be induced by input the
caller does not control, that is the advisory channel.

**Multi-term identities.** The first core canonicalises single-term symmetries
only, and the README says which problems belong elsewhere. Not supporting a
Bianchi or Schouten identity is a documented boundary rather than a defect.

**Anything in `harness/`.** It is outside the workspace, the gate never
compiles it, no leg produces a result, and every leg exits non-zero saying so.
Running it requires the machine it names, and whoever has that machine already
has the machine.

**Anything that starts from write access to this repository.** The workflows
run on `push` and `pull_request` and never on `pull_request_target`, every
action is pinned to a full commit hash, checkout runs with
`persist-credentials: false`, and the default permission is `contents: read`. A
finding that begins by assuming a commit already landed on `main` is a report
about account compromise, which is real and which no file in this tree fixes.

**Scanner output with no path from an input to an effect.** There are no
dependencies here to carry advisories, and a report that some setting is not
hardened, without saying what an attacker gets from it, is not something I can
act on.

**Licensing and intended use.** The terms are AGPL-3.0-or-later in `LICENSE`
and the intended-use notice is `NOTICE.md`. Neither is a security channel.

## Scope

Everything tracked in this repository on `main`, and nothing else. The products
the README compares against are other people's code, and a defect in one of
them belongs to them.

## After a report

I fix it on `main` and publish an advisory from the same channel the report
came in on, naming what was reachable and from where. If you want credit, say
so in the report and you get it; if you would rather not be named, that is fine
too. Nothing is published from this tree today, `publish = false` in the
workspace manifest and the repository has no releases and no tags, so there is
nothing downstream to yank. When that changes, this section changes with it.
