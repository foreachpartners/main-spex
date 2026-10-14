# `spex-review-spec` skill

Spec-axis review of a diff, committed changes since a fixed point or uncommitted working-tree changes, against a spec: does the code match what the spec asked for? Loaded when the user wants to check a diff, branch, or PR against a spec, or asks to "review spec since X".

## Acceptance criteria

- MUST have the spec provided, and ask the user when it is not.
- MUST pin the diff to review as specified in [review-target-skill.md](review-target-skill.md).
- MUST fetch the spec or specs with `spex specs --ai --id <spec-id-a> <spec-id-b>`.
- MUST report, against the diff and the spec contents, the requirements the spec asked for that are missing or partial.
- MUST report the behaviour in the diff that was not asked for (scope creep).
- MUST report the requirements that look implemented but where the implementation looks wrong.
- MUST quote the spec line for each finding.
- MUST end with a one-line summary: total findings, and the worst issue (if any).
