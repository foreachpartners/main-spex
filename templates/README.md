# Templates

Assets scaffolded into client projects by `spex init` and `spex update`. Everything in this folder reaches clients, so MUST NOT place repository-internal notes, drafts, or examples here — add a file only when clients need it.

## Scaffolding rules

- `schemas/` — JSON Schemas for `spex.yaml`, specs, rules, and plans. Overwritten on every run.
- `docs/` — framework reference docs. Overwritten on every run. Each file is a symlink into the repository's own `docs/` folder, which stays their single source of truth; this folder lists exactly which docs ship.
- `AGENTS.md` — seeds the client's root `AGENTS.md` and its `CLAUDE.md` symlink on `init`. `update` rewrites only the managed section above the `# Project` marker, keeping the client's own content.
- `spex.yaml` — seeds the client's `spex.yaml` on `init`, replacing the `unknown` project and spex repository names with the project name. It lists only the spex repository; `spex-setup` adds the others. `init` adds the installed CLI's `cli_version` (the template carries none), and `update` only refreshes it.
- `project-name.code-workspace` — seeds the client's `<project-name>.code-workspace` on `init`, with a folder for the spex repository only; `spex-setup` adds the others. `update` never touches it.
- `rules/` — a rule file and its script folder (same name, beside it) are written when the rule is missing locally or carries `sync_with_upstream: true`.
- `skills/` — written unless the local skill sets `metadata.sync-with-upstream: false`. `update` rewrites each `SKILL.md` only above its `# Project` marker, keeping the client's own content, so every bundled `SKILL.md` MUST end with that marker.

## Editing docs

MUST edit a shipped doc in the repository's own `docs/` folder — the symlink resolves there, so both stay in sync. MUST NOT replace a symlink with a regular file, which forks the doc into two copies.

This `README.md` is the one file that never scaffolds: it documents this folder for spex contributors.
