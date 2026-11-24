---
name: spex-implement
description: Carry out an existing plan, or implement an ad-hoc user request. Use when the user wants to carry out a plan from <spex-dir>/plans/<plan-name>, or asks for a change without a formal plan.
---

# Process

**You MUST follow each process step without skipping**, no matter whether you are implementing an existing plan or an ad-hoc user request.

## Step 1 — Gather context

**Do not edit or write any code files until this step is done; do not skip it**.

For a plan, take the IDs and packages from its `Specs` and `Packages`. For an ad-hoc request, determine the affected packages and the spec IDs that govern the change. Use the spec ID and criterion named in the request when they cover the entire change.

Assemble the task context once and read it before editing.

```sh
spex context --id <spec-id-a> <spec-id-b> --package <package-a> <package-b> > <tmp-path>
```

Use the bundle's spec criteria, package scope and rules. Open its file references for details needed by the task. When the change has no governing spec because it changes no behavior, read the package rules directly with `spex rules --package <package>`. Resolve the relevant spec before changing behavior. Tell the user which specs and rules govern the work.

## Step 2 — Implement

### With a plan

Its steps are already ordered — skip any step or task already `completed` and resume at the first incomplete one.

Edit the `Status` in place as you go. Use only `not-started`, `in-progress` and `completed`. Leave every other line of the plan untouched.

### Without a plan (ad-hoc user request)

Split the request into tasks only when it spans independent packages or clearly separable files; most requests are a single task.

### Both paths

Run a single task directly. For independent tasks, use parallel agents when the runtime supports them and the user permits it. Otherwise work through the tasks in the order required by their dependencies.

If a task fails or an agent cannot complete it, stop, report what failed, and ask the user how to proceed rather than pressing on with dependent work.

This step implements the change only — it does not write tests. Use `spex-test` for that once implementation is done.

As you implement each acceptance criterion, annotate its boundary with `@spec impl <spec-id>#<ac-id>` (or, for a contract, an OpenAPI `x-*` vendor extension whose value is `@spec contract <spec-id>` — anchored per criterion as `@spec contract <spec-id>#<ac-id>` when the spec's `contract` is `for-each-ac`), tagging the specific criterion id it satisfies. `spex coverage` traces active and in-progress specs per criterion from these annotations, so a criterion implemented without one still shows as a gap. A criterion flagged `is_done: false` stays pending until its impl annotation exists; once it does, `spex coverage` reports its missing test (and contract, where required) as a gap too.

Place each annotation on the smallest unit that actually satisfies the criterion — a specific field, function, or handler — not on the file, the enclosing type, or a whole module. Widen the placement only when the criterion's scope genuinely spans that boundary (e.g. an endpoint, contract). Keep the total number of annotations minimal.

## Step 3 — Finalize the specs

For every spec the work implemented or advanced, open its path from the task context:

- If `status:` is absent or `planned`, set it to `in-progress` (add the line if absent). Never set it to `active` here — this skill doesn't write tests, so it can't know a criterion is actually done; that determination, and clearing each criterion's `is_done: false`, is `spex-test`'s job once it verifies impl+test tracing with `spex coverage`.

Touch only the `status` line — leave the rest of each spec, including `is_done`, untouched.

## Step 4 — Verify

Run formatters, linters and the existing tests relevant to the change. For a routine change confined to the implementation of one existing acceptance criterion in one package, run targeted tests unless the package rules require a broader suite.

Prove the work respects the governing rules:

```sh
spex check --package <package-a> <package-b>
```

When there are governing specs, inspect their traces:

```sh
spex coverage --id <spec-id-a> <spec-id-b>
```

A missing `test` trace for a criterion you just annotated is expected here. Address the other gaps introduced by the change. List inherited gaps that affect the task.

## Step 5 — Delete the plan

### With a plan

Once every step and task in the plan is `completed`, delete the plan file. A finished plan is a working document with nothing left to say — the spec, code, git and tests already record what shipped. Leaving it in the repository only pollutes future search with an obsolete artifact.

If any step or task is not yet `completed` (the run stopped partway through), leave the plan file in place so work can resume later.

Don't ask user for confirmation about plan deletion.

### Without a plan (ad-hoc user request)

Nothing to delete.

## Step 6 — Report

Summarize the run:

- Which units of work completed.
- Which specs you set to `in-progress` (or left as-is), and which acceptance criteria `spex coverage` still shows untraced.
- The rule checks, trace coverage, contract validation, tests executed and manual acceptance outcomes that apply to this change. Identify any gate that remains pending.
- The task context path and digest for the reviewer and test author. Reuse the bundle while its governing specs, rules and contracts stay current. Update it when those inputs change.
- Whether the plan file was deleted (fully completed) or left in place (partial run).

## Step 7 — Point to the next skill

Ask the user to do an optional review with `spex-review`, then to add test coverage for the change with `spex-test`.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
