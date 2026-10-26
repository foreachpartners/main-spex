---
name: spex-fix
description: Fix a reported problem at its root cause — a bug, a review finding, a failing check. Use when the user wants something fixed or a finding resolved.
---

Fix what is really broken, properly, at its root cause — or stop and ask. Work through reported problems one at a time.

## Process

### 1. Confirm the problem is real

Read the code and the governing specs and rules (`spex specs --ai --id <spec-id>`; `spex rules --package <package> --ai --output <tmp-path>`), then reproduce the problem: a failing test or command, a `spex check` finding, or the exact code path that misbehaves.

A report can be wrong — the spec may require the behavior, a rule may allow it, the code may already be correct. When you cannot show the problem, change nothing and report your evidence.

### 2. Find the root cause

Trace the problem to the code that causes it, not the code where it shows up.

### 3. Decide the proper fix

A proper fix removes the cause and follows the specs and rules. A guard, special case, retry, suppression (including a `check_ignore` entry) or skipped test that only hides the symptom is a workaround — apply one only when the user explicitly asks for it in this request.

If the proper fix is major — a refactor, an architectural change, a spec or contract change, or edits beyond the task's scope — stop (see **When to stop**). A fix that changes specified behavior is a spec change first.

### 4. Fix

Apply the proper fix, matching the surrounding code and changing only what the cause requires. Keep `@spec` annotations accurate for every acceptance criterion the fix touches.

### 5. Verify

Reproduce the problem again — it must be gone. Run formatters, linters, the existing tests and `spex check --package <package>`; run `spex coverage` when annotations changed. Fix any failure you caused.

## When to stop

Stop means: change nothing further, report what you found with evidence, and ask the user how to proceed.

- The proper fix is major (step 3): present the proper solution and wait for explicit permission.
- Any step reveals a major problem — a different bug, a spec/code mismatch, a failing check, a design flaw — related to the task or not. Never fix it silently, work around it, or ignore it.

Minor problems wait for the report.

## Report

- The problem, whether it was real (with evidence), and its root cause.
- What changed, and how it removes the cause.
- The outcome of the reproduction, formatters, linters, tests, `spex check` and `spex coverage`.
- Minor problems noticed along the way, left unfixed.

Offer to add regression coverage with `spex-test`.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
