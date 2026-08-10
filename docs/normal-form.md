# The normal form

This document fixes the normal form. It is the authority for it: where code, a
test or another document disagrees with what is written here, the other one is
the defect.

The decision that this is written before the canonicaliser is
[0004](adr/0004-normal-form-first.md), and the reasons for writing it at all are
in [#6](https://github.com/iderex/indexwerk/issues/6). This file holds the five
orderings that decision names, and the order on permutations that
[#20](https://github.com/iderex/indexwerk/issues/20) builds against.

Version `1`. What that number is for is the last section.

## What a canonical form is here

A monomial has many spellings that mean the same thing. The factors of a
product commute, a dummy pair may be renamed to any unused name, the two halves
of a dummy pair may be exchanged, and on a manifold with a metric a dummy pair
may be raised or lowered together. Those moves generate a set of spellings, and
the canonical form is one distinguished member of it.

Which member is a choice, and this document is that choice: **the canonical
form is the least spelling under the total order defined below.** Everything
after this sentence exists to make "least" mean one thing.

Two consequences worth having in front of a reader before the orderings.

The order is defined on a monomial as it is written down, not on the search
that finds the minimum. A canonicaliser is free to reach the minimum by any
route, and it is right exactly when the value it returns is the least element of
the set. That is what makes the specification testable by somebody who did not
write the implementation.

The order is total on every monomial the model admits, not only on two
monomials that mean the same thing. A partial order would leave two unrelated
monomials incomparable, and then a sort of a collection of terms would depend on
the order the terms arrived in, which is the same defect as a canonical form
that depends on thread count.

## The concrete syntax

The syntax a case is written in is the text form of the expression model, and
its grammar is in the module documentation of `indexwerk_core::expression`.

    cargo doc --no-deps -p indexwerk-core

It is not restated here. A grammar written in two places drifts, and the copy
somebody reads is then a coin toss. What this document adds to it is the
orderings, which the text form deliberately does not impose.

## 1. The order on index names

Index names are compared by length first and by ASCII byte value second. `a`
comes before `b`, `b` comes before `aa`, and `a2` comes before `a10`.

The reason is the second of those examples. Plain lexicographic order puts `a10`
before `a2`, so a family of generated names reorders itself when the family
reaches ten members, and a canonical form that was stable becomes a different
canonical form because of the width of a number. Comparing length first removes
that.

It also gives the set of names a least element for any non-empty subset, which
is what "the next unused dummy name" needs to be a definition rather than a
convention.

Tensor names are compared the same way, for the same reason and so that a
reader has one rule to hold rather than two.

## 2. The order on variance

Upper comes before lower.

On a manifold with a metric this is more than a tie-break, because there a
dummy pair may sit in either variance and both spellings mean the same thing.
The canonical form writes the first half of a dummy pair upper and the second
half lower, in the order the halves are read.

That fixes a degree of freedom before the search starts rather than inside it,
which is the cheap place to fix one. It also matches how a contraction is
written in the literature, so the canonical form of a contracted pair is the
form a reader would have written by hand.

On a manifold with no metric nothing raises or lowers, so the rule costs
nothing there: a pair already sits one half upper and one half lower, and the
model refuses the other case by name.

## 3. Free indices before dummy pairs

A slot holding a free index is less than a slot holding a dummy index,
whatever the two names are. Name and variance decide only between two slots of
the same kind.

A free index names something outside the expression and may not be renamed. A
dummy name is chosen by the canonicaliser and means nothing outside the
monomial. Sorting the part that cannot move ahead of the part that can means
two monomials with the same free indices agree on a prefix of the order, so the
comparison that separates them is the comparison over the renameable part, and a
search can settle the fixed part first and prune on it.

The alternative, ordering by name and letting a free index fall wherever its
name puts it, makes the position of the fixed data depend on a choice the
canonicaliser is free to make, which is the wrong way round.

## 4. The order on tensor factors within a monomial

Factors are compared by tensor name first, then by rank, then by their slot
sequences position by position. A shorter slot sequence is less than a longer
one that begins with it.

The tensor name comes first because it is what a reader looks for and what a
collection step groups on. Rank comes second because two factors can name the
same tensor at different ranks and the order has to be total, and it is second
rather than first because a reader searching for `R` wants every `R` together.

Slots are compared under rules 1 to 3: kind first, free before dummy, then
name, then variance.

The order on factors is not permission to reorder slots inside one factor.
Slots move only where a declared slot symmetry says they may, which is
[#25](https://github.com/iderex/indexwerk/issues/25) and is not this document.
The order compares two factors; the symmetries decide which factors are
reachable.

## 5. Where the overall sign is carried

On the coefficient, and nowhere else.

A canonical monomial carries its sign in the numerator of its rational
coefficient. No factor carries a sign, no slot does, and there is no separate
sign field beside the coefficient.

One place for the sign is what makes two canonical forms equal exactly when
their text is equal, which is the property the whole normal form exists to
give. A sign that may live in two places gives one value two spellings, and
then a comparison of canonical forms has to know about both, which is the bug
this rule prevents rather than a matter of taste.

A monomial the symmetries force to vanish is not a monomial with the
coefficient zero. It is a distinct result and the canonicaliser returns it as
one, so that a caller who forgets to check gets nothing usable rather than a
plausible term.

## 6. The order on monomials

The five rules above compose into one comparison. Two monomials are compared on
the first of these that differs:

1. The manifold, with a metric before without one.
2. The number of factors, fewer first.
3. The factors, position by position, under rule 4.
4. The coefficient, by numerator and then by denominator.

The manifold is first and it never decides anything within one canonical
problem, because the moves that generate the spellings of a monomial do not
change its manifold. It is in the list so that the order is total over
everything the model admits, which the second consequence in the opening
section asks for.

The coefficient is last because the moves reach it only through the sign, so
comparing it earlier would sort by a quantity that is nearly constant across the
set the minimum is taken over.

## 7. The order on permutations

The canonicaliser searches over group elements, so the elements need an order of
their own, and it is fixed here rather than inherited from whatever a container
does with them.

A permutation is compared by width first, narrower first, and then by its image
array position by position. A signed permutation is compared by its permutation
first and by its sign second, with positive before negative before zero.

Width first, because permutations of different widths are not comparable in any
way the images alone give, and the order still has to be total.

Sign last, because the sign leaves the permutation and lands on the coefficient
by rule 5, so it never decides which spelling is canonical. It is ordered anyway
so that the type has one written order rather than an implicit one, and zero is
last because a zero sign is not a group element at all: it is the record that
the monomial vanishes.

This order is not claimed to be carried over to the order on monomials by the
group action. The canonical form is the least monomial, by section 6, and never
the image of the least permutation. Anybody implementing this should read that
sentence twice, because assuming the two agree is the shortcut that produces a
fast canonicaliser returning the wrong representative.

### The fixture

[`../conformance/order/permutations.txt`](../conformance/order/permutations.txt)
lists signed permutations of widths zero to three in ascending order under this
rule, one per line, written as the sign and then the width and image array.

The file opens:

    + 0:
    - 0:
    0 0:
    + 1:0
    - 1:0
    0 1:0
    + 2:0,1

and it is read by the test suite rather than only quoted here, so an
implementation whose order disagrees with this section reds the build instead of
being noticed later by a reader.

## Changing this document

Changing any ordering above changes which spelling is canonical, so it changes
the output of the library for inputs that were already working. That is a
breaking change and it is treated as one.

The version number at the top is what carries it. It appears in the conformance
vector file as well, and a change to any rule here increments it in both places
in the same commit. A reader comparing two vector files can then tell a normal
form that moved from a case that was added.

The vectors themselves are owed by
[#6](https://github.com/iderex/indexwerk/issues/6) and
[#28](https://github.com/iderex/indexwerk/issues/28), and
[`../conformance/`](../conformance/) holds no vector file yet. So this document
fixes the orderings and nothing in the tree yet compares a canonicaliser against
them, because there is no canonicaliser: the engine is M4 and the search is M5.
