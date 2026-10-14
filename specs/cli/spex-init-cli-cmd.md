# `spex init`

Initializes a spex project in one of the three [layouts](../project/layout-project.md): `multi-repo`, `single-repo`, or `monorepo`. It sets up a fresh project only, and leaves finishing the setup to the [`spex-setup` skill](../skill/spex-setup-skill.md).

## Acceptance criteria

- MUST run from the project root, the folder holding the packages in `multi-repo` and the repository root otherwise, or from inside an empty spex directory (`<project-name>-spex` in `multi-repo`, `spex` otherwise), which is reused instead of created.
- MUST take a folder for the spex directory only when its name fits and it is empty or already holds a `spex.yaml`; any other folder is the project root, so a repository that happens to be named `spex` initializes like any other and its folder name is the default project name.
- MUST fail when the spex directory is not empty, so it sets up a fresh project only and never overwrites; `spex update` is for an initialized project.
- MUST prompt for the layout, as a numbered list answered with the number or the name and defaulting to `multi-repo`.
- MUST prompt to confirm the project name, defaulting to the project root folder name.
- MUST create the spex directory in the project root: `<project-name>-spex` in `multi-repo`, `spex` otherwise.
- MUST write `spex.yaml` in the spex directory, seeded from the bundled template with the project name filled in, the installed CLI's version as `cli_version`, the chosen `layout`, and `agent_root` (`.` in `multi-repo`, `..` otherwise).
- MUST list in `spex.yaml`, in `multi-repo`, only the spex repository (`<project-name>-spex`, at `path: .`) as a package, with the rules that apply to it.
- MUST list in `spex.yaml`, in `single-repo`, only the single package, named after the project (its `package_name` and `package_short_name`), at `path: ..`, with the rules that apply to it.
- MUST list no package in `spex.yaml` in `monorepo`; `/spex-setup` adds them.
- MUST write the bundled templates into the spex directory: schemas, docs, and every bundled rule with its scripts, plus empty `specs/`, `plans/`, and `contracts/` folders.
- MUST write `<project-name>.code-workspace` in `multi-repo` only, seeded from the bundled template with a folder for the spex repository only.
- MUST write, in the agent root, `.agents/skills/` with the bundled skills, and a `.claude` -> `.agents` symlink, so different agent tools detect the same skills.
- MUST, when a `.claude` directory already exists, move the skills in its `skills/` into `.agents/skills/` and make `.claude/skills` a symlink to `../.agents/skills`.
- MUST write, in the agent root, an `AGENTS.md` and a `CLAUDE.md` -> `AGENTS.md` symlink, split by the `# Project` marker ([managed-section-framework.md](../framework/managed-section-framework.md)).
- MUST keep an existing `AGENTS.md` content as the user's own section below the marker.
- MUST handle an existing regular `CLAUDE.md` the same way as `AGENTS.md`, then replace it with the symlink.
- MUST fail before writing anything when a skill to move would overwrite a skill of the same name in `.agents/skills/` or a bundled one.
- MUST fail before writing anything when both `AGENTS.md` and `CLAUDE.md` exist as regular files, since it cannot tell which one to keep.
- MUST NOT look for, ask about, or configure the other packages.
- SHOULD be followed by the `/spex-setup` skill: it completes `spex.yaml` (the other packages in `multi-repo`, the packages in `monorepo`) and creates a skill for every listed package, including the spex repository in `multi-repo`; in `multi-repo` it also adds the packages to the workspace file.

## Examples

```sh
spex init
```
