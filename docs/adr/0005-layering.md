# 0005. A core with no input or output, a C interface, a Python package

Status: accepted.

Issue: [#7](https://github.com/iderex/indexwerk/issues/7).

## The decision

Three layers, each with a rule.

### The core crate

It holds the algorithm and performs no input or output. It opens no file, reads
no environment variable, prints nothing, and starts no thread pool the caller
did not ask for. It is testable with no bindings present.

### The foreign-interface crate

It holds the C interface and every unsafe line in the project. It is the only
crate where `#![forbid(unsafe_code)]` is absent, and it carries a comment at the
top naming this record and saying why it is the exception.

### The Python package

It sits on the C interface like any other consumer, with no privileged access to
the core. If the Python layer needs something the C interface cannot express,
the answer is to widen the C interface, not to reach past it.

## The reasons

### A core with no input or output is a core that tests headless

It tests in parallel, without a display and without elevation, which M3 makes a
birth requirement. Every piece of input or output added to a core is a piece of
test setup added to every test that touches it.

### One small unsafe crate makes the audit surface a named set of files

Putting all unsafe code in one place turns the audit surface into a fixed list
instead of a property of the whole tree. It is also what makes the compile-time
refusal recorded in the decision record on the means possible everywhere else.

### Python must not be a privileged consumer

Otherwise the other consumers named in the roadmap never arrive. The measure of
whether the C interface is adequate is whether the Python package needed to
cheat, and that measure only works if cheating is impossible.

## The names

The three layers are three workspace members and no more. Their names are fixed
here rather than in the manifest, because the manifest is where they are used
and this is where they are argued about.

| Layer | Crate | Directory |
| --- | --- | --- |
| Core | `indexwerk-core` | `crates/indexwerk-core` |
| Foreign interface | `indexwerk-ffi` | `crates/indexwerk-ffi` |
| Python package | `indexwerk-python` | `crates/indexwerk-python` |

The foreign-interface crate builds a library called `indexwerk`, and every
symbol it exports carries the `indexwerk_` prefix so that linking alongside
another tensor library does not collide.

The layering rule is visible in the manifests rather than only in this
paragraph. `indexwerk-python` declares `indexwerk-ffi` as its only dependency
and does not declare `indexwerk-core`, so reaching past the C interface is a
line somebody has to add to a manifest rather than something that happens by
accident.

## What follows from this record elsewhere

The workspace created in M2 has exactly this shape, and a check in M8 refuses an
unsafe block outside the declared crate, so that removing the compiler attribute
is refused as well.
