# Templates

Assets scaffolded into client projects by `spex init` and `spex update`, or included in the `spex` binary to be accessed from it. Everything in this folder is public and part of the spex CLI, so MUST NOT place repository-internal notes, drafts, or examples here — add a file only when clients need it.

## Scaffolding rules

- `schemas/` — JSON Schemas for `spex.yaml`, specs, rules, and plans. Overwritten on every run.
- `AGENTS.md` — seeds the `AGENTS.md` in the client's agent root on `init`. `update` rewrites only the managed section above the `# Project` marker, keeping the client's own content. Its glossary line is replaced by the repository's `docs/glossary.md`, which is never shipped as a file of its own.
- `spex.yaml` — seeds the client's `spex.yaml` on `init`, the same for every layout. `init` replaces every `unknown` with the project name, the layout, the layout's `agent_root` and the spex package name, and adds the installed CLI's `cli_version` (the template carries none). It lists only the spex package; `spex-setup` adds the rest. `update` only refreshes `cli_version`.
- `project-name.code-workspace` — seeds the client's `<project-name>.code-workspace` on `init` in multi-repo only, with a folder for the spex package only; `spex-setup` adds the others. `update` never touches it.
- `rules/` — the rules library: rules for conventions many projects share, each a rule file and its script folder (same name, beside it). Rules may sit in any number of subfolders (`rules/db/`, `rules/db/postgres/`), each with no rule file of its name beside it. Included in the binary and accessed with `spex lib rules`, never scaffolded: a project takes a rule with `spex lib rules --copy`, and `update` then rewrites it, with its scripts, unless it sets `sync_with_upstream: false`. Every bundled rule MUST set `sync_with_upstream: true` right after `title`, which is how the `spex-rule` skill knows a local rule came from upstream.
- `skills/` — written unless the local skill sets `metadata.sync-with-upstream: false`. `update` rewrites each `SKILL.md` only above its `# Project` marker, keeping the client's own content, so every framework `SKILL.md` MUST end with that marker.
- `specs/` — the specs library: template specs the `spex-spec` skill offers as a draft. Included in the binary and accessed with `spex lib specs`, never scaffolded.

This `README.md` is the one file outside `specs/` that never scaffolds: it documents this folder for spex contributors.
