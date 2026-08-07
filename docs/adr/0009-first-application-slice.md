# 0009. The first application computation

Status: accepted.

Issue: [#43](https://github.com/iderex/indexwerk/issues/43).

## The decision

The first release carries metric perturbation theory to first order as its
application slice: the linearised curvature of a metric perturbed about a
background, computed from the perturbation through the covariant derivative,
the contraction and the canonicalisation, and collected into a normal form.

It is the smallest computation that is both perturbation theory and independently
checkable. Smaller candidates exist and are rejected below for one shared
reason, which is that they are not perturbation theory.

## Why not the smallest candidate

The recommendation this decision was asked to argue against is to take the
smallest computation whose answer is independently known, which points at the
Einstein tensor of the FLRW metric, the case the speed comparison already uses.

That would break a decision this project has already taken.
`0001-core-before-layer.md` binds the first release to carry one worked
perturbation-theory computation, and the reason it does is written there: a core
with no application layer competes with Cadabra on Cadabra's own ground, and the
half Cadabra does not cover with packages is the perturbation-theory half. The
Einstein tensor of a background metric is a curvature computation, not a
perturbative one. Shipping it as the application slice would satisfy the letter
of "one computation carried end to end" and none of the reason for it.

So the recommendation is right about the size heuristic and wrong about which
computation it lands on. Applying the same heuristic inside the constraint that
already exists, rather than across it, gives the decision above: the smallest
perturbation-theory computation, not the smallest computation.

The FLRW Einstein tensor keeps a job. It is the benchmark case and it is the
natural first end-to-end correctness check of the core, because its answer is
published and short. It is a test, not the slice.

## What each rejected candidate would have demanded

Cosmological perturbations, the territory of xPand. A background splitting and a
gauge choice on top of everything the chosen slice needs. It is more useful to
more people and it is the larger first bite. Both of those are reasons it is a
second slice rather than a first one: a gauge choice is a modelling decision
that has to be got right in public, and getting it wrong in a first release
discredits the parts that were right.

Polynomial invariants of the Riemann tensor, the territory of Invar. Smaller and
well specified, and the best pure showcase for canonicalisation, because
reduction to a basis of invariants is almost entirely a canonicalisation
problem. Rejected because that is also its weakness as a first slice: it
exercises the least of the application layer, so it demonstrates the core again
rather than demonstrating that the layer above it is reachable. It also runs
into the boundary of `0003-single-term-only.md` sooner than the others, because
a basis of invariants is where multi-term identities bite.

The Einstein tensor of the FLRW metric. It would have demanded the least of
anything: a metric, a curvature computation, and a comparison against a
published answer. Rejected for the reason argued above.

## What the slice does not do

These sentences are the ones the release notes will use, unchanged.

It computes to first order in the perturbation and no further. Second order is
where the term count and the symmetry work grow, and it is not in this release.

It handles single-term symmetries only. An expression whose simplification needs
a multi-term identity, meaning the first or second Bianchi identity, the
Schouten identity, or the Garnir relations they are instances of, is not reduced
by this library, and Cadabra solves that problem today. This is
`0003-single-term-only.md` and it is not a limitation of the slice but of the
core underneath it.

It is one computation, not a package. It is not xPert. It demonstrates that the
path from a perturbation to a collected result runs on this core, and the
breadth of xPert, xPand, Invar and the harmonics work is not in this release and
is not claimed to be.

It carries no gauge machinery. Any gauge condition the worked example uses is
imposed by hand in the example and is not a feature of the library.

## What the answer is checked against

Three independent routes, none of which is this project.

A published result. The linearised curvature of a perturbed metric is standard
and appears with its derivation in the general relativity literature, so the
expected expression is fixed outside this repository before the computation is
run rather than read off its output.

An open system recomputing the same thing. Cadabra computes this class of
expression today and SymPy carries the same canonicalisation algorithm, so both
can be asked the same question independently. That is the differential testing
of M6 applied to the slice rather than to the core, and it is the route that
catches a convention difference rather than an error.

The reader. M9 requires the worked example to be reproducible by somebody who is
not the author, which is a check the other two cannot make: it is the one that
fails if the answer is right and the instructions are wrong.

Where the three disagree, the published result is the authority for the physics
and the normal form document of #6 is the authority for the form the answer is
written in. Those are different disagreements and are not resolved by the same
argument.
