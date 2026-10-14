# `spex-implement` skill

Carries out an existing plan, or implements an ad-hoc user request. Loaded when the user wants to carry out a plan from `<spex-dir>/plans/<plan-name>`, or asks for a change without a formal plan.

## Acceptance criteria

### Process

- MUST follow each process step without skipping, whether implementing an existing plan or an ad-hoc user request.

### Step 1: gather context

- MUST finish this step before editing or writing any code file.
- MUST, with a plan, read the plan and run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the plan's `Specs` to read the governing specs.
- MUST, without a plan (ad-hoc user request), determine the packages to work on (e.g. frontend, backend), run `spex rules --package <package-a> <package-b> --ai --output <tmp-path>` for those packages and read `<tmp-path>` for the governing rules, determine all governing spec IDs related to the request, and run `spex specs --ai --id ...` for them.
- MUST, without a plan, produce a note in its response with a list of the governing rules and specs.
- MUST read all the specs and the governing rules, without skipping anything.

### Step 2: implement

- MUST skip, with a plan, any step or task already `completed` and resume at the first incomplete one, since the steps are already ordered.
- MUST edit the plan's `Status` in place as it goes, using only `not-started`, `in-progress` and `completed`, and leave every other line of the plan untouched.
- MUST split, without a plan, the request into tasks only when it spans independent packages or clearly separable files; most requests are a single task.
- MUST run every task within a unit in parallel, one AI agent per task.
- MUST stop, report what failed, and ask the user how to proceed when a task fails or an agent cannot complete it, rather than pressing on with dependent work.
- MUST annotate, as it implements each acceptance criterion, its boundary with `@spec impl <spec-id>#<ac-id>`, tagging the specific criterion id it satisfies; for a contract, an OpenAPI `x-*` vendor extension whose value is `@spec contract <spec-id>`, anchored per criterion as `@spec contract <spec-id>#<ac-id>` when the spec's `contract` is `for-each-ac`.
- MUST place each annotation on the smallest unit that actually satisfies the criterion (a specific field, function, or handler), widening the placement only when the criterion's scope genuinely spans that boundary (e.g. an endpoint, contract), and keep the total number of annotations minimal.
- MUST NOT write tests in this step; `spex-test` does it once implementation is done.
- MUST NOT annotate the file, the enclosing type, or a whole module.

### Step 3: finalize the specs

- MUST open `<spex-dir>/specs/<spec-id>.yaml` for every spec the work implemented or advanced (the plan's `Specs`, or the specs identified in Step 1).
- MUST set `status:` to `in-progress`, adding the line if absent, when it is absent or `planned`.
- MUST touch only the `status` line, leaving the rest of each spec, including `is_done`, untouched.
- MUST NOT set `status` to `active` here: this skill does not write tests, so it cannot know a criterion is actually done; that determination, and clearing each criterion's `is_done: false`, is `spex-test`'s job once it verifies impl+test tracing with `spex coverage`.

### Step 4: verify

- MUST run formatters and linters, and run the existing test suite to confirm the change did not break anything.
- MUST prove the work respects the governing rules with `spex check --package <package-a> <package-b>`.
- MUST prove the work completes the governing specs with `spex coverage`.
- MUST fix every gap `spex coverage` reports, except a missing `test` for a criterion it just annotated, which is expected here since `spex-test` adds it next.

### Step 5: delete the plan

- MUST delete the plan file, with a plan, once every step and task in it is `completed`, without asking the user for confirmation: a finished plan has nothing left to say, since the spec, code, git and tests already record what shipped, and leaving it only pollutes future search with an obsolete artifact.
- MUST leave the plan file in place when any step or task is not yet `completed` (the run stopped partway through), so work can resume later.
- MUST delete nothing without a plan.

### Step 6: update package skill documentation

- MUST check, for every package touched, whether its `<package-name>` skill (`SKILL.md` and any `references/*.md`) still matches the code: project structure, commands, and any documented behavior the work changed or added ([package-skill-skill.md](package-skill-skill.md)).
- MUST update only what is now inaccurate or missing, without rewriting unrelated sections.
- MUST add a new `references/*.md` file only when the change introduces package-specific context worth documenting that way, and index it in `SKILL.md`.
- MUST NOT duplicate what specs or rules already state in the skill doc; reference them instead.

### Step 7: report

- MUST summarize which units of work completed.
- MUST summarize which specs it set to `in-progress` (or left as-is), and which acceptance criteria `spex coverage` still shows untraced.
- MUST summarize the format, lints, tests, `spex check` and `spex coverage` run outcome.
- MUST summarize whether the plan file was deleted (fully completed) or left in place (partial run).
- MUST summarize which package skill docs it updated or added, and why.

### Step 8: point to the next skill

- MUST ask the user to do an optional review with `spex-review`, then to add test coverage for the change with `spex-test`.
