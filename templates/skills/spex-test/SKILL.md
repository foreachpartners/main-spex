---
name: spex-test
description: Add or update tests for an already-implemented change. Use after spex-implement, or whenever the user wants test coverage added for existing code, from a plan, a diff, or an ad-hoc request.
---

# Process

**You MUST follow each process step without skipping.**

This skill only writes tests and finalizes spec completion — it never edits plan files, a spec's acceptance criteria, or non-test code; implementation is `spex-implement`'s job.

## Step 1 — Gather context

**Do not write any test files until this step is done; do not skip it**.

Use the implementation's task context while its governing specs, rules and contracts stay current. Otherwise take the IDs and packages from the plan, request or traces in the diff and run `spex context --id <spec-id-a> <spec-id-b> --package <package-a> <package-b>`. For a change with no governing spec, read the applicable package rules with `spex rules --package <package>`.

Read the spec criteria and applicable rules from the bundle. Inspect the actual implementation and open its file references for details needed by the test. Tell the user which specs and rules govern the work.

## Step 2 — Add tests

1. Inspect `spex coverage --id <spec-id-a> <spec-id-b>` for the governing specs. Add tests for affected criteria that have an `impl` trace and are missing the required `test` trace.
2. Add any other tests the implementation needs based on package testing patterns.
3. Annotate new tests with their spec ID and the specific acceptance criterion they prove, `@spec test <spec-id>#<ac-id>`, where the governing rules require test-layer traceability. Place each annotation on the smallest unit that proves the criterion — the specific test case (`it`/`test` block), not the file or the suite's top-level `describe`. Keep the total number of annotations minimal.

Run a single task directly. Use parallel agents for independent tasks when the runtime supports them and the user permits it. Otherwise keep the work in one session.

If a task fails or an agent cannot complete it, stop, report what failed, and ask the user how to proceed rather than pressing on with dependent work.

It's important to implement tests against specification, if you see that test fails to meet spec acceptance criteria - fix the implementation if change is small or report to user if problem is bigger, don't try to fit a test to implementation. Testing is layer that ensures spec acceptance criteria are strictly followed.

## Step 3 — Execute tests

Run formatters and linters, then execute the relevant test suite. For a routine change confined to one existing acceptance criterion in one package, run targeted tests unless the package rules require a broader suite.

Keep failed or unavailable outcomes explicit. Confirm any required contract validation or manual acceptance before marking the affected spec complete.

## Step 4 — Finalize and check the specs

After the relevant tests pass, run `spex coverage --id <spec-id-a> <spec-id-b>` and open each governing spec at its path from the task context:

- Clear `is_done: false` for criteria with the required traces and passing verification. Remove the line because absent counts as done.
- Once every criterion in the spec is done (`is_done` `true` or absent), set `status: active`.
- For each spec you set to `active`, search `<spex-dir>/specs/` for other specs listing its id in `blocked_by`, and remove that entry, dropping the `blocked_by` key when it becomes empty. A satisfied blocker must not stay listed.

Touch only `is_done` lines and `status` in the governing specs, and only the `blocked_by` entry in the dependent specs — leave the rest of each spec, including acceptance criteria text and ids, untouched.

Check the final rules and spec state:

```sh
spex check --package <package-a> <package-b>
```

## Step 5 — Report

Summarize the run:

- Which units of work completed.
- Which specs and acceptance criteria you added coverage for.
- The rule checks, trace coverage, contract validation, tests executed and manual acceptance outcomes. Give each outcome its own result so a trace can be distinguished from an executed test.
- Which specs you set to `active`, which dependent specs had a `blocked_by` entry removed, and which acceptance criteria still show `is_done: false` because `spex coverage` doesn't yet show them traced.

When a receipt journal has been requested, append a structured record with `spex receipt --journal <directory> append <file>`. Include the context digest, exact revisions when available, evidence references, and any linked decisions or unresolved issues.

## Next

Once the user has committed the change, offer to open a pull request with `spex-create-pr`.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
