# 0001. The core is built before the application layer

Status: accepted.

Issue: [#3](https://github.com/iderex/indexwerk/issues/3).

## The decision

Build the canonicalisation core first. Bind the first release to also carry one
worked perturbation-theory computation on top of it, so that what ships is a
library with a demonstrated user rather than a core waiting for one.

## The reasons

### An application layer on a slow core inherits the slowness

The distance between the Mathematica and the Python implementations of the same
package is a factor of nine, 270 ms against 2400 ms for the Einstein tensor of
the FLRW metric. Those figures are published by the author of that port and are
quoted here rather than re-measured, which is what M6 exists to correct. An
application layer written on a core that carries
that factor carries it too. The layer is also the larger body of work by a wide
margin, four packages of physics against one algorithm, so starting with the
layer spends the most effort on the part most likely to be rewritten once a fast
core exists.

### The core has consumers waiting and the layer has none

SymPy's canonicaliser is pure Python and everything above it pays for that.
SageManifolds has no canonicaliser of its own. Cadabra has a good one, but it
lives inside Cadabra rather than as a library with a stable interface, so nobody
else can take it. A core published as a linkable library with bindings is
something those projects could adopt, which makes this cooperation rather than a
second Cadabra with fewer features.

### A core with no application layer competes with Cadabra on its own ground

That is the outcome worth avoiding, and the answer to it is not to build the
layer first. It is to refuse to call a release finished until one slice of the
layer runs on the core. That slice is M9, and M10 depends on it.

## The condition that would overturn this

If the differential testing in M6 shows that the open canonicalisers are already
fast enough for the workloads the application layer needs, then the bottleneck
argument is wrong and the order should flip. That measurement is planned in M6
and is worth taking seriously rather than filing as a formality.
