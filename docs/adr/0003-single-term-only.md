# 0003. The first core handles single-term symmetries only

Status: accepted.

Issue: [#5](https://github.com/iderex/indexwerk/issues/5).

## The decision

The first core canonicalises with respect to single-term symmetries only. That
means the slot symmetries of each tensor, including the sign-carrying ones, and
the renaming of contracted dummy index pairs.

Multi-term symmetries are out of scope. The four excluded identities, named so
that nobody has to discover the boundary by hitting it:

1. The first Bianchi identity.
2. The second Bianchi identity.
3. The Schouten identity.
4. The Garnir relations, of which the three above are instances.

## The reasons

### Multi-term symmetry is a different problem, not a harder case of the same one

Butler-Portugal finds a minimal representative of a double coset, which is a
statement about one monomial. A multi-term identity says that a sum of monomials
vanishes, so the canonical form is a choice of basis for a quotient, and the
method that solves it is Young projection with linear algebra over that
quotient. Cadabra is the system that did this, and it is real research rather
than an optimisation of what is planned here.

### Doing both at once entangles the fast path with the hard path

The two would be intertwined from the first commit, and the speed claim, which
is the reason this project exists, would be measured on a system carrying an
unfinished second algorithm.

### The boundary is honest rather than convenient

A user who hits it should be pointed at Cadabra, which solves their problem
today. The documentation says so in those words.

## The two conditions that would reopen it

Both of the following, not either one on its own:

1. A working single-term core with the evidence of M6 behind it.
2. A stated use in the application layer that cannot be expressed without
   multi-term reduction.
