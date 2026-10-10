---
name: spex-review-spec
description: Review the changes since a fixed point (commit, branch, tag, or merge-base) against a spec — does the code match what the spec asked for? Use when the user wants to check a diff, branch, or PR against a spec, or asks to "review spec since X".
---

Spec-axis review of a diff — committed changes since a fixed point, or uncommitted working-tree changes — against a spec.

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

A bad ref or empty diff should fail here rather than partway through the review.

### 2. Fetch the spec

Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the provided spec(s).

### 3. Review

Against the diff and spec contents, report:

- Requirements the spec asked for that are missing or partial.
- Behaviour in the diff that wasn't asked for (scope creep).
- Requirements that look implemented but where the implementation looks wrong.

Quote the spec line for each finding.

End with a one-line summary: total findings, and the worst issue (if any).

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
