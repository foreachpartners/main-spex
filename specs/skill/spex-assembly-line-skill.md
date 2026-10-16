# `spex-assembly-line` skill

Implements a plan or an ad-hoc request, loops review and fix until clean or a 3-pass cap, then adds tests. Loaded when the user wants a change implemented while it is automatically reviewed, fixed in a loop until it passes, and tested.

The roles are the Orchestrator (the agent running this skill, which never edits code), the Implement agent (`spex-implement`), the Rule agent (`spex-review-rule`), the Spec agent (`spex-review-spec`), the Fix agent, and the Test agent (`spex-test`).

## Acceptance criteria

- MUST run two independent review/fix loops one after the other — first Rules, then Specs — each repeating from a clean review until a pass finds nothing or 3 passes are reached, running them sequentially because a Spec fix can introduce a rule violation and vice versa.
- MUST NOT let the Orchestrator edit code itself.
- MUST spawn one Implement agent to run `spex-implement` to completion, stopping and reporting an unresolved failure without moving on to review.
- MUST review in each pass by running the `spex-review-rule` (then `spex-review-spec`) process as Orchestrator, and spawn one Fix agent with the findings and the rules (then the spec contents) to fix every finding and re-run formatters, linters, and tests.
- MUST stop a loop when a review reports zero findings, or after pass 3 with findings still open, noting residual findings and moving on.
- MUST spawn one Test agent to run `spex-test` against the reviewed change, last, after both loops settle, stopping and going to Report on an unresolved failure.
- MUST report what was implemented and tested, each loop's passes and findings, whether each axis finished clean or hit the cap, and whether testing finished cleanly.
