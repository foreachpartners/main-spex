# Migration guide

Steps required to move a project from one spex CLI version to a newer one, one section per version that needs them. A version not listed here needs no migration steps.

`spex update --migration-guide <from_version>` prints the sections after a version, newest first, and the `spex-update` skill reads them all and applies them from the oldest version to the newest before it runs `spex update`: a step must not rely on files `spex update` writes. Keep every version heading as `## MAJOR.MINOR.PATCH`.

## 0.24.0

The baseline of pre-existing errors, `ignored_errors`, is now `check_ignore`, like `coverage_ignore` for `spex coverage`. Each entry keeps its `rule_id`, `check` and `files`. Until it is renamed, `spex.yaml` and every `spex.package.yaml` fail schema validation. In `<spex-dir>`:

- In `spex.yaml`, rename every `ignored_errors` key, in the package entries, to `check_ignore`, keeping each entry and its position.
- In every package that has a `spex.package.yaml`, rename its root `ignored_errors` key to `check_ignore`.
- In every CI job, script, skill, or doc of the project that reads the `ignored_errors` key of `spex info` output, read `check_ignore` instead.
- Wherever your own text (`AGENTS.md` below the managed section, specs, rules, skills, docs) mentions `ignored_errors`, say `check_ignore`.

## 0.23.0

`.claude` is no longer a symlink to `.agents`: it is a real directory that keeps its own files, such as `settings.json`, and only `.claude/skills` is a symlink to `../.agents/skills`. A project made by an earlier version has the whole `.claude` linked to `.agents`, so Claude Code's own files were written into `.agents`. In `<agent-root>`:

- If `.claude` is a symlink to `.agents` (check with `ls -l .claude`), remove the symlink with `rm .claude`, without a trailing slash, then create the `.claude` directory and link `.claude/skills` to `../.agents/skills`.
- List the entries of `.agents` other than `skills`. Move the ones Claude Code wrote, such as `settings.json`, `settings.local.json`, `commands`, `agents` or `hooks`, into `.claude`. Ask the user about any other entry before moving it.
- If `.claude` is already a real directory, change nothing: only its `skills` link is needed, and `spex init` already made it.

## 0.22.0

A bundled rule that sets no `sync_with_upstream` is now synced from upstream: `spex update` overwrites it, and its scripts, with the bundled version. Before, a rule without the field was kept as a local customization. `sync_with_upstream: false` now opts a rule out, and `schemas/rule.schema.json` defaults the field to `true`. `spex update` also needs `agent_root` in `spex.yaml` and fails without it, where it used to assume the spex directory. In `<spex-dir>`:

- For every rule under `rules/**/*.yaml` named like a bundled rule (`commit-message-format`, `dependencies-rules`, `graphql-conventions`, `naming-convention`, `openapi-conventions`, `secret-handling`, `testing-rules`, `writing-style`) that sets no `sync_with_upstream`, ask the user whether it holds local customizations. If it does, add `sync_with_upstream: false` to it. If not, change nothing: it now syncs. Rules with `sync_with_upstream: true` or `false` need no change, and `spex update` never touches a rule that no bundled rule shares its name with.

## 0.21.0

A spec's `scope` field is now `packages`, and `spex coverage` and `spex specs` filter with `--package` instead of `--scope`. `schemas/spec.schema.json` lists `packages` instead of `scope`. Until every spec is renamed, `spex check` fails on `scope`. In `<spex-dir>`:

- In every spec under `specs/**/*.yaml`, rename the `scope` key to `packages`, keeping its list and its position.
- In every CI job, script, skill, or doc of the project that runs `spex coverage --scope` or `spex specs --scope`, replace `--scope` with `--package`. It now accepts a full package name as well as a short name, like `--package` of `spex check`, `spex info` and `spex rules`.
- The "Gaps" section of `spex coverage` now lists a spec that sets no `packages` under `no package` instead of `unscoped`. In every CI job or script that reads the report for that heading, look for `no package`.
- Wherever your own text (`AGENTS.md` below the managed section, specs, rules, skills, docs) calls a spec's packages its `scope`, say `packages`.

## 0.20.0

"Workspace" is no longer a spex term: the whole is the project, and its top-level folder is the project root (formerly the workspace root). A script check's `run: workspace` is now `run: project`, and `schemas/check.schema.json` lists `project` instead of `workspace`. Until every rule is renamed, `spex check` fails on `run: workspace`. In `<spex-dir>`:

- In every rule under `rules/**/*.yaml`, replace `run: workspace` with `run: project`. Rules with `sync_with_upstream: true` and their scripts are rewritten by `spex update` and need no change.
- `spex check` now reports the findings of a `run: project` script under a `project` heading instead of `workspace`. In every CI job or script that reads the `spex check` output for that heading, look for `project`.
- Wherever your own text (`AGENTS.md` below the managed section, specs, rules, skills, docs) says "workspace root" or "workspace" for the project, say "project root" or "project". The editor's `<project-name>.code-workspace` file keeps its name.

## 0.19.0

`spex.yaml` gained two required fields: `layout`, how the project is arranged on disk (`multi-repo`, `single-repo`, or `monorepo`), and `agent_root`, the directory holding `AGENTS.md`, `CLAUDE.md` and `.agents/skills/`, relative to the spex directory. Every existing project is a `multi-repo` project, whose agent files live in the spex repository itself, so add `layout: multi-repo` and `agent_root: .` to `<project-root>/<project-name>-spex/spex.yaml`, in that order, right after `cli_version`. Until they are added, every command, `spex update` included, fails to load `spex.yaml`.

A workspace repository is now called a package: one word for a separate git repository (multi-repo), a package folder (monorepo) and the single repository (single-repo). Rename the following. Until `spex.yaml` is renamed, every command except `spex update` fails to load it. In `<project-root>/<project-name>-spex/`:

- In `spex.yaml`, rename the `repositories` key to `packages`, and in each of its entries `repository_name` to `package_name` and `repository_short_name` to `package_short_name`, keeping each field where it is.
- In every package that has a `spex.repo.yaml`, rename it to `spex.package.yaml`.
- Delete `schemas/spex.repo.schema.json`: `spex update` writes `schemas/spex.package.schema.json` and doesn't delete files.
- In every rule under `rules/**/*.yaml`, replace `run: per-repo` with `run: per-package`. In every script, skill, or doc of the project that runs `spex info --repo` or `spex check --repo`, replace `--repo` with `--package`. Rules with `sync_with_upstream: true` and their scripts are rewritten by `spex update` and need no change.
- In every `ignored_errors` entry of `spex.yaml` or a `spex.package.yaml` that names the `naming-convention` rule's check `repo-dir`, rename the check to `package-dir`.
- Delete the files `spex update` leaves behind after their renames: `rules/naming-convention/repo-dir.sh` (now `package-dir.sh`), and `.agents/skills/spex-setup/references/GENERIC-REPO-SKILL-TEMPLATE.md` (now `GENERIC-PACKAGE-SKILL-TEMPLATE.md`) under the agent root.
