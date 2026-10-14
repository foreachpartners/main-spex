# `spex-review` skill

Reviews the changes since a fixed point along both axes at once, governing rules and specs, by running the [`spex-review-rule`](spex-review-rule-skill.md) and [`spex-review-spec`](spex-review-spec-skill.md) skills as parallel sub-agents, then aggregating their findings. Loaded when the user wants to review a branch, a PR, or work-in-progress changes.

A change can pass one axis and fail the other: code that follows every rule but implements the wrong thing is a Rule pass and a Spec fail; code that does exactly what the issue asked but breaks the project's conventions is a Spec pass and a Rule fail. Reporting the axes separately stops one from masking the other.

## Acceptance criteria

- MUST run the `spex-review-rule` and `spex-review-spec` skills as parallel sub-agents against the review target, so they do not pollute each other's context.
- MUST have the spec provided, and ask the user when it is not.
- MUST pin the diff to review as specified in [review-target-skill.md](review-target-skill.md), failing there rather than inside two parallel sub-agents.
- MUST pin the exact `git diff` and `git log` commands, with resolved refs, so both sub-agents review the identical diff.
- MUST spawn both sub-agents in a single message with two `Agent` tool calls, using the `general-purpose` subagent for both.
- MUST include in the Rule sub-agent prompt the exact pinned git commands and the instruction to invoke the `spex-review-rule` skill via the Skill tool against this diff and report its findings back verbatim.
- MUST include in the Spec sub-agent prompt the exact pinned git commands, the spec ID(s) to review against, and the instruction to invoke the `spex-review-spec` skill via the Skill tool against this diff and spec and report its findings back verbatim.
- MUST skip the spec sub-agent when the spec is missing, and note this in the final report.
- MUST present the two reports under `## Rules` and `## Specs` headings, verbatim or lightly cleaned.
- MUST end with a one-line summary: total findings per axis, and the worst issue within each axis (if any).
- MUST offer to add test coverage for the change with `spex-test` once both axes are clean, or the user has resolved what they want to resolve.
- MUST NOT merge or rerank findings across the two axes.
- MUST NOT pick a single winner across axes.
