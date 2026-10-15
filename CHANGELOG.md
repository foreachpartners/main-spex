# Changelog

Public-facing and major changes to spex, following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions match `Cargo.toml` (`cli_version` in `spex.yaml`). `spex update` prints the entries newer than the version it's updating from.

## [0.22.0] - 2026-10-15

- **Breaking**: a bundled rule that sets no `sync_with_upstream` is now synced from upstream by `spex update`, which overwrites it and its scripts. The field defaults to `true`, and `sync_with_upstream: false` opts a rule out to keep local changes, where before an unset field kept the rule as a local customization. `schemas/rule.schema.json` follows, and the bundled rules no longer carry `sync_with_upstream: true`. See the migration guide.
- `spex update --migration-guide <from_version>` prints the steps newest first, prints `no migration needed` when none follow, and accepts `MAJOR` and `MAJOR.MINOR` besides `MAJOR.MINOR.PATCH`, reading the missing parts as `0`. The `spex-update` skill reads all the steps and applies them from the oldest version to the newest.
- `spex update` requires `agent_root` in `spex.yaml` and fails without it, instead of assuming the spex directory, and no longer re-creates a missing `.claude` symlink, which `spex init` still creates. `spex update` rewrites the bundled schema and skill.

## [0.21.4] - 2026-10-15

- `spex update --migration-guide <from_version>` prints the migration steps of the installed `spex` for every version after `<from_version>`, oldest first, without reading `spex.yaml` or changing any file. The `spex-update` skill now runs it and applies the steps before `spex update`, so a breaking change no longer meets an update that cannot handle the project. `spex update` rewrites the skill; no migration steps.

## [0.21.3] - 2026-10-15

- `spex info` merges every package's `spex.package.yaml` into its output: the `coverage_ignore` entries join the root `coverage_ignore`, and the `ignored_errors` entries join the `ignored_errors` of their package, the same union `spex check` and `spex coverage` use. No migration steps.

## [0.21.2] - 2026-10-15

- `spex init` names the spex package `spex` in `single-repo` and `monorepo`, the same as its `spex/` folder, and keeps `<project-name>-spex` in `multi-repo` only, where the project name prefix tells the spex repository apart from the other packages. Only new projects are affected; no migration steps.

## [0.21.1] - 2026-10-14

- The `package-dir` check of the bundled `naming-convention` rule now runs only in the `multi-repo` layout, read from `layout` in `spex.yaml`. In `single-repo` and `monorepo` the package directory is named after the repository or the folder structure, so the check passes there. `spex update` rewrites the rule and its script; no migration steps.

## [0.21.0] - 2026-10-14

- Naming is aligned: a package is now called a package everywhere, where `scope` was a second name for the same thing in a spec's field, in the flag of `spex coverage` and `spex specs`, and in the skills.
- **Breaking**: a spec's `scope` is now `packages`, since it always listed packages: the `package_short_name`s of the packages that implement the capability. `schemas/spec.schema.json` and the `spex check` finding that requires each entry to match a package follow. See the migration guide.
- **Breaking**: `spex coverage` and `spex specs` filter by `--package <name>...` instead of `--scope <package-short-name>...`. It takes a full `package_name` or a `package_short_name`, like `--package` of `spex check`, `spex info` and `spex rules`, and an unknown name errors with `unknown package`. The "Gaps" section of `spex coverage` lists a spec that sets no `packages` under `no package` instead of `unscoped`.
- The plan written by the `spex-plan` skill lists its packages under `Packages` instead of `Packages scope`; the skills no longer call a set of packages a scope.
- `spex update` rewrites the bundled schema, skills, and the CLI commands list in the managed section of `AGENTS.md` accordingly. No migration steps for the skill wording.

## [0.20.0] - 2026-10-14

- **Breaking**: "workspace" is no longer a spex term. Since `single-repo` and `monorepo` joined `multi-repo` in 0.19.0, a workspace of several repositories exists in only one of the three layouts, so the word no longer fits. The whole is now the **project**, and its top-level folder is the **project root** (formerly the workspace root): the folder grouping the packages in `multi-repo`, the repository root otherwise. A script check's `run: workspace` is now `run: project`, validated by `schemas/check.schema.json`, and `spex check` reports the findings of those scripts under a `project` heading instead of `workspace`. The error and schema texts, the bundled glossary, skills, and the managed section of `AGENTS.md` follow; `spex update` rewrites them. The editor's `<project-name>.code-workspace` file keeps its name. See the migration guide.

## [0.19.1] - 2026-10-14

- The artifact chain no longer has a BRD: the PRD is now the root artifact and covers both why something is built and what the product should do. `spex update` rewrites the bundled glossary and the managed section of `AGENTS.md` accordingly; no migration steps.
- The single-source-of-truth rule in the managed section of `AGENTS.md` now covers business rules, behavior, contracts, and configuration values, and states that repeating wording or an idea for readability is not duplication. `spex update` rewrites it; no migration steps.

## [0.19.0] - 2026-10-12

- `spex init` supports `single-repo` and `monorepo` layouts besides `multi-repo`: it asks for the layout, records it as `layout` with `agent_root` in `spex.yaml`, puts the spex directory in `spex/` and the agent files (`AGENTS.md`, `CLAUDE.md`, `.agents/skills`, `.claude`) in the repository root. Both are required, so existing projects must add `layout: multi-repo` and `agent_root: .` to `spex.yaml` (see the migration guide).
- **Breaking**: a workspace repository is now a **package**, one word for a separate git repository (multi-repo), a package folder (monorepo) and the single repository (single-repo). In `spex.yaml`, `repositories` is now `packages`, `repository_name` is `package_name`, and `repository_short_name` is `package_short_name`. The optional per-repository `spex.repo.yaml` is now `spex.package.yaml`, validated by `schemas/spex.package.schema.json`. The `--repo` option of `spex check` and `spex info` is now `--package`, and a check's `run: per-repo` is now `run: per-package`. The bundled `naming-convention` rule's `repo-dir` check is now `package-dir` (and its script `package-dir.sh`), and the `spex-setup` skill's `GENERIC-REPO-SKILL-TEMPLATE.md` is now `GENERIC-PACKAGE-SKILL-TEMPLATE.md`. See the migration guide.
