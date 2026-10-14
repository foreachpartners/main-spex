# `spex-update` skill

Updates a spex project's CLI version and applies the migration steps that upgrade requires. Loaded when the user wants to update spex, upgrade to a new spex version, or run `spex update`.

## Acceptance criteria

### Process

- MUST follow each process step without skipping.

### Step 1: verify a clean baseline

- MUST run `spex check` and `spex coverage`, and, if either reports a failure, stop and ask the user whether to fix the existing issues first or update anyway.
- MUST go on to Step 2 when a failure to load `spex.yaml` is due to a field the installed CLI requires and `spex.yaml` lacks, which is expected after upgrading the binary: Step 3 still runs and Step 4 adds the field.

### Step 2: record the current version

- MUST read `cli_version` from `<spex-dir>/spex.yaml` as the previous version, and record it before Step 3 overwrites it.

### Step 3: update

- MUST run [`spex update`](../cli/spex-update-cli-cmd.md), which bumps `cli_version` in `spex.yaml` and rewrites every bundled schema, doc, rule (with its scripts) and skill, including this skill's `references/migration_guide.md`, which now holds the latest steps.
- MUST read `cli_version` from `spex.yaml` again as the current version.

### Step 4: migrate

- MUST re-read `references/migration_guide.md`, which Step 3 refreshed.
- MUST apply, in order, every version section strictly greater than the previous version and up to and including the current version.
- MUST treat a version missing from the guide as needing no migration, and a listed step that does not apply to the project (e.g. it names a feature the project does not use) as needing no action.

### Step 5: verify

- MUST run `spex check` and `spex coverage` again, and fix any issue they report before finishing.
