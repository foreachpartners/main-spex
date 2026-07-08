---
name: spex-plan
description: Create a new implementation plan for one or more capability specs, or edit an existing plan. Use when the user wants to plan, break down, prepare, update, or amend implementation for a CS spec, or change a named existing plan. Reads the spec, contract, the affected repositories, and the es-specs that govern them, inspects the current code, then writes or updates a step-by-step implementation plan that AI agents can do in parallel.
---

# Create or Edit an Implementation Plan

An implementation plan turns one or more approved capability specs into an ordered set of steps that AI agents execute. Your job is to read the specs, the repositories they touch, and the es-specs that govern those repositories, study the current code, then write one plan file split into steps whose tasks can run in parallel — or, when the user names an existing plan, to revise its steps, tasks, or test cases in place.

The plan is a working document: it records what to build, the governing es-specs, the test cases that prove each acceptance criterion, and a status on every step and task so progress is trackable.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the lead spec's `id`.

Each spec's plans live in their own directory: `<workspace-root>/<project-name>-spex/plans/<spec-id>/<date>-<slug>.md`. The schema at `<workspace-root>/<project-name>-spex/schemas/plan-filename.schema.json` is authoritative for the filename rules.

## When to use this skill

Use it when the user wants to plan or prepare the implementation of one or more CS specs — either named (for example `CS-PLACE-PIZZA-ORDER`) or the spec just last created — or to update, amend, or re-plan a named existing plan file. Do not use it to author a spec, draft a contract, or to execute a plan. The spec must exist and be approved before planning; its contract if applicable should be drafted first so the plan builds on a settled interface.

## Step 0 — Establish create vs. edit, and the target specs

First decide whether this is a new plan or a change to one that already exists:

- If the user names an existing plan (a filename, a path under `<workspace-root>/<project-name>-spex/plans/`, or "the plan for `<spec-id>`" when a plan is already there) or asks to update, revise, re-plan, or change a plan, this is an **edit**. Locate it (see `spex-execute`'s Step 0 for how to resolve a plan from a spec ID when several exist) and read it in full — this is the file you will modify, not replace. Confirm with the user what should change (added, removed, or reordered steps/tasks, revised test cases, a changed scope), then jump to Step 5 using the file's existing steps as the base rather than drafting from scratch — re-run Steps 1–4 only for the parts that changed, so the update stays grounded in the specs, contract, and code. Preserve every untouched line, its ordering, and formatting, and keep each unchanged step/task's `Status:` as-is.
- Otherwise, this is a **new plan** — continue below.

Determine which specs to plan before doing anything else:

- If the user named one or more spec IDs, plan those.
- If the user said "the one I just made" or gave no ID but a spec was created earlier in this session, plan that one and confirm the ID.
- If no spec is identified, ask which to plan. Do not invent a spec.

When several specs are given, pick the lead spec (the one the others depend on, or the first named) — its `id` names the plan directory. List every targeted spec inside the plan.

Before continuing, confirm two preconditions:

- Each target spec's file exists at `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml`. If a spec does not exist yet, stop and create it first with `spex-capability-spec`, then return here. Do not plan a spec that has not been written and approved.
- The contract is settled when applicable. If any acceptance criterion crosses a repository boundary (it needs an endpoint, message, or schema), the affected contract must be drafted or updated with `spex-contract` before planning. If the work needs no interface change (it is internal to one repository), proceed straight from spec to plan with no contract.

## Step 1 — Read the specs

For each targeted spec, read `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml` in full. Capture its `title`, `description`, every `acceptance_criteria`, its `scope` (the in-scope repositories), and `blocked_by`.

If a spec lists `blocked_by`, those are specs that must be in place first. Confirm with the user whether to plan around them or stop.

## Step 2 — Resolve scope and governing es-specs

From `<workspace-root>/<project-name>-spex/spex.yaml` (repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role):

- Collect the union of every targeted spec's `scope` — these are the affected repositories.
- For each affected repository, read its `es_specs` list. The union of those lists, deduplicated, is the set of es-specs that govern this work. (`spex.yaml` already expands repo-wide specs into each repository, so the per-repo lists are authoritative.)
- Read each governing es-spec in `<workspace-root>/<project-name>-spex/es-specs/` so the plan and its tasks respect them (naming, annotations, secrets, contract conventions, testing).

Record which repository each governing es-spec comes from so a task in one repository only carries the es-specs that apply there.

## Step 3 — Inspect the current code

For each affected repository, resolve its `path` and study the current state so the plan extends real code instead of guessing. Use the repository `description`s to identify each one's role (backend, frontend, mobile, and so on); interface contracts live in the `spex` repository's `contracts/` folder, listed per repository under `contracts` in `spex.yaml`:

- Find the contract(s) the spec touches — in the `spex` repository's `contracts/` folder — and read the relevant endpoints, messages, or schemas.
- Read the handlers, modules, and tests nearest to the change, and note an existing feature to copy the structure from (the "by example" pattern: how a similar endpoint, screen, or test is already built).
- Note the build/test commands and any annotation conventions the repository already uses.

## Step 4 — Design the steps and tasks

Focus on general picture, not on details, show database/domain model changes, new/changed methods signatures, not their implementation. Prefer bullet points for plan.

Break the work into ordered **steps**. Within a step, split work into **tasks** only when there is more than one independent unit.

The rule that defines a step boundary:

- Every task inside one step MUST be safe to run in parallel by separate AI agents. Tasks in the same step must not depend on each other's output and must not edit the same files.
- Put work that depends on an earlier result in a later step. A contract change comes before the implementation that consumes it; implementation comes before the tests that exercise it — unless tests are written against the contract, in which case they can be parallel.
- Give each task a disjoint write scope (distinct files or repositories) so parallel agents never collide. State the repository each task runs in.

Keep steps as few as the dependencies allow; maximize parallelism within each step. A small spec may be a single step with a single task.

When a task introduces or changes a table, schema, or domain model, show it in its native structure instead of describing it in prose:

- A database table: a Markdown table with one row per column, naming the column, type, and constraints (PK/FK, nullability, defaults).
- A domain model (Rust struct, TypeScript interface, and so on): a fenced code block in that language showing the fields and their types.

Follow the structure with a sentence or two of prose only for what the structure cannot express — invariants, behavior, methods to add.

## Step 5 — Draft the plan

Write the plan in Markdown using the template below. Keep the implementation overview short — a few sentences per repository, not a design essay. Derive test cases directly from the acceptance criteria so every criterion is provably covered. The repository names in the template (`api`, `frontend`) are placeholders; use the real names and roles from `spex.yaml`.

# Plan: <lead spec title>

- Specs: CS-LEAD-SPEC, CS-OTHER-SPEC
- Scope: api, frontend
- Engineering-specs: ES-SECRET-HANDLING, ES-NAMING-CONVENTION, ES-SPEC-ID-ANNOTATIONS, ES-OPENAPI-CONVENTIONS

## Overview

<Short implementation overview: what changes in each affected repository and how it follows the existing code by example. A few sentences per repository.>

## Steps

### Step 1 — <name>

Status: not-started

#### Task 1.1 — <name> (repo: spex, contracts/)

Status: not-started

<What to do as bullet points, database/domain model changes, new/changed methods signatures, not their implementation>

#### Task 1.2 — <name> (repo: frontend)

Status: not-started

<...>

<Example of showing a table change instead of describing it in prose:>

| Column                | Type        | Constraints         |
| --------------------- | ----------- | ------------------- |
| user_id               | uuid        | PK, FK -> users.id  |
| failed_login_attempts | int         | not null, default 0 |
| locked_until          | timestamptz | nullable            |
| version               | int         | not null, default 0 |

<Example of showing a domain model change instead of describing it in prose:>

```rust
struct Auth <
    user_id: UserId,
    failed_login_attempts: u32,
    locked_until: Option<DateTime<Utc>>,
    version: u32,
>
```

### Step 2 — <name>

Status: not-started

<A step with a single unit needs no task subheadings; give it one Status and its details.>

## Test cases

Each case maps to an acceptance criterion and proves it.

- [CS-LEAD-SPEC / AC description] <case>: <given / when / then>
- [CS-LEAD-SPEC / AC description] <case>: <given / when / then>

Rules for the plan content:

- The plan lives at `<spec-id>/<date>-<slug>.md`, where `<spec-id>` is the lead spec chosen in Step 0. `<date>` is today's date as `YYYY-MM-DD`; get the real current date, do not invent it. `<slug>` is a short lowercase hyphenated summary of the plan (for example `add-list-pizza-menu-endpoint`). The filename stem `<date>-<slug>` must validate against `<workspace-root>/<project-name>-spex/schemas/plan-filename.schema.json` and stay within 50 characters (for example `CS-PLACE-PIZZA-ORDER/2024-11-24-add-place-order-endpoint.md`). Multiple plans can coexist in one spec's directory; never overwrite an existing one.
- List every targeted spec ID under `Specs`, the affected repositories under `Scope`, and the combined unique governing es-specs under `Engineering-specs`.
- Every step has a `Status:` line. Every task has its own `Status:` line. Both use `not-started` or `completed`; write all as `not-started` in a fresh plan.
- Cover every acceptance criterion of every targeted spec with at least one test case, and reference the criterion it proves.
- Within each task, name the applicable es-specs and the spec to annotate per `ES-SPEC-ID-ANNOTATIONS` so the executing agent stays compliant. State which single place carries the annotation — once at the boundary, or the first file in a relation chain (handler, not the service and repository it calls) — so the spec is not repeated through nested helpers.

## Step 6 — Write, then offer edits

1. For a new plan, write it to `<workspace-root>/<project-name>-spex/plans/<spec-id>/<date>-<slug>.md`, creating the spec's plan directory if it does not exist; the filename stem must validate against `<workspace-root>/<project-name>-spex/schemas/plan-filename.schema.json`. For an edit, update the existing plan file in place at its current path — never rename or duplicate it.
2. Tell the user the path you wrote (or the changes you applied, for an edit), walk through the step boundaries and which tasks run in parallel, then ask whether they want to change anything further. Apply any requested edits in place.

## Step 7 — Point to the next skill

After the plan is written, offer to execute it: run `spex-execute <spec-id>`. Note that it is better to start a new chat first to save context.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
