---
name: spex-execute-plan
description: Execute an existing implementation plan. Use when the user wants to run, implement, or carry out a plan from <workspace-root>/spex/plans/. Reads the plan and the specs, repositories, and tech-specs it names, then works through the steps in order — running the parallel tasks within each step as separate AI agents, marking each step and task completed as it lands, running `spex check` on the affected repositories, and setting each implemented business spec's status to active.
---

# Execute an Implementation Plan

An implementation plan is an ordered set of steps whose tasks AI agents carry out. Your job is to read one plan, then work through its steps in order: within a step, run the independent tasks in parallel as separate agents; between steps, wait for the earlier step to finish before starting the next. As each task and step lands, mark it completed in the plan file, then run `spex check` on the affected repositories to prove the work respects the governing tech-specs.

You coordinate the work — you do not redo what an agent already did. Each task in a step has a disjoint write scope, so parallel agents never collide. When the plan's steps pass, set each business spec it implemented to `active` so the capability comes under `spex coverage`.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the lead spec's `id`.

## When to use this skill

Use it when the user wants to execute, run, or implement an existing plan. Do not use it to author a plan (use `spex-create-plan`), to author specs (`spex-create-biz-spec`, `spex-create-tech-spec`), or to draft a contract (use `spex-create-contract`). The plan must already exist in `<workspace-root>/spex/plans/`.

## Step 0 — Establish the target plan

Determine which plan to execute before doing anything else:

- If the user named a plan (a filename, a spec ID, or "the plan I just made"), use it. When only a spec ID is given, look in `<workspace-root>/spex/plans/<spec-id>/`; if it holds several plans, pick the most recent by the `<date>` in the filename and confirm.
- If the user named nothing, list the plan directories in `<workspace-root>/spex/plans/` and their plans, then ask which to execute. Do not invent a plan.

## Step 1 — Read the plan and its context

Read the chosen plan in `<workspace-root>/spex/plans/<spec-id>/<date>_<slug>.md` in full. Capture its `Specs`, `Scope` (the affected repositories), `Tech-specs`, the `Contract changes`, every step and its tasks with their current `Status`, and the `Test cases`.

Then read enough context to execute faithfully:

- Load the combined context of the plan's specs and governing tech-specs in one call, passing every spec ID from the plan's `Specs` and `Tech-specs`:
  ```sh
  spex spec-context <spec-id> <tech-spec-id>...
  ```
  This prints each spec's `id`, `title`, `description`, and `acceptance_criteria` (plus `examples` for tech-specs) as Markdown — enough for you to coordinate against. Run it from anywhere inside the workspace.
- For each affected repository, resolve its `path` from `<workspace-root>/spex/spex.yaml` and note its build/test commands and annotation conventions. Repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role.

The agents that edit code must still read the full tech-spec files (Step 3), since `spex spec-context` omits the checks; use it to orient yourself, not as the authoritative source for compliance.

If any step or task is already `completed`, skip it and resume from the first incomplete step.

## Step 2 — Confirm the run

Show the user the plan's steps, which tasks will run in parallel within each step, and the affected repositories. Confirm before executing. If the plan looks wrong or stale, say so and suggest re-planning with `spex-create-plan` rather than executing a broken plan.

## Step 3 — Execute the steps in order

Work step by step, top to bottom. Do not start a step until the previous one is complete.

Within a step:

- Run every task in the step in parallel, one AI agent per task. The tasks are independent and have disjoint write scopes by construction, so they are safe to run together.
- Give each agent only its task: the task description, the repository it runs in, the files it owns, the governing tech-specs that apply there, the spec IDs to annotate per `TECH-SPEC-ID-ANNOTATIONS`, and the relevant contract changes and test cases. Do not let an agent edit files outside its task's scope.
- Instruct each agent to read the named tech-spec files in full from `<workspace-root>/spex/tech-specs/` before editing — not a summary — so it complies with the authoritative source rather than a paraphrase.
- A step with a single unit runs as one agent.

As each task finishes, verify its result against the task description and the acceptance criteria it serves, then mark it completed (Step 4). Only when every task in the step is completed do you move to the next step.

If a task fails or an agent cannot complete it, stop the step, report what failed, and ask the user how to proceed rather than pressing on with dependent steps.

## Step 4 — Update status as work lands

Edit the plan file in place as you go — never at the end in a batch:

- When a task finishes and is verified, set its `Status:` line to `completed`.
- When every task in a step is completed, set the step's `Status:` line to `completed`.
- Use only `not-started` or `completed`, matching the plan's existing status vocabulary. Leave every other line of the plan untouched.

Keeping status current means an interrupted run can resume from the first incomplete step.

## Step 5 — Run checks on the affected repositories

After the steps are complete, prove the work respects the governing tech-specs by running `spex check` scoped to the affected repositories:

```sh
spex check --repo <repo-a> <repo-b>
```

Pass the repositories from the plan's `Scope`. Run from inside the workspace (any directory under it; `spex` finds the workspace config). Read the findings:

- Fix any `error` findings — make the focused change in the owning repository, or dispatch an agent scoped to that repository to fix it, then re-run `spex check` for that repository until it passes.
- Report `warn` findings to the user but do not block on them unless the user asks.

Run the repositories' own tests where the plan's test cases call for them, to prove each acceptance criterion holds.

## Step 6 — Mark the implemented specs active

Once the steps are complete and `spex check` passes, set each business spec the plan implemented to `active`. For every spec ID in the plan's `Specs`, open `<workspace-root>/spex/biz-specs/<spec-id>.yaml` and set its `status:` line to `active` (add the line if absent — `status` defaults to `planned`). This signals the capability is implemented and brings it under `spex coverage`.

Leave a spec's status unchanged if its acceptance criteria are not yet fully implemented; instead set `dirty: true` so it is clear the spec has moved ahead of the code. Touch only the `status` (and, where needed, `dirty`) line — leave the rest of each spec untouched.

## Step 7 — Report the result

Summarize the run for the user:

- Which steps and tasks completed, referenced by their numbers.
- The `spex check` outcome per affected repository, and any findings you fixed or left.
- Which acceptance criteria are now proven, and any test cases still outstanding.
- Which specs you set to `active`, and any you left `planned` or marked `dirty`, with the reason.
- Any step that was skipped (already completed) or stopped, with the reason.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
