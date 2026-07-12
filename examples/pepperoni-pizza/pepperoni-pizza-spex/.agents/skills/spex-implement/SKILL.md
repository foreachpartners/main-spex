---
name: spex-implement
description: Carry out a plan existing plan, or implement ad-hoc user request. Use when the user wants to carry out a plan from <workspace-root>/<project-name>-spex/plans/<spec-id>/<plan-name>, or asks for a change without a formal plan.
---

# Process

**You MUST follow each process step without skipping** no matter if you implement existing plan or ad-hoc user request.

## Step 1 — Gather context

**Do not edit, or write any code file until this step is done and don't skip it**. Every run needs the same four things:

1. affected repositories aka scope (e.g. frontend, backend)
2. governing rule specs
3. governing capability specs
4. units of work to perform (e.g. plan or ad-hoc user request)

If you carry out existing plan it's already contains all the information, just read it.

If you carry out ad-hoc user request:

1. determinate determinate scope based on affected repositories
2. read all governing rule specs `rs_specs` in `<workspace-root>/<project-name>-spex/spex.yaml` for affected scope.
3. determinate and read governing capability specs in `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml`

## Step 2 — Implement

Do not start until Step 2's context note exists in your response. Work through the units of work in order, top to bottom; do not start one until the previous is complete.

- With a plan, its steps are already ordered — skip any step or task already `completed` and resume at the first incomplete one.
- Without a plan, split the request into tasks only when it spans independent repositories or clearly separable files; most requests are a single task.

Within a unit, run every task in parallel, one AI agent per task.

For a plan, edit the `Status` in place as you go. Use only `not-started`, `in-progress` and `completed`. Leave every other line of the plan untouched.

If a task fails or an agent cannot complete it, stop, report what failed, and ask the user how to proceed rather than pressing on with dependent work.

## Step 3 — Finalize the capability specs

For every capability spec the work implemented or advanced — the plan's `Specs`, or a spec you identified in Step 1 — open `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml`:

- Tick the checkbox of every acceptance criterion the run implemented, `[ ]` to `[x]`; leave any criterion not yet implemented as `[ ]`.
- Change `status:` to `active` (add the line if absent — it defaults to `planned`)..

Touch only the `status` line and the acceptance-criteria checkboxes — leave the rest of each spec untouched.

## Step 4 — Verify

Run formatter, lint's and tests to verify implementation.

Prove the work respects the governing rs-specs:

```sh
spex check --repo <repo-a> <repo-b>
```

Prove the work completes the governing cs-specs:

```sh
spex coverage
```

## Step 5 — Report

Summarize the run:

- Which units of work completed.
- Which specs you set to `active`, which acceptance criteria you ticked `[x]` or left `[ ]`, and why.
- The format, lint's, tests, `spex check` and `spex coverage` run outcome.

## Step 6 — Point to the next skill

Ask user to do optional review with `spex-review` for implemented changes.
