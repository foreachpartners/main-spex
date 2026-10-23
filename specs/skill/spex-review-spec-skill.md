# `spex-review-spec` skill

Spec-axis review of the current branch against a spec: is what is implemented of the spec's acceptance criteria correct? Loaded when the user wants to review the implementation of a spec.

## Acceptance criteria

- MUST use the spec the user provided.
- MUST resolve the spec, when not provided, from the feature branch diff against its base branch and the uncommitted changes: the specs changed in them and the specs named by `@spec` annotations in the changed code.
- MUST ask the user when no spec resolves.
- MUST fetch the spec or specs with `spex specs --ai --id <spec-id-a> <spec-id-b>`.
- MUST read the PRD in full when `spex info` prints `prd`, for the product's goal behind the spec.
- MUST review the implementation as it stands on the current branch, not the changes since a starting point.
- MUST find the implementation by its `@spec impl <spec-id>` annotations in the packages listed in the spec's `packages`.
- MUST report, against the implementation, its contract, and the spec contents, the acceptance criteria whose implementation looks wrong.
- MUST quote the acceptance criterion for each finding.
- MUST list, as a note and not as a finding, the acceptance criteria not yet implemented or implemented partially, as long as what is implemented is correct.
- MUST NOT report missing tests, since testing comes in later steps as the last verification layer.
- MUST end with a one-line summary: total findings, and the worst issue (if any).
