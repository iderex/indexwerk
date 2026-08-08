# The checks a change must pass

A required check is matched by its literal name. A job renamed in a workflow
file silently stops being required and nothing reports that it happened. So the
list lives here, next to the workflows that produce it, the contributing
documentation points at this file rather than restating it, and changing a check
name and this file is one change.

The list also says what is deliberately not required and why, so that an absence
can be told from an oversight.

## What enforces this list today, which is nothing

This file is the list. It is not the enforcement. The ruleset on `main` carries
no required status check, read from the ruleset rather than from anyone's
memory:

    gh api repos/iderex/indexwerk/rulesets/20527780 --jq '{name: .name, rules: [.rules[].type]}'
    {"name":"gate","rules":["deletion","non_fast_forward","pull_request"]}

So a red check below blocks nothing on its own, and a merge is refused only for
a direct push, a force push, or the absence of a pull request. Making the
ruleset require these names is a repository setting rather than a change to this
tree, and it is not made by the change that adds this file.

## Required

| Check name | Produced by |
| --- | --- |
| `build` | `.github/workflows/build.yml` |
| `test` | `.github/workflows/build.yml` |
| `build (floor toolchain)` | `.github/workflows/build.yml` |
| `test (floor toolchain)` | `.github/workflows/build.yml` |
| `test (windows)` | `.github/workflows/build.yml` |
| `format` | `.github/workflows/lint.yml` |
| `format (windows)` | `.github/workflows/lint.yml` |
| `lint` | `.github/workflows/lint.yml` |
| `DCO sign-off` | `.github/workflows/dco.yml` |
| `Reject Trojan Source Unicode` | `.github/workflows/unicode-guard.yml` |
| `dependency-review` | `.github/workflows/dependency-review.yml` |
| `Audit workflows (zizmor)` | `.github/workflows/zizmor.yml` |
| `invariants` | `.github/workflows/invariants.yml` |

`invariants` runs a suite the `test` leg already runs, and the duplication is
the point rather than an oversight. #41 asks for one check name that carries
every greppable invariant, and a name that also carried the rest of the
workspace would report a broken invariant and a failing unit test as the same
red. What that leg enforces is listed in `docs/invariants.md`, which is
rendered by the check rather than written beside it.

The two format names say which platform they ran on, for the reason #16 gives:
a formatter default that assumes one platform reports a whole clean tree as
failing on the other, and the only way to know this one does not is to run it on
both. `format` also runs a check the `test` leg already runs, in
`crates/indexwerk-checks/tests/formatting.rs`, and the duplication is deliberate
in the same way `invariants` is: a formatting failure and a failing unit test
should not arrive as the same red.

The two floor names say which compiler they ran on rather than only that they
ran, because a reader of this list has to be able to tell the leg that exercises
the minimum supported version from the leg that exercises the pinned one. Which
version each is comes from the tree rather than from the workflow: the pinned
one is `rust-toolchain.toml` and the floor is `rust-version` in the workspace
manifest, and the floor legs read the manifest and refuse to run if the compiler
they got is not the one it declares.

Two of those names are not chosen freely. `dependency-review` is the job id,
used because that workflow declares no job `name:` and the check run then takes
the id; giving the job a name would rename the check run. `Reject Trojan Source
Unicode` runs twice on a pull request from a branch of this repository, once for
the push and once for the pull request, because that workflow triggers on both;
both runs carry the same name.

## Runs but is not required

| Check name | Produced by | Why it is not required |
| --- | --- | --- |
| `zizmor` | code scanning, from the SARIF that `.github/workflows/zizmor.yml` uploads | It reports the same findings the `Audit workflows (zizmor)` job already fails on, so requiring it would gate the same failure twice and would also gate the upload, which is skipped on a fork or Dependabot pull request where the token cannot write security events. |

## Declared but not produced yet

These names are fixed in their issues and have no check run yet, so they cannot
be required. They are listed so that their absence reads as owed rather than as
decided against.

Nothing is in this state today. `format` and `lint` were, until
`.github/workflows/lint.yml` landed and produced them.

## Does not run on a pull request, by design

| Check name | Produced by | Why |
| --- | --- | --- |
| `Scorecard analysis` | `.github/workflows/scorecard.yml` | It declares no `pull_request` trigger. Its Branch-Protection check reads the default branch's ruleset and its results can only be published from the default branch, so a pull-request run would score the wrong thing and could not publish. It runs on push to `main`, on a schedule, and on a ruleset change. |

## Growing this list

M8 is where the quality parity programme adds the rest, in #37. An element that
programme marks as required is added here with its exact name in the same change
that produces it.
