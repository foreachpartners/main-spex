# `spex update`

Syncs the project's spex files with the latest versions shipped in the installed `spex` binary: the bundled templates, schemas, docs, and rules (including their checks and scripts) in the spex directory, skills and `AGENTS.md` in the agent root. The [`spex-update` skill](../skill/spex-update-skill.md) drives it together with the migration steps an upgrade requires.

## Acceptance criteria

- MUST locate the agent root from `agent_root` in `spex.yaml`, and take the spex directory when it is absent, so a project written before `agent_root` existed can still update before it is migrated.
- MUST fail when no `spex.yaml` is found: it updates an initialized project only, so `spex init` runs first.
- MUST rewrite a bundled skill already present locally only when its `metadata.sync-with-upstream` is not explicitly `false`; a skill that opts out keeps its local changes, so consumers can override a global skill.
- MUST rewrite a synced skill's `SKILL.md` above its `# Project` marker and the skill's other bundled files, preserving everything from that marker onward and any files added to the skill folder ([managed-section-framework.md](../framework/managed-section-framework.md)).
- MUST rewrite a bundled rule, and its scripts, already present locally only when it has `sync_with_upstream: true`; a local rule without it is left untouched, so consumers can override a global rule.
- MUST deploy a bundled rule missing locally, unless its id is listed in `rules_unused` in `spex.yaml`; listing an id there records a deliberate deletion, so it stays deleted on future updates.
- MUST rewrite the managed section of the agent root's `AGENTS.md`, everything above the `# Project` marker, preserving everything from that marker onward.
- MUST re-create a missing `.claude` symlink, or `.claude/skills` symlink when a `.claude` directory exists, the way [`spex init`](spex-init-cli-cmd.md) creates it.
- MUST overwrite managed files only.
- MUST update `cli_version` in `spex.yaml` to the installed binary's version.
- MUST print the `CHANGELOG.md` entries newer than the previous `cli_version`, newest first; print nothing if there are none, and every entry if the previous `cli_version` is not found in the changelog.
- MUST NOT touch skills, schemas, docs, or rules added by the user (and the scripts beside them), or plans.
- SHOULD run after upgrading the `spex` binary, to pull in framework updates; `check`, `coverage`, `info`, `rules`, and `specs` warn when `cli_version` does not match the installed CLI.

## Examples

```sh
spex update
```
