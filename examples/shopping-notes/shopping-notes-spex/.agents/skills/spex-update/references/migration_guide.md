# Migration guide

Steps required to move a project from one spex CLI version to a newer one, one section per version that needs them. A version not listed here needs no migration steps.

## 0.20.0

"Workspace" is no longer a spex term: the whole is the project, and its top-level folder is the project root (formerly the workspace root). A script check's `run: workspace` is now `run: project`, and `schemas/check.schema.json` lists `project` instead of `workspace`. Until every rule is renamed, `spex check` fails on `run: workspace`. In `<spex-dir>`:

- In every rule under `rules/**/*.yaml`, replace `run: workspace` with `run: project`. Rules with `sync_with_upstream: true` and their scripts are rewritten by `spex update` and need no change.
- `spex check` now reports the findings of a `run: project` script under a `project` heading instead of `workspace`. In every CI job or script that reads the `spex check` output for that heading, look for `project`.
- Wherever your own text (`AGENTS.md` below the managed section, specs, rules, skills, docs) says "workspace root" or "workspace" for the project, say "project root" or "project". The editor's `<project-name>.code-workspace` file keeps its name.

## 0.19.0

`spex.yaml` gained two required fields: `layout`, how the project is arranged on disk (`multi-repo`, `single-repo`, or `monorepo`), and `agent_root`, the directory holding `AGENTS.md`, `CLAUDE.md` and `.agents/skills/`, relative to the spex directory. Every existing project is a `multi-repo` project, whose agent files live in the spex repository itself, so add `layout: multi-repo` and `agent_root: .` to `<project-root>/<project-name>-spex/spex.yaml`, in that order, right after `cli_version`. Until they are added, every command except `spex update` fails to load `spex.yaml`.

A workspace repository is now called a package: one word for a separate git repository (multi-repo), a package folder (monorepo) and the single repository (single-repo). Rename the following. Until `spex.yaml` is renamed, every command except `spex update` fails to load it. In `<project-root>/<project-name>-spex/`:

- In `spex.yaml`, rename the `repositories` key to `packages`, and in each of its entries `repository_name` to `package_name` and `repository_short_name` to `package_short_name`, keeping each field where it is.
- In every package that has a `spex.repo.yaml`, rename it to `spex.package.yaml`.
- Delete `schemas/spex.repo.schema.json`: `spex update` writes `schemas/spex.package.schema.json` and doesn't delete files.
- In every rule under `rules/**/*.yaml`, replace `run: per-repo` with `run: per-package`. In every script, skill, or doc of the project that runs `spex info --repo` or `spex check --repo`, replace `--repo` with `--package`. Rules with `sync_with_upstream: true` and their scripts are rewritten by `spex update` and need no change.
- In every `ignored_errors` entry of `spex.yaml` or a `spex.package.yaml` that names the `naming-convention` rule's check `repo-dir`, rename the check to `package-dir`.
- Delete the files `spex update` leaves behind after their renames: `rules/naming-convention/repo-dir.sh` (now `package-dir.sh`), and `.agents/skills/spex-setup/references/GENERIC-REPO-SKILL-TEMPLATE.md` (now `GENERIC-PACKAGE-SKILL-TEMPLATE.md`) under the agent root.
