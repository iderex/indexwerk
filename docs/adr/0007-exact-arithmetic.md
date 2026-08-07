# 0007. Exact arithmetic only, no floating point in the core

Status: accepted.

Issue: [#9](https://github.com/iderex/indexwerk/issues/9).

## The decision

The core uses machine integers and, where a coefficient is needed, exact
rationals with arbitrary-precision numerators and denominators. Floating point
types are not used in the core crate at all, and a check refuses their
appearance there.

## The reasons

### Nothing in the work needs an approximation

Canonicalisation is combinatorics on finite groups. A float in that path is
either a mistake or a shortcut, and a shortcut there produces a
machine-dependent answer.

### A machine-dependent canonical form breaks the equality test

The library exists to let two expressions be compared by comparing their
canonical forms. A form that differs between two machines because of rounding
breaks that comparison, and it breaks it in a way that is nearly impossible to
attribute to its cause.

### Coefficients arriving from the application layer are exact in the sources

The rational factors that appear when a perturbation expansion is collected are
exact rationals in the literature this project follows. Representing them as
floats would introduce error into a computation that had none.

## The cost

Arbitrary-precision rationals are slower than machine doubles and they pull in a
dependency.

The speed cost lands on coefficient arithmetic. That is not where the published
factor of nine lives, so the cost is paid in the cheap place.

The dependency is named and justified in the supply chain work in M8, which is
where every dependency of this tree is accounted for.

## What follows from this record elsewhere

A greppable-invariant check in M8 refuses `f32` and `f64` in the core crate and
names the file and line when it fires.
