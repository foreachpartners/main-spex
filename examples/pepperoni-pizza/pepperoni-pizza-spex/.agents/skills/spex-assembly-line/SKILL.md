---
name: spex-assembly-line
description: Implement a plan or ad-hoc request, then loop review and fix until clean or a 3-pass cap. Use when the user wants a change or implement something while automatically reviewed and fixed in loop until it passes.
---

- **Orchestrator** — you, running this skill. Drives the loop below and never edits code itself.
- **Implement agent** — a subagent that runs `spex-implement` to produce the change.
- **Rule agent** / **Spec agent** — the two parallel sub-agents `spex-review` spawns to inspect the change.
- **Fix agent** — a subagent that resolves what the Rule and Spec agents found.

Review and Fix repeat, each pass starting from a clean review, until a pass finds nothing or **3 passes** are reached.

## Process

### 1. Implement

Spawn one **Implement agent** (`Agent` tool, subagent `general-purpose`) with the plan or request the user gave you, instructing it to run the `spex-implement` skill's process to completion. Wait for it to finish before moving on.

If the Implement agent reports a failure it couldn't resolve, stop here and report it — don't move on to Review on a broken implementation.

### 2. Review + Fix, up to 3 passes

For pass 1 through 3:

1. **Review** — run the `spex-review` skill's process yourself (as Orchestrator) against the uncommitted diff the Implement agent produced. This spawns the **Rule agent** and **Spec agent** in parallel per that skill's process — do not wrap this step in another subagent yourself, or you'd be nesting subagents for no benefit.
2. If both the Rule agent and Spec agent report zero findings, the change is clean — stop the loop and go to Report.
3. Otherwise, spawn one **Fix agent** (`Agent` tool, subagent `general-purpose`): give it the full aggregated findings from step 1, the rules and specs `spex-review` read, and instruct it to fix every finding, re-running the project's formatters, linters, and tests afterward. Wait for it to finish.
4. If this was pass 3 and findings remain unresolved (the Fix agent couldn't fix everything), stop the loop and go to Report with the residual findings called out. Otherwise continue to the next pass.

## Report

Summarize the run:

- What the Implement agent implemented.
- Each pass's findings from the Rule and Spec agents, and what the matching Fix agent fixed.
- Whether the change finished clean, or hit the 3-pass cap with findings still open.
