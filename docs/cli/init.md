# `spex init`

Initializes a spex project in one of the three [layouts](../specification.md#layout): `multi-repo`, `single-repo`, or `monorepo`. Run it from the workspace root — the folder holding the packages in multi-repo, the repository root otherwise — or from inside an empty spex directory (`<project-name>-spex` in multi-repo, `spex` otherwise), which is reused instead of created.

It sets up a fresh project only: it fails when the spex directory is not empty, so it never overwrites it. Use `spex update` for an initialized project.

It prompts for the layout, as a numbered list answered with the number or the name and defaulting to `multi-repo`, and to confirm the project name (defaulting to the workspace root folder name), then writes:

- the spex directory, created in the workspace root: `<project-name>-spex` in multi-repo, `spex` otherwise. It holds:
  - `spex.yaml`, seeded from the bundled template with the project name filled in, the installed CLI's version as `cli_version`, the chosen `layout`, and `agent_root` (`.` in multi-repo, `..` otherwise). It lists the rules that apply to each package it lists:
    - multi-repo: only the spex repository (`<project-name>-spex`, at `path: .`)
    - single-repo: only the single package, named after the project (its `package_name` and `package_short_name`), at `path: ..`
    - monorepo: no package, `/spex-setup` adds them
  - the bundled templates: schemas, docs, and every bundled rule with its scripts, plus empty `specs/`, `plans/`, and `contracts/` folders
  - in multi-repo only, `<project-name>.code-workspace`, seeded from the bundled template with a folder for the spex repository only
- in the agent root, which may already hold agent files of its own:
  - `.agents/skills/` with the bundled skills, and a `.claude` -> `.agents` symlink, so different agent tools detect the same skills. When a `.claude` directory already exists, the skills in its `skills/` move into `.agents/skills/` and `.claude/skills` becomes a symlink to `../.agents/skills`
  - an `AGENTS.md` and a `CLAUDE.md` -> `AGENTS.md` symlink. A `# Project` marker splits `AGENTS.md`: content above it is managed and kept in sync by `spex update`, content from the marker onward is the user's own and is never touched. An existing `AGENTS.md` keeps its content as the user's own section, below the marker. An existing regular `CLAUDE.md` is handled the same way and then replaced by the symlink

It fails before writing anything when a skill to move would overwrite a skill of the same name in `.agents/skills/` or a bundled one, or when both `AGENTS.md` and `CLAUDE.md` exist as regular files, since it cannot tell which one to keep.

It does not look for, ask about, or configure the other packages.

```sh
spex init
```

Finish setup afterwards by running the `/spex-setup` skill: it completes `spex.yaml` — the other packages in multi-repo, the packages in monorepo — and creates a skill for every listed package, including the spex repository in multi-repo. In multi-repo it also adds the packages to the workspace file.
