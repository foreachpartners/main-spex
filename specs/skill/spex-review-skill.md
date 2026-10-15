# `spex-review` skill

Reviews the changes since a starting point along both axes at once, governing rules and specs, by running the [`spex-review-rule`](spex-review-rule-skill.md) and [`spex-review-spec`](spex-review-spec-skill.md) skills as parallel sub-agents, then aggregating their findings. Loaded when the user wants to review a branch, a PR, or work-in-progress changes.

A change can pass one axis and fail the other: code that follows every rule but implements the wrong thing is a Rule pass and a Spec fail; code that does exactly what the issue asked but breaks the project's conventions is a Spec pass and a Rule fail. Reporting the axes separately stops one from masking the other.

## Acceptance criteria

- MUST run the `spex-review-rule` and `spex-review-spec` skills as parallel sub-agents against the review target, so they do not pollute each other's context.
- MUST resolve the spec, as the [`spex-review-spec`](spex-review-spec-skill.md) skill does, in the Orchestrator before spawning the sub-agents, and ask the user when no spec resolves.
- MUST resolve the starting point in the Orchestrator before spawning the sub-agents: the one the user gave (a commit, branch, tag, or PR), otherwise `HEAD` for uncommitted changes or the merge-base with the base branch for a feature branch.
- MUST ask the user when the starting point is unclear.
- MUST spawn both sub-agents in a single message with two `Agent` tool calls, using the `general-purpose` subagent for both.
- MUST include in the Rule sub-agent prompt the resolved starting point, the packages to take rules from (the ones the user provided, otherwise those listed in the spec's `packages`), and the instruction to invoke the `spex-review-rule` skill via the Skill tool against the changes since it and report its findings back verbatim.
- MUST include in the Spec sub-agent prompt the spec ID(s) to review, and the instruction to invoke the `spex-review-spec` skill via the Skill tool for this spec and report its findings back verbatim.
- MUST NOT pass the starting point to the Spec sub-agent, since it reviews the spec's current implementation.
- MUST skip the spec sub-agent when the spec is missing, and note this in the final report.
- MUST present the two reports under `## Rules` and `## Specs` headings, verbatim or lightly cleaned.
- MUST end with a one-line summary: total findings per axis, and the worst issue within each axis (if any).
- MUST offer to add test coverage for the change with `spex-test` once both axes are clean, or the user has resolved what they want to resolve.
- MUST NOT merge or rerank findings across the two axes.
- MUST NOT pick a single winner across axes.
