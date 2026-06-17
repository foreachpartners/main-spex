---
name: spex-create-plan
description: Draft an implementation plan for one or more business specs. Use when the user wants to plan, break down, or prepare implementation for a BIZ spec (named, or the one just created). Reads the spec, the affected repositories, and the tech-specs that govern them, inspects the current code, then writes a step-by-step plan to <workspace-root>/spex/plans/<spec-id>.md whose steps hold tasks that AI agents can run in parallel.
---

# Create an Implementation Plan

An implementation plan turns one or more approved business specs into an ordered set of steps that AI agents execute. Your job is to read the specs, the repositories they touch, and the tech-specs that govern those repositories, study the current code, then write one plan file split into steps whose tasks can run in parallel.

The plan is a working document: it records what to build, the contract changes, the governing tech-specs, the test cases that prove each acceptance criterion, and a status on every step and task so progress is trackable.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the lead spec's `id`.

## When to use this skill

Use it when the user wants to plan or prepare the implementation of one or more BIZ specs — either named (for example `BIZ-PLACE-PIZZA-ORDER`) or the spec just created in this session. Do not use it to author a spec (use `spex-create-biz-spec`) or to execute a plan (use `spex-execute-plan`). The spec must exist and be approved before planning.

## Step 0 — Establish the target specs

Determine which specs to plan before doing anything else:

- If the user named one or more spec IDs, plan those.
- If the user said "the one I just made" or gave no ID but a spec was created earlier in this session, plan that one and confirm the ID.
- If no spec is identified ask which to plan. Do not invent a spec.

When several specs are given, pick the lead spec (the one the others depend on, or the first named) — its `id` names the file. List every targeted spec inside the plan.

## Step 1 — Read the specs

For each targeted spec, read `<workspace-root>/spex/biz-specs/<spec-id>.yaml` in full. Capture its `title`, `description`, every `acceptance_criteria`, its `scope` (the in-scope repositories), and `depends_on`.

If a spec lists `depends_on`, check those specs `status`. Flag any dependency that is not yet shipped and confirm with the user whether to plan around it or stop.

## Step 2 — Resolve scope and governing tech-specs

From `<workspace-root>/spex/spex.yml`:

- Collect the union of every targeted spec's `scope` — these are the affected repositories.
- For each affected repository, read its `tech-specs` list. The union of those lists, deduplicated, is the set of tech-specs that govern this work. (`spex.yml` already expands repo-wide specs into each repository, so the per-repo lists are authoritative.)
- Read each governing tech-spec in `<workspace-root>/spex/tech-specs/` so the plan and its tasks respect them (naming, annotations, secrets, contract conventions, testing).

Record which repository each governing tech-spec comes from so a task in one repository only carries the tech-specs that apply there.

## Step 3 — Inspect the current code

For each affected repository, resolve its `path` and study the current state so the plan extends real code instead of guessing:

- Find the contract(s) the spec touches and read the relevant endpoints, messages, or schemas.
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

Write the plan in Markdown using the template below. Keep the implementation overview short — a few sentences per repository, not a design essay. Derive test cases directly from the acceptance criteria so every criterion is provably covered.

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

#### Task 1.1 — <name> (repo: contracts)

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

- The filename is `<lead-spec-id>.md`. Only when that file already exists, append the current local time to keep both: `<lead-spec-id>_<datetime>.md` where `<datetime>` is `YYYY-MM-DD-HH-MM` (for example `BIZ-PLACE-PIZZA-ORDER_2024-11-24-14-44.md`). Get the real current time; do not invent it.
- List every targeted spec ID under `Specs`, the affected repositories under `Scope`, and the combined unique governing tech-specs under `Tech-specs`.
- Every step has a `Status:` line. Every task has its own `Status:` line. Both use `not-started` or `completed`; write all as `not-started` in a fresh plan.
- Cover every acceptance criterion of every targeted spec with at least one test case, and reference the criterion it proves.
- Within each task, name the applicable tech-specs and the spec to annotate per `TECH-SPEC-ID-ANNOTATIONS` so the executing agent stays compliant.

## Step 6 — Confirm, then write

1. Show the drafted plan to the user — especially the step boundaries and which tasks run in parallel — and confirm. Apply edits if needed.
2. Write it to `<workspace-root>/spex/plans/<lead-spec-id>.md`. If that file already exists, write to `<workspace-root>/spex/plans/<lead-spec-id>_<datetime>.md` instead so the existing plan is preserved.
3. Confirm the path back to the user.

## Step 7 — Point to the next skill

After the plan is written, prompt the user to execute it: run `spex-execute-plan <lead-spec-id>`.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
