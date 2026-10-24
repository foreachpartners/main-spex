# `spex update`

Syncs the project's spex files with the latest versions shipped in the installed `spex` binary: the bundled templates, schemas, docs, rules, skills and `AGENTS.md`. The [`spex-update` skill](../../templates/skills/spex-update/SKILL.md) drives it together with the migration steps an upgrade requires, which it reads with `--migration-guide` and applies before the update.

## Acceptance criteria

- MUST fail when no `spex.yaml` is found: it updates an initialized project only, so `spex init` runs first.
- MUST rewrite a bundled skill already present locally only when its `metadata.sync-with-upstream` is not explicitly `false`; a skill that opts out keeps its local changes, so consumers can override a global skill.
- MUST rewrite a synced skill's `SKILL.md` above its `# Project` marker and the skill's other bundled files, preserving everything from that marker onward and any files added to the skill folder. `# Project` is mark telling that above is managed by spex section, below user customized one.
- MUST rewrite a bundled rule, and its scripts, already present locally only when it's set `sync_with_upstream: true` or absent which defaults to true; a local rule with `sync_with_upstream: false` is left untouched, so consumers can override a global rule.
- MUST deploy a bundled rule missing locally, unless its id is listed in `rules_unused` in `spex.yaml`; listing an id there records a deliberate deletion, so it stays deleted on future updates. We keep `rules_unused` so that users can see all available rules for future use.
- MUST rewrite the managed section of the agent root's `AGENTS.md`, everything above the `# Project` marker, preserving everything from that marker onward.
- MUST overwrite managed files only.
- MUST update `cli_version` in `spex.yaml` to the installed binary's version.
- MUST print the `CHANGELOG.md` entries newer than the previous `cli_version`, newest first; print nothing if there are none, and every entry if the previous `cli_version` is not found in the changelog.
- MUST NOT touch skills, schemas, docs, or rules added by the user (and the scripts beside them), or plans.
- MUST, given `--migration-guide <from_version>`, print instead of updating the sections of the installed binary's bundled migration guide for every version strictly greater than `<from_version>`, newest first; print "no migration needed" if there are none.
- MUST NOT read `spex.yaml` or change any file when given `--migration-guide`, so the steps can be read before the update, even by a project the installed CLI can no longer load.
- MUST fail when the `<from_version>` of `--migration-guide` is not `MAJOR` or `MAJOR.MINOR` or `MAJOR.MINOR.PATCH`.
- SHOULD run after installing latest `spex` binary, to pull in framework updates; `check`, `coverage`, `info`, `rules`, and `specs` warn when `cli_version` does not match the installed CLI.

## Examples

```sh
spex update
spex update --migration-guide 0.19.0
```
