# `spex-update` skill

Updates a spex project's CLI version and applies the migration steps the upgrade requires. Loaded when the user wants to update spex, upgrade to a new spex version, or run `spex update`.

## Acceptance criteria

- MUST follow each process step without skipping.
- MUST run `spex check` and `spex coverage` first and stop to ask the user on any failure, except a failure to load `spex.yaml` from a field the installed CLI requires, which the migration steps add.
- MUST record `cli_version` from `spex.yaml` as the previous version before updating.
- MUST run `spex update --migration-guide <previous version>`, read all its printed steps, and apply them from the oldest version to the newest, the reverse of the printed order, before updating.
- MUST treat `no migration needed` and a printed step that does not apply to the project as needing no action.
- MUST run `spex update` to bump `cli_version` in `spex.yaml` and rewrite every bundled schema, doc, rule (with its scripts), and skill.
- MUST run `spex check` and `spex coverage` again and fix any issue they report.
