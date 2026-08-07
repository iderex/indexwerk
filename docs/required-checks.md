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
| `test (windows)` | `.github/workflows/build.yml` |
| `DCO sign-off` | `.github/workflows/dco.yml` |
| `Reject Trojan Source Unicode` | `.github/workflows/unicode-guard.yml` |
| `dependency-review` | `.github/workflows/dependency-review.yml` |
| `Audit workflows (zizmor)` | `.github/workflows/zizmor.yml` |

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

| Check name | Owed by | State |
| --- | --- | --- |
| `format` | #16 | The workflow that would produce it does not exist. |
| `lint` | #16 | The workflow that would produce it does not exist. |

## Does not run on a pull request, by design

| Check name | Produced by | Why |
| --- | --- | --- |
| `Scorecard analysis` | `.github/workflows/scorecard.yml` | It declares no `pull_request` trigger. Its Branch-Protection check reads the default branch's ruleset and its results can only be published from the default branch, so a pull-request run would score the wrong thing and could not publish. It runs on push to `main`, on a schedule, and on a ruleset change. |

## Growing this list

M8 is where the quality parity programme adds the rest, in #37. An element that
programme marks as required is added here with its exact name in the same change
that produces it.
