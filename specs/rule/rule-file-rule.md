# Rule file

A rule, defined in [glossary.md](../../docs/glossary.md), lives at `rules/<rule-id>.yaml`, nested subdirectories allowed; the Rule ID is the filename, whatever the nesting.

## Acceptance criteria

- MUST name the file `<rule-id>.yaml`, whatever its nesting under `rules/`.
- MUST follow `schemas/rule-id.schema.json` for the `id`: lowercase letters, digits and hyphens only, at most 40 characters.
- MUST have the required fields `id`, `title`, `description`, and `acceptance_criteria`.
- MUST hold the individual rules in `acceptance_criteria`, ordered `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`, each criterion a string starting with that commitment.
- MUST apply a rule package-wide, not per feature, once a package opts into it by listing it in `rules` in `spex.yaml`.
- MUST keep a bundled rule in sync from the bundled template, with its scripts, on `spex update`, unless it sets `sync_with_upstream: false`; an unset `sync_with_upstream` defaults to `true`.
- MUST keep local changes across updates to a rule with `sync_with_upstream: false`, which customizes it locally.
- MUST recognize a deleted bundled rule as deliberate only once its id is added to `rules_unused` in `spex.yaml`.
- MUST NOT give a rule a lifecycle status, or its acceptance criteria an implementation flag.
- SHOULD write the code in `examples` in Rust, unless the rule targets another language or format.
- MAY have the optional fields `checks`, `examples`, and `sync_with_upstream`; `sync_with_upstream` defaults to `true`, and `false` opts a bundled rule out of `spex update`.
