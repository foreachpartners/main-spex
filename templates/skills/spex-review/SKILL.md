---
name: spex-review
description: Review the changes since a fixed point (commit, branch, tag, or merge-base) along both axes at once — governing rules (does the code follow cross-cutting conventions and coding standards?) and specs (does the code match what the spec asked for?). Uses spex-review-rule and spex-review-spec skills. Use when the user wants to review a branch, a PR, or work-in-progress changes.
---

Runs the `spex-review-rule` and `spex-review-spec` skills as **parallel sub-agents** against review target, so they don't pollute each other's context, then aggregates their findings.

The spec should have been provided to you, ask user if not.

## Process

### 1. Pin the fixed point

If the user specified a fixed point — a commit SHA, branch name, tag, `main`, `HEAD~5`, etc. — use the committed diff. Otherwise, review **uncommitted changes** (staged and unstaged) against `HEAD`. Review repository spec is scoped to.

**Committed diff** (user gave a fixed point):

- Capture: `git diff <fixed-point>...HEAD` (three-dot, merge-base diff).
- Commit list: `git log <fixed-point>..HEAD --oneline`.
- Confirm the ref resolves: `git rev-parse <fixed-point>`.

**Uncommitted diff** (no fixed point):

- Capture: `git diff HEAD` (staged + unstaged vs `HEAD`).
- Commit list: "working tree (uncommitted)".
- If the working tree is clean, fail — nothing to review.

A bad ref or empty diff should fail here — not inside two parallel sub-agents. Pin the exact `git diff`/`git log` commands (with resolved refs) so both sub-agents review the identical diff.

### 2. Spawn both sub-agents in parallel

Send a single message with two `Agent` tool calls. Use the `general-purpose` subagent for both.

**Rule sub-agent prompt** — include:

- The exact `git diff`/`git log` commands pinned in step 1.
- The instruction: "Invoke the `spex-review-rule` skill via the Skill tool against this diff. Report its findings back verbatim."

**Spec sub-agent prompt** — include:

- The exact `git diff`/`git log` commands pinned in step 1.
- The spec ID(s) to review against.
- The instruction: "Invoke the `spex-review-spec` skill via the Skill tool against this diff and spec. Report its findings back verbatim."

If the spec is missing, skip the spec sub-agent and note this in the final report.

### 3. Aggregate

Present the two reports under `## Rules` and `## Specs` headings, verbatim or lightly cleaned. Do **not** merge or rerank findings — the two axes are deliberately separate (see _Why two axes_).

End with a one-line summary: total findings per axis, and the worst issue _within each axis_ (if any). Don't pick a single winner across axes — that's the reranking the separation exists to prevent.

### 4. Point to the next skill

Once both axes are clean (or the user has resolved what they want to resolve), offer to add test coverage for the change with `spex-test`.

## Why two axes

A change can pass one axis and fail the other:

- Code that follows every rule but implements the wrong thing -> **Rule pass, Spec fail.**
- Code that does exactly what the issue asked but breaks the project's conventions -> **Spec pass, Rule fail.**

Reporting them separately stops one axis from masking the other.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
