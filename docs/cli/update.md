# `spex update`

Rewrites the bundled templates in the spex repository — skills, schemas, docs, and rules (including their checks and scripts) — with the versions shipped in the installed `spex` binary.

It updates an initialized project only: it fails when no `spex.yaml` is found, so run `spex init` first.

- Rewrites a bundled skill already present locally only when its `metadata.sync-with-upstream` is not explicitly `false`; a skill that opts out keeps its local changes, so consumers can override a global skill.
- Rewrites a synced skill's `SKILL.md` above its `# Project` marker and the skill's other bundled files, preserving everything from that marker onward and any files you added to the skill folder; a local `SKILL.md` predating the marker is fully overwritten, since managed and user content cannot be told apart in it.
- Rewrites a bundled rule (and its scripts) already present locally only when it has `sync_with_upstream: true`; a local rule without it is left untouched, so consumers can override a global rule.
- Deploys a bundled rule missing locally, unless its id is listed in `rules_unused` in `spex.yaml`; listing an id there records a deliberate deletion, so it stays deleted on future updates.
- Rewrites the managed section of the root `AGENTS.md` (everything above the `# Project` marker), preserving everything from that marker onward; a local `AGENTS.md` predating the marker is fully overwritten, since managed and user content cannot be told apart in it.
- Overwrites managed files only; any skills, schemas, docs, or rules you added (and the scripts beside them), and plans, are left untouched.
- Updates `cli_version` in `spex.yaml` to the installed binary's version.
- Prints the `CHANGELOG.md` entries newer than the previous `cli_version`, newest first; prints nothing if there are none, and every entry if the previous `cli_version` is not found in the changelog.
- Run it after upgrading the `spex` binary to pull in framework updates; `check`, `coverage`, `info`, `rules` and `specs` warn when `cli_version` does not match the installed CLI.

```sh
spex update
```
