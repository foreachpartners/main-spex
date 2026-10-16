# `spex-review-rule` skill

Rule-axis review of the changes since a starting point against the governing cross-cutting conventions and rules. Loaded when the user wants to check a diff, branch, or PR against the package's rules, or asks to "review rules since X".

## Acceptance criteria

- MUST resolve the starting point — the one the user gave, otherwise `HEAD` for uncommitted changes or the merge-base with the base branch — and ask the user when it is unclear.
- MUST use the packages the user provided, otherwise the packages affected by the changes.
- MUST read the governing rules with `spex rules --package ... --ai`, and run `spex check --package ...`, treating every reported violation as a hard finding.
- MUST carry, on top of whatever a package documents, a fixed baseline of Fowler code smells, letting a documented package standard win and treating each smell as a labelled judgement call.
- MUST skip anything tooling already enforces.
- MUST make enforcing the coding rules and ensuring simplicity the primary objective.
- MUST report every `spex check` violation, every documented-rule breach (citing the rule), and any baseline smell spotted (naming it and quoting the hunk), distinguishing hard violations from judgement calls.
- MUST end with a one-line summary: total findings and the worst issue.
