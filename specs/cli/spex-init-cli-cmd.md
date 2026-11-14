# `spex init`

Initializes the spex directory for a project in one of the three [layouts](../project/layout-project.md): `multi-repo`, `single-repo`, or `monorepo`. It sets up a fresh project only, and leaves finishing the setup to the [`spex-setup` skill](../../templates/skills/spex-setup/SKILL.md).

## Acceptance criteria

- MUST allow to run `spex init` in project root only.
- MUST prompt to confirm the project name, defaulting to the project root folder name.
- MUST prompt for the layout, as a numbered list answered with the number or the name and defaulting to `multi-repo`.
- MUST create the spex directory in the project root: `<project-name>-spex` in `multi-repo`, `spex` otherwise.
- MUST copy bundled `spex.yaml` in spex directory and replace `unknown` for chosen `project_name`, the installed CLI's version as `cli_version`, the chosen `layout`, `agent_root` (`.` in `multi-repo`, `..` otherwise), `package_name` as the spex directory name (`<project-name>-spex` in `multi-repo`, `spex` otherwise), and keep its `prd: docs/prd.md` and `project_short_name: unknown`, which the `spex-setup` skill fills.
- MUST write the bundled templates into the spex directory as is: schemas.
- MUST create empty `docs/`, `rules/`, `specs/`, `plans/`, and `contracts/` folders in spex dir.
- MUST NOT write any rule: the [rules library](../rule/rules-library-rule.md) holds them, and the `spex-setup` skill takes the ones that fit each package.
- MUST write, in the agent root, an `AGENTS.md`.
- MUST write `<project-name>.code-workspace` in `multi-repo` only, seeded from the bundled template with a folder for the spex repository only.
- MUST write, in the agent root, `.agents/skills/` with the framework skills, and a `.claude/skills` -> `../.agents/skills` symlink inside a real `.claude` directory, so different agent tools detect the same skills while the rest of `.claude` (such as its settings) stays out of `.agents`.
- MUST, if a `.claude` directory already exists in `agent_root`, move its `skills/` into `.agents/skills/` before making `.claude/skills` that symlink, leaving the rest of `.claude` in place.
- MUST keep an existing `AGENTS.md` content as the user's own section below the `# Project` marker.
- MUST NOT write, link, or change a `CLAUDE.md`: Claude Code reads `AGENTS.md` itself.
- MUST NOT look for, ask about, or configure the other packages.
- SHOULD be followed by the `/spex-setup` skill: it completes `spex.yaml` setup and configures other packages.

## Examples

### multi-repo

```text
shopping-notes/                  project root
  shopping-notes-api/
  shopping-notes-frontend/
  shopping-notes-spex/           spex directory and agent root
    spex.yaml
    shopping-notes.code-workspace
    AGENTS.md
    .agents/skills/
    .claude/skills -> ../.agents/skills
    docs/  schemas/  rules/  specs/  plans/  contracts/
```

### single-repo

```text
snake-game/                           project root, agent root
  AGENTS.md
  .agents/skills/
  .claude/skills -> ../.agents/skills
  src/
  spex/                          spex directory
    spex.yaml
    docs/  schemas/  rules/  specs/  plans/  contracts/
```

### monorepo

```text
shopping-notes/                  project root, agent root
  AGENTS.md
  .agents/skills/
  .claude/skills -> ../.agents/skills
  apps/api/
  apps/frontend/
  spex/                          spex directory
    spex.yaml
    docs/  schemas/  rules/  specs/  plans/  contracts/
```
