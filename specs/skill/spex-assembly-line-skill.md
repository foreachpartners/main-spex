# `spex-assembly-line` skill

Implements a plan or an ad-hoc request, loops review and fix until clean or a 3-pass cap, then adds tests. Loaded when the user wants a change implemented while it is automatically reviewed, fixed in a loop until it passes, and tested.

The roles are:

- **Orchestrator**: the agent running this skill; drives the loops and never edits code itself.
- **Implement agent**: a subagent that runs `spex-implement` to produce the change.
- **Rule agent**: reviews the change via `spex-review-rule`.
- **Spec agent**: reviews the change via `spex-review-spec`.
- **Fix agent**: a subagent that resolves what a Rule or Spec agent found.
- **Test agent**: a subagent that runs `spex-test` to add coverage for the reviewed change.

## Acceptance criteria

### Process

- MUST run two independent review/fix loops one after the other, first Rules, then Specs, each repeating, from a clean review, until a pass finds nothing or 3 passes are reached.
- MUST run the loops sequentially, not in parallel, because a Spec fix can introduce a rule violation (or vice versa); running Rules to closure first gives the Spec loop a stable base to check against.
- MUST NOT let the Orchestrator edit code itself.

### Step 1: implement

- MUST spawn one Implement agent (`Agent` tool, subagent `general-purpose`) with the plan or request the user gave, instructing it to run the `spex-implement` skill's process to completion, and wait for it to finish before moving on.
- MUST stop and report when the Implement agent reports a failure it could not resolve, without moving on to Review on a broken implementation.

### Step 2: Rule loop, up to 3 passes

- MUST review, in each pass, by running the `spex-review-rule` skill's process as Orchestrator against the uncommitted diff, without wrapping it in another subagent.
- MUST stop the loop and move to the Spec loop when a review reports zero findings, because the rule axis is clean.
- MUST otherwise spawn one Fix agent (`Agent` tool, subagent `general-purpose`) with the review's findings and the rules `spex-review-rule` read, instructing it to fix every finding, re-run the project's formatters, linters, and tests afterward, and wait for it to finish.
- MUST stop the loop after pass 3 with findings unresolved, note the residual findings, and move to the Spec loop anyway.

### Step 3: Spec loop, up to 3 passes

- MUST review, in each pass, by running the `spex-review-spec` skill's process as Orchestrator for the spec(s) the change implements.
- MUST stop the loop and move to Test when a review reports zero findings, because the spec axis is clean.
- MUST otherwise spawn one Fix agent with the findings and the spec contents, instructing it to fix every finding and re-run formatters, linters, and tests, and wait for it to finish.
- MUST stop the loop after pass 3 with findings unresolved, note the residual findings, and move to Test anyway.

### Step 4: test

- MUST spawn one Test agent (`Agent` tool, subagent `general-purpose`) with the plan or request the user gave, instructing it to run the `spex-test` skill's process to completion against the reviewed change, and wait for it to finish.
- MUST run testing last, after both review/fix loops settle, so tests target the final shape of the code rather than a version the Spec loop then rewrites.
- MUST stop and go to Report, with the failure called out, when the Test agent reports a failure it could not resolve.

### Report

- MUST summarize what the Implement agent implemented and what the Test agent covered.
- MUST summarize each loop's passes: the Rule or Spec agent's findings per pass, and what the matching Fix agent fixed.
- MUST state whether each axis finished clean, or hit the 3-pass cap with findings still open.
- MUST state whether the Test agent finished cleanly or reported an unresolved failure.
