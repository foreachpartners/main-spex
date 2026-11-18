---
name: spex-assembly-line
description: Implement a plan or ad-hoc request, loop review and fix until clean or a 3-pass cap, then add tests. Use when the user wants a change or implement something while automatically reviewed, fixed in loop until it passes, and tested.
---

## Routine change

Use the short path when the user delegates the workflow choice and the change affects the implementation of one existing acceptance criterion in one package. The current specification, contract and architecture must still apply.

1. Run `spex-implement` for the criterion and retain the task context.
2. Review the resulting diff once against both the rules and the criterion. Use a separate reviewer when available. Report unresolved findings before moving to tests.
3. Run `spex-test` with the same task context and targeted tests, subject to the package's rules.

Report any unresolved finding and the verification completed. Use the review loops below when the user requests them or when the change needs broader coordinated work.

## Review loops

- **Orchestrator** — coordinates steps and passes the original task and bundle to each role.
- **Implement agent** — runs `spex-implement` to produce the change and assemble its task context.
- **Rule review agent** — reviews the change via `spex-review-rule`.
- **Spec review agent** — reviews the change via `spex-review-spec`.
- **Fix agent** — runs `spex-fix` to resolve what a reviewer found.
- **Test agent** — runs `spex-test` to add tests for the change.

Give each reviewer a fresh context when the runtime supports independent agents and the user permits using them. Pass the task context, diff and starting point to each role. In a runtime with one agent, execute the roles in order and report how review was performed.

Run the rule review loop first and the spec review loop second, with up to **3 review passes** per loop. A spec fix can affect the rule review result.

The Fix agent follows `spex-fix`, which stops and asks instead of making a major change. If it stops, end the run and go to Report with its evidence and proposed solution called out, for the user to decide.

## Process

### 1. Implement

Have the **Implement agent** run `spex-implement` with the plan or request. Wait for it to finish and retain the task context.

If the Implement agent reports a failure it couldn't resolve, stop here and report it — don't move on to Review on a broken implementation.

### 2. Rule loop, up to 3 passes

For pass 1 through 3:

1. **Review** — have the **Rule review agent** run `spex-review-rule` over the uncommitted changes from `HEAD`, with the task context. Wait for its findings.
2. If it reports zero findings, record a clean rule verdict and move to the Spec loop.
3. On pass 3, record outstanding findings and move to the Spec loop.
4. Otherwise give the findings to the **Fix agent** and run `spex-fix`. Continue to the next review pass.

### 3. Spec loop, up to 3 passes

Review the intended implementation against the specs in the task context:

1. **Review** — give one **Spec review agent** the plan or request and task context. Instruct it to run `spex-review-spec` for the specs the change implements, change nothing, and report its findings back verbatim. Wait for it to finish.
2. If the Spec review agent reports zero findings, the spec axis is clean. Move to the final rule review.
3. On pass 3, record outstanding findings and move to the final rule review.
4. Otherwise give the findings to the **Fix agent** and run `spex-fix`. Continue to the next review pass.

### 4. Final rule review

If the Spec loop applied a fix after the latest Rule review, run `spex-review-rule` over the resulting diff at the same starting point. Use that result as the rule verdict. If either axis still has unresolved findings, proceed to Report with both outcomes.

### 5. Test

Have the **Test agent** run `spex-test` on the reviewed diff with the task context. Wait until it has completed and report the checks that executed.

If the Test agent reports a failure it couldn't resolve, stop here and go to Report with it called out.

## Report

Summarize the run:

- What the Implement agent implemented and what the Test agent covered.
- Each loop's passes, the review agent's findings per pass, and what the matching Fix agent fixed.
- Each axis's verdict on the final diff, including any findings left at the pass cap.
- Any stop by a Fix agent that ended the run.
- The rule checks, trace coverage, contract validation, executed tests and manual acceptance outcomes that apply.
- Minor problems, and major ones a small change can fix, that any agent reported.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
