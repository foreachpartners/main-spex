---
name: spex-review
description: Review the changes since a starting point (commit, branch, tag, or PR) along both axes at once — governing rules (does the code follow cross-cutting conventions and coding standards?) and specs (is the implementation of the spec's acceptance criteria correct?). Uses spex-review-rule and spex-review-spec skills. Use when the user wants to review a branch, spec, a PR, or work-in-progress changes.
---

## Process

### 1. Resolve the spec

Use the spec the user provided. Otherwise resolve it from the current work: the feature branch diff against its base branch (`git diff <base-branch>...HEAD`) and the uncommitted changes (`git diff HEAD`). Take the specs changed in them and the specs named by `@spec` annotations in the changed code. Ask the user when no spec resolves.

### 2. Resolve the starting point

Use the starting point the user gave (a commit, branch, tag, or PR), otherwise `HEAD` for uncommitted changes or the merge-base with the base branch for a feature branch, and ask the user when unclear.

### 3. Spawn both sub-agents in parallel

Run the `spex-review-rule` and `spex-review-spec` skills as **parallel sub-agents**, so they don't pollute each other's context. Send a single message with two `Agent` tool calls. Use the `general-purpose` subagent for both.

**Rule sub-agent prompt** — include:

- The starting point resolved in step 2.
- The packages to take rules from: the ones the user provided, otherwise those listed in the spec's `packages`.
- The instruction: "Invoke the `spex-review-rule` skill via the Skill tool against the changes since this starting point. Report its findings back verbatim."

**Spec sub-agent prompt** — include:

- The spec ID(s) to review.
- The instruction: "Invoke the `spex-review-spec` skill via the Skill tool for this spec. Report its findings back verbatim."

If the spec is missing, skip the spec sub-agent and note this in the final report.

### 4. Aggregate

Present the two reports under `## Rules` and `## Specs` headings, verbatim or lightly cleaned. Do **not** merge or rerank findings — the two axes are deliberately separate (see _Why two axes_).

End with a one-line summary: total findings per axis, and the worst issue _within each axis_ (if any). Don't pick a single winner across axes — that's the reranking the separation exists to prevent.

### 5. Point to the next skill

Once both axes are clean (or the user has resolved what they want to resolve), offer to add test coverage for the change with `spex-test`.

## Why two axes

A change can pass one axis and fail the other:

- Code that follows every rule but implements the wrong thing -> **Rule pass, Spec fail.**
- Code that does exactly what the issue asked but breaks the project's conventions -> **Spec pass, Rule fail.**

Reporting them separately stops one axis from masking the other.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
