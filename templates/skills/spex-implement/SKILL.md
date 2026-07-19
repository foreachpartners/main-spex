---
name: spex-implement
description: Carry out an existing plan, or implement an ad-hoc user request. Use when the user wants to carry out a plan from <workspace-root>/<project-name>-spex/plans/<spec-id>/<plan-name>, or asks for a change without a formal plan.
---

# Process

**You MUST follow each process step without skipping**, no matter whether you are implementing an existing plan or an ad-hoc user request.

## Step 1 — Gather context

**Do not edit or write any code files until this step is done; do not skip it**.

### With a plan

Plan already contains all the information — read it.

### Without a plan (ad-hoc user request)

1. Determine repositories we will work on (e.g. scope: frontend, backend).
2. Determine all governing rule specs (`rs_specs`) listed in `<workspace-root>/<project-name>-spex/spex.yaml` for the affected scope from step 1.
3. Determine all governing functional specs in `<workspace-root>/<project-name>-spex/fs-specs/<spec-id>.yaml` related to the request.
4. Produce a note in response with a list of governing rule and functional specs.

### Both paths

Read all rule and functional spec files listed. Don't skip anything, this is not negotiable!

## Step 2 — Implement

### With a plan

Its steps are already ordered — skip any step or task already `completed` and resume at the first incomplete one.

Edit the `Status` in place as you go. Use only `not-started`, `in-progress` and `completed`. Leave every other line of the plan untouched.

### Without a plan (ad-hoc user request)

Split the request into tasks only when it spans independent repositories or clearly separable files; most requests are a single task.

### Both paths

Within a unit, run every task in parallel, one AI agent per task.

If a task fails or an agent cannot complete it, stop, report what failed, and ask the user how to proceed rather than pressing on with dependent work.

## Step 3 — Add tests

1. Add tests for each completed acceptance criterion of implemented functional specs.
2. Add all necessary tests for new functionality or changes based on repository testing patterns.

## Step 4 — Finalize the functional specs

For every functional spec the work implemented or advanced — the plan's `Specs`, or the specs you identified in Step 1 — open `<workspace-root>/<project-name>-spex/fs-specs/<spec-id>.yaml`:

- Tick the checkbox of every acceptance criterion the run implemented, `[ ]` to `[x]`; leave any criterion not yet implemented as `[ ]`.
- Change `status:` to `active` (add the line if absent — it defaults to `planned`).

Touch only the `status` line and the acceptance-criteria checkboxes — leave the rest of each spec untouched.

## Step 5 — Verify

Run formatters, linters, and tests to verify implementation.

Prove the work respects the governing rs-specs:

```sh
spex check --repo <repo-a> <repo-b>
```

Prove the work completes the governing fs-specs:

```sh
spex coverage
```

## Step 6 — Report

Summarize the run:

- Which units of work completed.
- Which specs you set to `active`, which acceptance criteria you ticked `[x]` or left `[ ]`, and why.
- The format, lints, tests, `spex check` and `spex coverage` run outcome.

## Step 7 — Point to the next skill

Ask the user to do an optional review with `spex-review` for implemented changes.
