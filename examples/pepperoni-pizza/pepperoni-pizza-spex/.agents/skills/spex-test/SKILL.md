---
name: spex-test
description: Add or update tests for an already-implemented change. Use after spex-implement, or whenever the user wants test coverage added for existing code, from a plan, a diff, or an ad-hoc request.
---

# Process

**You MUST follow each process step without skipping.**

This skill only writes tests. It never edits plan files, spec files, or non-test code — implementation is `spex-implement`'s job.

## Step 1 — Gather context

**Do not write any test files until this step is done; do not skip it**.

### With a plan

Read the plan. Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the plan's `Specs` to read the governing specs.

### Without a plan (ad-hoc request, or testing an existing diff)

1. Determine repositories we will work on (e.g. scope: frontend, backend).
2. Run `spex rules --repo <repo-a> <repo-b> --ai --output <tmp-path>` for the affected scope from step 1, then read `<tmp-path>` for the governing rules.
3. Determine all governing spec IDs related to the change — from the request, or from `@spec` annotations already in the changed code.
4. Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the spec IDs from step 3 to read the governing specs.
5. Produce a note in response with a list of governing rules and specs.

### Both paths

Read all specs from step 1 and the governing rules from `<tmp-path>`. Inspect the actual implementation under test — don't test from the plan or spec text alone. Don't skip anything, this is not negotiable!

## Step 2 — Add tests

1. For each governing spec, add tests for every acceptance criterion already implemented (`[x]`) that isn't yet covered.
2. Add any other tests the implementation needs based on repository testing patterns.
3. Annotate new tests with their spec IDs where the governing rules require test-layer traceability.

Split into tasks only when work spans independent repositories or clearly separable files; most requests are a single task. Within a unit, run every task in parallel, one AI agent per task.

If a task fails or an agent cannot complete it, stop, report what failed, and ask the user how to proceed rather than pressing on with dependent work.

## Step 3 — Verify

Run formatters, linters, and the test suite to confirm the new tests pass and nothing else broke.

Prove the new tests respect the governing rules:

```sh
spex check --repo <repo-a> <repo-b>
```

## Step 4 — Report

Summarize the run:

- Which units of work completed.
- Which specs and acceptance criteria you added coverage for.
- The format, lints, tests, and `spex check` run outcome.
