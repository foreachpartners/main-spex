---
name: spex-create-plan
description: Draft an implementation plan for one or more business specs. Use when the user wants to plan, break down, or prepare implementation for a BIZ spec (named, or the one just created). Reads the spec, the affected repositories, and the tech-specs that govern them, inspects the current code, then writes a step-by-step plan to <workspace-root>/spex/plans/<spec-id>/<date>_<slug>.md whose steps hold tasks that AI agents can run in parallel.
---

# Create an Implementation Plan

An implementation plan turns one or more approved business specs into an ordered set of steps that AI agents execute. Your job is to read the specs, the repositories they touch, and the tech-specs that govern those repositories, study the current code, then write one plan file split into steps whose tasks can run in parallel.

The plan is a working document: it records what to build, the contract changes, the governing tech-specs, the test cases that prove each acceptance criterion, and a status on every step and task so progress is trackable.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the lead spec's `id`.

Each spec's plans live in their own directory: `<workspace-root>/spex/plans/<spec-id>/<date>_<slug>.md`. The schema at `<workspace-root>/spex/schemas/plan-filename.schema.json` is authoritative for the filename rules.

## When to use this skill

Use it when the user wants to plan or prepare the implementation of one or more BIZ specs — either named (for example `BIZ-PLACE-PIZZA-ORDER`) or the spec just created in this session. Do not use it to author a spec (use `spex-create-biz-spec`), to draft a contract (use `spex-create-contract`), or to execute a plan (use `spex-execute-plan`). The spec must exist and be approved before planning; its contract should be drafted first so the plan builds on a settled interface.

## Step 0 — Establish the target specs

Determine which specs to plan before doing anything else:

- If the user named one or more spec IDs, plan those.
- If the user said "the one I just made" or gave no ID but a spec was created earlier in this session, plan that one and confirm the ID.
- If no spec is identified ask which to plan. Do not invent a spec.

When several specs are given, pick the lead spec (the one the others depend on, or the first named) — its `id` names the plan directory. List every targeted spec inside the plan.

Before continuing, confirm two preconditions:

- Each target spec's file exists at `<workspace-root>/spex/biz-specs/<spec-id>.yaml`. If a spec does not exist yet, stop and create it first with `spex-create-biz-spec`, then return here. Do not plan a spec that has not been written and approved.
- The contract is settled when applicable. If any acceptance criterion crosses a repository boundary (it needs an endpoint, message, or schema), the affected contract must be drafted or updated with `spex-create-contract` before planning. If the work needs no interface change (it is internal to one repository), proceed straight from spec to plan with no contract.

## Step 1 — Read the specs

For each targeted spec, read `<workspace-root>/spex/biz-specs/<spec-id>.yaml` in full. Capture its `title`, `description`, every `acceptance_criteria`, its `scope` (the in-scope repositories), and `depends_on`.

If a spec lists `depends_on`, check those specs `status`. Flag any dependency that is not yet shipped and confirm with the user whether to plan around it or stop.

## Step 2 — Resolve scope and governing tech-specs

From `<workspace-root>/spex/spex.yaml` (repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role):

- Collect the union of every targeted spec's `scope` — these are the affected repositories.
- For each affected repository, read its `tech_specs` list. The union of those lists, deduplicated, is the set of tech-specs that govern this work. (`spex.yaml` already expands repo-wide specs into each repository, so the per-repo lists are authoritative.)
- Read each governing tech-spec in `<workspace-root>/spex/tech-specs/` so the plan and its tasks respect them (naming, annotations, secrets, contract conventions, testing).

Record which repository each governing tech-spec comes from so a task in one repository only carries the tech-specs that apply there.

## Step 3 — Inspect the current code

For each affected repository, resolve its `path` and study the current state so the plan extends real code instead of guessing. Use the repository `description`s to identify each one's role (backend, frontend, mobile, and so on); interface contracts live in the `spex` repository's `contracts/` folder, listed per repository under `contracts` in `spex.yaml`:

- Find the contract(s) the spec touches — in the `spex` repository's `contracts/` folder — and read the relevant endpoints, messages, or schemas.
- Read the handlers, modules, and tests nearest to the change, and note an existing feature to copy the structure from (the "by example" pattern: how a similar endpoint, screen, or test is already built).
- Note the build/test commands and any annotation conventions (for example `@spec` tags) the repository already uses.

## Step 4 — Design the steps and tasks

Break the work into ordered **steps**. Within a step, split work into **tasks** only when there is more than one independent unit.

The rule that defines a step boundary:

- Every task inside one step MUST be safe to run in parallel by separate AI agents. Tasks in the same step must not depend on each other's output and must not edit the same files.
- Put work that depends on an earlier result in a later step. A contract change comes before the implementation that consumes it; implementation comes before the tests that exercise it — unless tests are written against the contract, in which case they can be parallel.
- Give each task a disjoint write scope (distinct files or repositories) so parallel agents never collide. State the repository each task runs in.

Keep steps as few as the dependencies allow; maximize parallelism within each step. A small spec may be a single step with a single task.

## Step 5 — Draft the plan

Write the plan in Markdown using the template below. Keep the implementation overview short — a few sentences per repository, not a design essay. Derive test cases directly from the acceptance criteria so every criterion is provably covered. The repository names in the template (`api`, `frontend`) are placeholders; use the real names and roles from `spex.yaml`. Contract changes are made in the `spex` repository's `contracts/` folder, so a contract task runs in `spex`.

```md
# Plan: <lead spec title>

- Specs: BIZ-LEAD-SPEC, BIZ-OTHER-SPEC
- Scope: api, frontend
- Tech-specs: TECH-SECRET-HANDLING, TECH-NAMING-CONVENTION, TECH-SPEC-ID-ANNOTATIONS, TECH-OPENAPI-CONVENTIONS

## Overview

<Short implementation overview: what changes in each affected repository and how it follows the existing code by example. A few sentences per repository.>

## Contract changes

<Endpoints, messages, or schemas added or changed, with method, path, status codes, and errors. State "none" if the contract is unchanged.>

## Steps

### Step 1 — <name>

Status: not-started

#### Task 1.1 — <name> (repo: spex, contracts/)

Status: not-started

<What to do, which files, which tech-specs apply, which spec to annotate.>

#### Task 1.2 — <name> (repo: frontend)

Status: not-started

<...>

### Step 2 — <name>

Status: not-started

<A step with a single unit needs no task subheadings; give it one Status and its details.>

## Test cases

Each case maps to an acceptance criterion and proves it.

- [BIZ-LEAD-SPEC / AC1] <case>: <given / when / then>
- [BIZ-LEAD-SPEC / AC2] <case>: <given / when / then>
```

Rules for the plan content:

- The plan lives at `<spec-id>/<date>_<slug>.md`, where `<spec-id>` is the lead spec chosen in Step 0. `<date>` is today's date as `YYYY-MM-DD`; get the real current date, do not invent it. `<slug>` is a short lowercase hyphenated summary of the plan (for example `add-list-pizza-menu-endpoint`). The filename stem `<date>_<slug>` must validate against `<workspace-root>/spex/schemas/plan-filename.schema.json` and stay within 50 characters (for example `BIZ-PLACE-PIZZA-ORDER/2024-11-24_add-place-order-endpoint.md`). Multiple plans can coexist in one spec's directory; never overwrite an existing one.
- List every targeted spec ID under `Specs`, the affected repositories under `Scope`, and the combined unique governing tech-specs under `Tech-specs`.
- Every step has a `Status:` line. Every task has its own `Status:` line. Both use `not-started` or `completed`; write all as `not-started` in a fresh plan.
- Cover every acceptance criterion of every targeted spec with at least one test case, and reference the criterion it proves.
- Within each task, name the applicable tech-specs and the spec to annotate per `TECH-SPEC-ID-ANNOTATIONS` so the executing agent stays compliant. State which single place carries the annotation — once at the boundary, or the first file in a relation chain (handler, not the service and repository it calls) — so the spec is not repeated through nested helpers.

## Step 6 — Confirm, then write

1. Show the drafted plan to the user — especially the step boundaries and which tasks run in parallel — and confirm. Apply edits if needed.
2. Write it to `<workspace-root>/spex/plans/<spec-id>/<date>_<slug>.md`, creating the spec's plan directory if it does not exist. The filename stem must validate against `<workspace-root>/spex/schemas/plan-filename.schema.json`.
3. Confirm the path back to the user.

## Step 7 — Point to the next skill

After the plan is written, offer to execute it: run `spex-execute-plan <spec-id>`. Note that it is better to start a new chat first to save context.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
