# Changelog

Public-facing and major changes to spex, following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions match `Cargo.toml` (`cli_version` in `spex.yaml`). `spex update` prints the entries newer than the version it's updating from.

## [0.29.0] - 2026-10-24

- Added new `cqrs-conventions` Command Query Responsibility Segregation conventions rule.
- Added new `ddd-conventions` Domain-Driven Design conventions rule.
- Added a `spex check` warning for a rule no package lists, suggesting to delete it.
- Changed the `spex-update` skill to ask which packages use each newly deployed rule and to delete the ones no package uses.
- Changed the `spex-assembly-line` skill to run each review in its own agent instead of in the orchestrator.

## [0.28.0] - 2026-10-23

- **Breaking**: `prd` is now a required field of `spex.yaml`. See the migration guide.
- **Breaking**: `spex check` errors on any file declared in `spex.yaml` (`prd`, contract paths) that does not exist. See the migration guide.

## [0.27.2] - 2026-10-23

- Changed the `spex-prd` skill to keep capabilities high level instead of capping them at about ten.

## [0.27.1] - 2026-10-23

- Changed the `spex-spec` and `spex-review-spec` skills to read the PRD when the project has one.

## [0.27.0] - 2026-10-22

- Added the `spex-prd` skill, which writes the project's PRD and records its path in the new optional `prd` field of `spex.yaml`.

## [0.26.1] - 2026-10-18

- Changed the `spex-rule` skill to also remove a rule, adding it to `rules_unused` so `spex update` does not redeploy it.

## [0.26.0] - 2026-10-17

- **Breaking**: a spec must now set `packages`. See the migration guide.

## [0.25.0] - 2026-10-16

- Added `exclude` globs to a package entry in `spex.yaml` and to `spex.package.yaml`; `spex check` and `spex coverage` skip the matching files of that package.

## [0.24.0] - 2026-10-16

- **Breaking**: `ignored_errors` is now `check_ignore`. See the migration guide.
- **Breaking**: `coverage_ignore` in `spex.package.yaml` now waives a requirement for its own package only. See the migration guide.
- **Breaking**: removed the root `coverage_ignore` of `spex.yaml`; a waiver for a whole spec goes in the spec. See the migration guide.
- Added `coverage_ignore` to a package entry in `spex.yaml`.
- Changed `spex check --fill-preexisting-errors` with `--package` to write the baseline to each selected package's `spex.package.yaml`.

## [0.23.4] - 2026-10-15

- Changed `spex coverage` to list every `in-progress` spec under its status group, with the criteria not implemented yet.

## [0.23.3] - 2026-10-15

- Changed `spex check` to also give the `status` warning to a spec with no acceptance criteria.

## [0.23.2] - 2026-10-15

- Changed `spex check` to validate `@spec` annotations in markdown files, so markdown contracts are checked.

## [0.23.1] - 2026-10-15

- Changed `spex check` to always run the spex directory's conformance and spec checks, whatever `--package` or `--files` select.

## [0.23.0] - 2026-10-15

- Changed `spex init` to create `.claude` as a real directory and link only `.claude/skills`. Existing projects: see the migration guide.

## [0.22.0] - 2026-10-15

- **Breaking**: `spex update` now syncs a bundled rule that sets no `sync_with_upstream`; set `sync_with_upstream: false` to keep local changes. See the migration guide.
- Changed `spex update --migration-guide` to print steps newest first and accept `MAJOR` and `MAJOR.MINOR` versions.
- Changed `spex update` to require `agent_root` in `spex.yaml`.

## [0.21.4] - 2026-10-15

- Added `spex update --migration-guide <from_version>`, which prints the migration steps after a version; the `spex-update` skill applies them before updating.

## [0.21.3] - 2026-10-15

- Changed `spex info` to merge every package's `spex.package.yaml` into its output.

## [0.21.2] - 2026-10-15

- Changed `spex init` to name the spex package `spex` in `single-repo` and `monorepo`.

## [0.21.1] - 2026-10-14

- Changed the `package-dir` check of the `naming-convention` rule to run only in the `multi-repo` layout.

## [0.21.0] - 2026-10-14

- **Breaking**: a spec's `scope` is now `packages`. See the migration guide.
- **Breaking**: `spex coverage` and `spex specs` take `--package` instead of `--scope`.

## [0.20.0] - 2026-10-14

- **Breaking**: "workspace" is no longer a spex term; a script check's `run: workspace` is now `run: project`. See the migration guide.

## [0.19.1] - 2026-10-14

- Changed the artifact chain to start at the PRD, which replaces the BRD.
- Changed the single-source-of-truth rule in the managed section of `AGENTS.md` to cover business rules, behavior, contracts, and configuration values.

## [0.19.0] - 2026-10-12

- Added the `single-repo` and `monorepo` layouts to `spex init`; existing projects must add `layout` and `agent_root` to `spex.yaml`. See the migration guide.
- **Breaking**: a workspace repository is now a package: `repositories` is `packages`, `--repo` is `--package`, `run: per-repo` is `run: per-package`, and `spex.repo.yaml` is `spex.package.yaml`. See the migration guide.
