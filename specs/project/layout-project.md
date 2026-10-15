# Layout

A project uses one of three layouts, chosen at `spex init` and recorded as `layout` in `spex.yaml`. [glossary.md](../../docs/glossary.md) defines the layouts, the project, its packages, and their naming.

## Acceptance criteria

- MUST use exactly one layout per project: `multi-repo`, `single-repo`, or `monorepo`, chosen at `spex init` and recorded as `layout` in `spex.yaml`.
- MUST make every package its own git repository under the project root folder in `multi-repo`.
- MUST make the spex directory one of the packages in every layout, listed in `packages`: in `multi-repo` it is the spex repository (`<project-name>-spex`), serving as both the spex directory and the agent root.
- MUST provide an editor workspace file (`<project-name>.code-workspace`) in the spex repository in `multi-repo`, opening every package together.
- MUST make the project one git repository in `single-repo`, whose root is the agent root and whose spex directory is its `spex/` folder.
- MUST list in `packages` the single package in `single-repo`, named after the project, at `path: ..`.
- MUST make one git repository hold several packages in `monorepo`, whose root is the agent root and whose spex directory is its `spex/` folder.
- MUST list in `packages` each package in `monorepo`, at its path inside the repository.
- MUST hold in the spex directory `spex.yaml`, the specs, rules (with their scripts), schemas, docs, plans, and the contracts shared across packages.
- MUST hold in the agent root `AGENTS.md`, `CLAUDE.md` (a symlink to `AGENTS.md`), and the skills in `.agents/skills/`.
- MUST link `.claude/skills` to `.agents/skills` so different agent tools detect the same skills: `.claude` is a real directory that keeps its own files (such as its settings), and only `.claude/skills` is a symlink to `../.agents/skills`.
- MUST resolve every location from `spex.yaml` once it is found, never from folder names: the spex directory is the directory holding it, and the agent root is its `agent_root`.
- MUST follow the [values in prd.md](../../docs/prd.md#values): tool-agnostic (specs, contracts, and checks are plain files and deterministic commands) and single-source-of-truth (every load-bearing fact one authoritative source, referenced rather than copied).
