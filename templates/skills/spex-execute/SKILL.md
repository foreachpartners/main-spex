---
name: spex-execute
description: Execute an existing implementation plan, or carry out an ad-hoc user request directly against the codebase. Use when the user wants to run, implement, or carry out a plan from <workspace-root>/<project-name>-spex/plans/, or asks for a change without a formal plan.
---

# Execute a Plan or a User Request

An implementation plan is an ordered set of steps whose tasks AI agents carry out. A user request is any change asked for directly, without a plan. Both follow the same job: find what governs the work, gather its specs, implement, verify, finalize, and report. A plan already states most of that; a bare request means deriving it yourself. Treat the two as one flow below — branch only where a plan changes what you'd otherwise work out on your own.

You coordinate the work — you do not redo what an agent already did. Each task has a disjoint write scope, so parallel agents never collide.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the lead spec's `id`.

## Step 1 — Find what governs the work

Decide whether a plan governs this work:

- The user is asking to execute a plan — by filename, by spec ID, by "the plan I just made," or just generally when a plan clearly applies — resolve which file: use the named file; with only a spec ID, look under `<workspace-root>/<project-name>-spex/plans/<spec-id>/` and, if it holds several, pick the most recent by the `<date>` in the filename and confirm; with nothing more specific, list the plan directories under `<workspace-root>/<project-name>-spex/plans/` and ask which to execute. Do not invent a plan.
- The user describes a change, bug fix, or feature directly, and no plan exists for it — there is no plan; the request itself is the only unit of work.
- It's ambiguous whether one should exist first (the request is large or spans multiple repositories) — ask the user: plan it first with `spex-plan`, or execute directly. Do not silently skip planning for large, multi-step work.

From here, "the plan" means the file you resolved; where none exists, every step below reduces to the request itself.

## Step 2 — Gather context (REQUIRED, never skip — even for a small or obvious fix)

Every run needs the same four things before you touch code: the affected repositories, the governing rule-specs, the governing capability specs, and the units of work to perform. A plan already states them; without one, derive them yourself.

- **Repositories** — the plan's `Scope`, or, without a plan, whichever repositories the request touches (ask if unclear, or infer from `<workspace-root>/<project-name>-spex/spex.yaml` and each repository's `description`).
- **Rule-specs** — the plan's `Rule-specs`, or, without a plan, the union of each affected repository's `rs_specs` list in `spex.yaml` (repository-wide specs are already expanded into it). Read every one in full from `<workspace-root>/<project-name>-spex/rs-specs/<spec-id>.yaml` — not a summary — so naming, structure, security, traceability, and annotation rules are settled before you write a line of code.
- **Capability specs** — the plan's `Specs`, or, without a plan, every spec under `<workspace-root>/<project-name>-spex/cs-specs/` whose `scope` includes an affected repository and whose subject overlaps the request. Read every relevant one in full from `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml` so the change stays consistent with its existing acceptance criteria. If a bare request extends or changes behavior an existing spec's acceptance criteria already cover, tell the user and offer to update that spec first with `spex-capability-spec`.
- **Units of work** — the plan's steps and tasks, each with its current `Status`, its `Contract changes`, and its `Test cases`; or, without a plan, the request itself as a single unit (Step 3 splits it further if needed).

For each affected repository, resolve its `path` from `spex.yaml` and note its build/test commands and annotation conventions; repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role.

## Step 3 — Implement

Work through the units of work in order, top to bottom; do not start one until the previous is complete.

- With a plan, its steps are already ordered — skip any step or task already `completed` and resume at the first incomplete one.
- Without a plan, split the request into tasks only when it spans independent repositories or clearly separable files; most requests are a single task.

Within a unit, run every task in parallel, one AI agent per task — safe because each has a disjoint write scope. A unit with a single task runs as one agent.

Give each agent only its task: the description, the repository it runs in, the files it owns, the governing rs-specs that apply there, the spec IDs to annotate per `RS-SPEC-ID-TRACING` (once at the boundary, or the first file in a relation chain — never repeated across the nested helpers it calls), and any relevant contract changes and test cases. Do not let an agent edit files outside its task's scope. Instruct it to read the named rs-spec files in full before editing — not a summary — so it complies with the authoritative source, not a paraphrase.

As each task finishes, verify its result against its description and the acceptance criteria it serves. With a plan, edit the file in place as you go — never in a batch at the end: set the task's `Status:` line to `completed` immediately, and once every task in a step is completed, set the step's `Status:` line too. Use only `not-started` or `completed`, and leave every other line of the plan untouched — keeping status current lets an interrupted run resume from the first incomplete step.

If a task fails or an agent cannot complete it, stop, report what failed, and ask the user how to proceed rather than pressing on with dependent work.

## Step 4 — Verify

Prove the work respects the governing rs-specs:

```sh
spex check --repo <repo-a> <repo-b>
```

Scope it to the repositories from Step 2. Run from inside the workspace (any directory under it; `spex` finds the workspace config).

- Fix any `error` finding — make the focused change in the owning repository, or dispatch an agent scoped to it — then re-run `spex check` for that repository until it passes.
- Report `warn` findings to the user but do not block on them unless asked.

Run the affected repositories' own tests to prove each acceptance criterion holds — the plan's test cases, or the request's own. Then check spec coverage:

```sh
spex coverage
```

## Step 5 — Finalize the capability specs

For every capability spec the work implemented or advanced — the plan's `Specs`, or a spec you identified in Step 2 — open `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml`:

- Tick the checkbox of every acceptance criterion the run implemented, `[ ]` to `[x]`; leave any criterion not yet implemented as `[ ]` so `spex coverage` keeps reporting it.
- Once `spex check` passes, set `status:` to `active` (add the line if absent — it defaults to `planned`). This brings the capability under `spex coverage`.

Touch only the `status` line and the acceptance-criteria checkboxes — leave the rest of each spec untouched.

## Step 6 — Report

Summarize the run:

- Which units of work completed, and any skipped because they were already `completed`.
- The `spex check` outcome per affected repository, and any findings fixed or left as `warn`.
- Which acceptance criteria are now proven, and any test cases still outstanding.
- Which specs you set to `active`, which acceptance criteria you ticked `[x]` or left `[ ]`, and why.
- Any step you stopped, and why.
- Any spec you flagged in Step 2 for a follow-up update.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
