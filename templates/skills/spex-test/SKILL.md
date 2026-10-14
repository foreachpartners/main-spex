---
name: spex-test
description: Add or update tests for an already-implemented change. Use after spex-implement, or whenever the user wants test coverage added for existing code, from a plan, a diff, or an ad-hoc request.
---

# Process

**You MUST follow each process step without skipping.**

This skill only writes tests and finalizes spec completion — it never edits plan files, a spec's acceptance criteria, or non-test code; implementation is `spex-implement`'s job.

## Step 1 — Gather context

**Do not write any test files until this step is done; do not skip it**.

### With a plan

Read the plan. Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the plan's `Specs` to read the governing specs.

### Without a plan (ad-hoc request, or testing an existing diff)

1. Determine packages we will work on (e.g. frontend, backend).
2. Run `spex rules --package <package-a> <package-b> --ai --output <tmp-path>` for the affected packages from step 1, then read `<tmp-path>` for the governing rules.
3. Determine all governing spec IDs related to the change — from the request, or from `@spec` annotations already in the changed code.
4. Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the spec IDs from step 3 to read the governing specs.
5. Produce a note in response with a list of governing rules and specs.

### Both paths

Read all specs from step 1 and the governing rules from `<tmp-path>`. Inspect the actual implementation under test — don't test from the plan or spec text alone. Don't skip anything, this is not negotiable!

## Step 2 — Add tests

1. For each governing spec, run `spex coverage` and add tests for every acceptance criterion that has an `impl` annotation but is missing its `test` annotation.
2. Add any other tests the implementation needs based on package testing patterns.
3. Annotate new tests with their spec ID and the specific acceptance criterion they prove, `@spec test <spec-id>#<ac-id>`, where the governing rules require test-layer traceability. Place each annotation on the smallest unit that proves the criterion — the specific test case (`it`/`test` block), not the file or the suite's top-level `describe`. Keep the total number of annotations minimal.

Split into tasks only when work spans independent packages or clearly separable files; most requests are a single task. Within a unit, run every task in parallel, one AI agent per task.

If a task fails or an agent cannot complete it, stop, report what failed, and ask the user how to proceed rather than pressing on with dependent work.

It's important to implement tests against specification, if you see that test fails to meet spec acceptance criteria - fix the implementation if change is small or report to user if problem is bigger, don't try to fit a test to implementation. Testing is layer that ensures spec acceptance criteria are strictly followed.

## Step 3 — Finalize the specs

For every governing spec, run `spex coverage` and open `<spex-dir>/specs/<spec-id>.yaml`:

- For each acceptance criterion `spex coverage` now shows fully impl+test traced, clear its `is_done: false` (remove the line — absent counts as done).
- Once every criterion in the spec is done (`is_done` `true` or absent), set `status: active`.

Touch only `is_done` lines and `status` — leave the rest of each spec, including acceptance criteria text and ids, untouched.

## Step 4 — Verify

Run formatters, linters, and the test suite to confirm the new tests pass and nothing else broke.

Prove the new tests respect the governing rules:

```sh
spex check --package <package-a> <package-b>
```

## Step 5 — Report

Summarize the run:

- Which units of work completed.
- Which specs and acceptance criteria you added coverage for.
- The format, lints, tests, and `spex check` run outcome.
- Which specs you set to `active`, and which acceptance criteria still show `is_done: false` because `spex coverage` doesn't yet show them traced.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
