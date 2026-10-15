---
name: spex-assembly-line
description: Implement a plan or ad-hoc request, loop review and fix until clean or a 3-pass cap, then add tests. Use when the user wants a change or implement something while automatically reviewed, fixed in loop until it passes, and tested.
---

- **Orchestrator** — you, running this skill. Drives the loops below and never edits code itself.
- **Implement agent** — a subagent that runs `spex-implement` to produce the change.
- **Rule agent** — reviews the change via `spex-review-rule`.
- **Spec agent** — reviews the change via `spex-review-spec`.
- **Fix agent** — a subagent that resolves what a Rule or Spec agent found.
- **Test agent** — a subagent that runs `spex-test` to add coverage for the reviewed change.

Two independent review/fix loops run one after the other — first Rules, then Specs — each repeating, from a clean review, until a pass finds nothing or **3 passes** are reached. They run sequentially, not in parallel, because a Spec fix can introduce a rule violation (or vice versa); running Rules to closure first gives the Spec loop a stable base to check against.

## Process

### 1. Implement

Spawn one **Implement agent** (`Agent` tool, subagent `general-purpose`) with the plan or request the user gave you, instructing it to run the `spex-implement` skill's process to completion. Wait for it to finish before moving on.

If the Implement agent reports a failure it couldn't resolve, stop here and report it — don't move on to Review on a broken implementation.

### 2. Rule loop, up to 3 passes

For pass 1 through 3:

1. **Review** — run the `spex-review-rule` skill's process yourself (as Orchestrator) against the uncommitted diff. Do not wrap this in another subagent — you're already inspecting, not editing.
2. If the Rule agent's review reports zero findings, the rule axis is clean — stop this loop and move to the Spec loop.
3. Otherwise, spawn one **Fix agent** (`Agent` tool, subagent `general-purpose`): give it the findings from step 1, the rules `spex-review-rule` read, and instruct it to fix every finding, re-running the project's formatters, linters, and tests afterward. Wait for it to finish.
4. If this was pass 3 and findings remain unresolved, stop this loop, note the residual findings, and move to the Spec loop anyway. Otherwise continue to the next pass.

### 3. Spec loop, up to 3 passes

Same structure as the Rule loop, using `spex-review-spec` for Review and the spec(s) it read for the Fix agent's brief:

1. **Review** — run the `spex-review-spec` skill's process yourself for the spec(s) the change implements.
2. If the Spec agent's review reports zero findings, the spec axis is clean — stop this loop and move to Test.
3. Otherwise, spawn one **Fix agent** with the findings and spec contents, instructing it to fix every finding and re-run formatters, linters, and tests. Wait for it to finish.
4. If this was pass 3 and findings remain unresolved, stop this loop, note the residual findings, and move to Test anyway. Otherwise continue to the next pass.

### 4. Test

Spawn one **Test agent** (`Agent` tool, subagent `general-purpose`) with the plan or request the user gave you, instructing it to run the `spex-test` skill's process to completion against the reviewed change. Wait for it to finish before moving on.

Testing runs last, against the change after both review/fix loops settle, so tests target the final shape of the code rather than a version the Spec loop then rewrites.

If the Test agent reports a failure it couldn't resolve, stop here and go to Report with it called out.

## Report

Summarize the run:

- What the Implement agent implemented and what the Test agent covered.
- Each loop's passes, the Rule or Spec agent's findings per pass, and what the matching Fix agent fixed.
- Whether each axis finished clean, or hit the 3-pass cap with findings still open.
- Whether the Test agent finished cleanly or reported an unresolved failure.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
