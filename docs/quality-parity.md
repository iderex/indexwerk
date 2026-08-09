# Quality parity with the target gate

This board takes its quality bar from a gate that has been run against real
changes for a long time rather than inventing one. The target is the merge gate
of the public single sign-on plugin board,
[Flowfin/jellyfin-plugin-sso](https://github.com/Flowfin/jellyfin-plugin-sso).
The address is here so a reader can go and compare rather than trust this
summary. Issue #37 is where the programme was argued.

That gate was built for a .NET authentication plugin. This is a compiled
mathematics library with Python bindings, so parity means the same failure is
refused, not that the same tool is run. Every element below is matched,
deviated from, or not adopted, and every deviation carries one sentence saying
why it is one.

This file restates no check name of this board. `docs/required-checks.md` is
that list, it moves in the same change that produces or renames a name, and a
second copy here would drift against it.

## What the target gate is, read rather than described

The required half is the ruleset that protects the target's mainline:

    gh api repos/Flowfin/jellyfin-plugin-sso/rulesets --jq '.[] | {id, name, enforcement}'
    {"enforcement":"active","id":18802863,"name":"Protect main and 5.0"}

    gh api repos/Flowfin/jellyfin-plugin-sso/rulesets/18802863 --jq '{rules:[.rules[].type], checks:[.rules[]|select(.type=="required_status_checks")|.parameters.required_status_checks[].context]}'
    {"checks":["build","ABI floor build","Package (JPRM) / Build package","Package (JPRM) / Generate SBOM","CodeQL","Analyze (csharp)","DCO sign-off","Deterministic PR-hygiene checks","Enforce greppable invariants","Reject Trojan Source Unicode","Audit workflows (zizmor)","prettier","dependency-review"],"rules":["deletion","non_fast_forward","required_status_checks","pull_request"]}

Thirteen names, and they are the rows of the first table below.

An unrequired leg leaves no trace in a ruleset, so the rest is read from the
workflow list. What makes a leg unrequired is that it is absent from the output
above:

    gh api repos/Flowfin/jellyfin-plugin-sso/actions/workflows --jq '.workflows[] | "\(.name)\t\(.path)"'

### A check name is not a workflow name

Reading those two outputs side by side invites a mistake worth naming, because
the names do not line up and four of the thirteen come out of a single workflow.
The mapping is derived rather than guessed, from the jobs that ran on a recent
pull request head on that board:

    for id in $(gh api "repos/Flowfin/jellyfin-plugin-sso/actions/runs?head_sha=a8311a6b1344d492ae04c485c870d741677bf567&per_page=50" --jq '.workflow_runs[].id'); do
      NAME=$(gh api "repos/Flowfin/jellyfin-plugin-sso/actions/runs/$id" --jq '.name')
      JOBS=$(gh api "repos/Flowfin/jellyfin-plugin-sso/actions/runs/$id/jobs" --jq '[.jobs[].name] | join(", ")')
      echo "$NAME  ==>  $JOBS"
    done

    PR Hygiene  ==>  Deterministic PR-hygiene checks
    Fuzz (SharpFuzz)  ==>  Fuzz idtoken, Fuzz jwks, Fuzz roles, Fuzz discovery, Fuzz saml
    .NET  ==>  build, ABI floor build, Package (JPRM) / Build package, Package (JPRM) / Generate SBOM
    Repo Invariant Lint (Opengrep)  ==>  Enforce greppable invariants
    Workflow Security Analysis  ==>  Audit workflows (zizmor)
    DCO  ==>  DCO sign-off
    unicode-guard  ==>  Reject Trojan Source Unicode
    PR Hygiene  ==>  Deterministic PR-hygiene checks
    Prettier Lint  ==>  prettier
    Dependency review  ==>  dependency-review
    CodeQL  ==>  Analyze (csharp), Analyze (javascript-typescript), Analyze (actions)
    unicode-guard  ==>  Reject Trojan Source Unicode
    Automatic Dependency Submission (NuGet)  ==>  submit-nuget

Three things follow from that output and none of them is visible in the two
lists on their own.

`Repo Invariant Lint (Opengrep)` is the workflow that produces the required
`Enforce greppable invariants`, not a second unrequired leg beside it. So the
invariant pass is one element and it appears once below.

`.NET` is the workflow behind four of the thirteen required names, and `Build`
is a reusable workflow it calls rather than an element of its own. Neither is a
row below for that reason.

`CodeQL` in the ruleset is the code-scanning aggregate and `Analyze (csharp)` is
one job of the workflow of the same name, whose two other jobs are not required.

A workflow appearing twice in that output triggers on both the push and the pull
request, and both runs carry the same job name.

## Where this board stands

The workflow files behind the standing column, read from the mainline rather
than from a working tree:

    git ls-tree -r --name-only origin/main -- .github/workflows/
    .github/workflows/build.yml
    .github/workflows/dco.yml
    .github/workflows/dependency-review.yml
    .github/workflows/invariants.yml
    .github/workflows/lint.yml
    .github/workflows/scorecard.yml
    .github/workflows/unicode-guard.yml
    .github/workflows/zizmor.yml

### Required on the target

| Target check | Status | Reasoning | Where it stands here |
| --- | --- | --- | --- |
| `build` | matched | | In the gate, with the compiler's warnings denied. |
| `ABI floor build` | deviated | The floor that breaks a consumer here is a toolchain version rather than a host application version, so it becomes a minimum supported compiler build. | In the gate, on legs that read the floor out of the workspace manifest and refuse to run if they got another compiler. |
| `Package (JPRM) / Build package` | deviated | The artefact an operator installs here is a wheel rather than a plugin package, and a release whose package was never built in the gate is a release nobody tested. | Owed by #48. |
| `Package (JPRM) / Generate SBOM` | deviated | The obligation is to publish what is inside the artefact rather than to run one particular generator, so the format and the tooling are chosen for this ecosystem. | Owed by #38. |
| `CodeQL` | deviated | This project's languages are not the one that engine reads best, so matching the tool alone would either analyse nothing or analyse only the thin binding layer. | Owed by #39. |
| `Analyze (csharp)` | deviated | Its language is absent from this tree, so the replacement is an analysis pass over the languages actually present. | Owed by #39. |
| `DCO sign-off` | matched | | In the gate. |
| `Deterministic PR-hygiene checks` | deviated | The hygiene rules carry a commit subject convention, and a convention is a property of the board it was written for. | Owed by #40. |
| `Enforce greppable invariants` | deviated | The mechanism is worth adopting and the invariants are not, because an invariant worth enforcing is a property of this code rather than of that one. | In the gate, and `docs/invariants.md` is what it enforces. |
| `Reject Trojan Source Unicode` | matched | | In the gate, and it matters more here, because index names are user-supplied text that ends up in source-shaped output. |
| `Audit workflows (zizmor)` | matched | | In the gate. |
| `prettier` | deviated | A formatter that does not know the language cannot check it, so this becomes the formatter of each language present. | In the gate for the languages present, with the Python half failing closed rather than configured, owed by #16. |
| `dependency-review` | matched | | In the gate. |

### Runs on the target and is not required there

| Target leg | Status | Reasoning | Where it stands here |
| --- | --- | --- | --- |
| `Scorecard supply-chain security` | matched | | In the gate, and not required here either. |
| `Stryker mutation testing` | deviated | A mutation tool rewrites source in one language and this tree is written in another. | Owed by #42, and not required, matching how it is run there. |
| `Fuzz (SharpFuzz)` | deviated | The same substitution of tool for language applies, and the properties worth fuzzing here belong to the parser and the canonicaliser rather than to an authentication path. | Owed by #30, and not required for the reason it is not required there, that a fuzz run has no fixed duration. |
| `E2E Login Harness` | deviated | The real user path here is an install-and-run of the built wheel computing the worked example, because there is no sign-in to exercise. | Owed by #45 and #51. |
| `Analyze (actions)` | deviated | Workflow files are analysed here by the audit leg that is already required, so a second engine over the same files would gate one failure twice. | In the gate, as the `Audit workflows (zizmor)` row above. |
| `Analyze (javascript-typescript)` | not adopted | Neither language is in this tree, and an engine run over an absent language reports nothing in either direction. | |
| `submit-nuget` | not adopted | It submits a build-time package graph for an ecosystem absent from this tree, and the lockfile a dependency review reads here is tracked instead. | |

### Not adopted

| Target legs | Status | Reasoning |
| --- | --- | --- |
| The publishing and manifest workflows in the workflow list above | not adopted | There is nothing to publish until M10, and a publishing gate written before there is a publishing pipeline would be guessed rather than derived. |
| `Wiki Lint` | not adopted | The documentation this project ships lands in the tree, which is #49, so a leg that reads a wiki has nothing here in its scope. |

The workflow list also holds entries under a `dynamic/` path rather than a
workflow file. Those are integrations configured on that board rather than
elements of its gate, and adopting one is a repository setting rather than a
change to a tree, so none of them is a row above.

## Every element above has an issue, and none of them is unassigned

The second condition of #37 asks that every element marked matched or deviated
has an issue here, and that each of those is closed or open with an assignee.
The issue numbers are in the standing column. The assignee half is a property of
the tracker rather than of this file:

    gh issue list --repo iderex/indexwerk --state open --limit 200 --json number,assignees --jq '[.[] | select(.assignees | length == 0) | .number] | length'
    0

No open issue on this board is unassigned, so the condition holds for the ones
named above and for the rest. That count moves, so re-run it rather than cite
it.

## What this file does not settle

Nothing here makes a check required. What the ruleset on this board requires is
read from the ruleset and recorded in `docs/required-checks.md`, in its own
opening section, and changing it is a repository setting rather than a change to
this tree. No check name of this board appears here, so this file is not a
second copy of that list to go stale when the ruleset moves.

The third condition of #37, that the required check list has grown to include
everything this programme marks as required, is not met by this file and cannot
be met yet. Four of the elements above have no check name fixed anywhere: the
wheel build in #48, the bill of materials in #38, the hygiene leg in #40, and
the analysis pass in #39. A required check is matched by its literal name, so a
name guessed here and spelled differently by the workflow that later produces it
would be a check that silently is not required, with a document asserting that
it is. Each name is added to `docs/required-checks.md` by the change that
produces it, which is what that file's own closing section says.
