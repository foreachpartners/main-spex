---
name: spex-implement
description: Carry out an existing plan, or implement an ad-hoc user request. Use when the user wants to carry out a plan from <workspace-root>/<project-name>-spex/plans/<plan-name>, or asks for a change without a formal plan.
---

# Process

**You MUST follow each process step without skipping**, no matter whether you are implementing an existing plan or an ad-hoc user request.

## Step 1 — Gather context

**Do not edit or write any code files until this step is done; do not skip it**.

### With a plan

Read the plan. Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the plan's `Specs` to read the governing specs.

### Without a plan (ad-hoc user request)

1. Determine repositories we will work on (e.g. scope: frontend, backend).
2. Run `spex rules --repo <repo-a> <repo-b> --ai --output <tmp-path>` for the affected scope from step 1, then read `<tmp-path>` for the governing rules.
3. Determine all governing spec IDs related to the request.
4. Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the spec IDs from step 3 to read the governing specs.
5. Produce a note in response with a list of governing rules and specs.

### Both paths

Read all specs from step 1 and the governing rules from `<tmp-path>`. Don't skip anything, this is not negotiable!

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

1. Add tests for each completed acceptance criterion of implemented specs.
2. Add all necessary tests for new functionality or changes based on repository testing patterns.

## Step 4 — Finalize the specs

For every spec the work implemented or advanced — the plan's `Specs`, or the specs you identified in Step 1 — open `<workspace-root>/<project-name>-spex/specs/<spec-id>.yaml`:

- Tick the checkbox of every acceptance criterion the run implemented, `[ ]` to `[x]`; leave any criterion not yet implemented as `[ ]`.
- Change `status:` to `active` (add the line if absent — it defaults to `planned`).

Touch only the `status` line and the acceptance-criteria checkboxes — leave the rest of each spec untouched.

## Step 5 — Verify

Run formatters, linters, and tests to verify implementation.

Prove the work respects the governing rules:

```sh
spex check --repo <repo-a> <repo-b>
```

Prove the work completes the governing specs:

```sh
spex coverage
```

## Step 6 — Delete the plan

### With a plan

Once every step and task in the plan is `completed`, delete the plan file. A finished plan is a working document with nothing left to say — the spec, code, git and tests already record what shipped. Leaving it in the repository only pollutes future search with an obsolete artifact.

If any step or task is not yet `completed` (the run stopped partway through), leave the plan file in place so work can resume later.

### Without a plan (ad-hoc user request)

Nothing to delete.

## Step 7 — Report

Summarize the run:

- Which units of work completed.
- Which specs you set to `active`, which acceptance criteria you ticked `[x]` or left `[ ]`, and why.
- The format, lints, tests, `spex check` and `spex coverage` run outcome.
- Whether the plan file was deleted (fully completed) or left in place (partial run).

## Step 8 — Point to the next skill

Ask the user to do an optional review with `spex-review` for implemented changes.
