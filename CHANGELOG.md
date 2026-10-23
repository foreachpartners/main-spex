# Changelog

Public-facing and major changes to spex, following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions match `Cargo.toml` (`cli_version` in `spex.yaml`). `spex update` prints the entries newer than the version it's updating from.

## [0.27.1] - 2026-10-23

- The `spex-spec` and `spex-review-spec` skills read the PRD when `spex info` prints `prd`: `spex-spec` before drafting, so each spec lines up with the product's goal and capabilities, and `spex-review-spec` for the product's goal behind the spec it reviews. `spex update` rewrites the skills; no migration steps.

## [0.27.0] - 2026-10-22

- New `spex-prd` skill writes or changes the project's PRD, the optional root of the artifact chain, and records its path in the new optional `prd` field of `spex.yaml`, which `spex info` prints as an absolute path. `spex update` writes the skill and rewrites the schema; no migration steps.

## [0.26.1] - 2026-10-18

- The `spex-rule` skill also removes a rule: it asks which packages stop using it, drops its `id` from their `rules` in `spex.yaml`, and, once no package lists it, deletes the rule file and its script folder and adds the `id` to `rules_unused`, so `spex update` never redeploys a bundled rule that was removed on purpose. The skill's description now mentions removal. `spex update` rewrites the skill; no migration steps.

## [0.26.0] - 2026-10-17

- **Breaking**: a spec must set `packages`, at least one `package_short_name`: `schemas/spec.schema.json` now requires it, where it was optional before. A spec that sets none, or an empty list, fails `spex check` with a schema error. The "Gaps" section of `spex coverage` therefore no longer has a `no package` heading. Until every spec lists its packages, `spex check` fails. See the migration guide.

## [0.25.0] - 2026-10-16

- Each package entry of `spex.yaml`, and the `spex.package.yaml` of a package, accepts `exclude`: glob patterns, the same globs a check's `include` and `exclude` use, matched against the path of every file and folder relative to the package's root. `spex check` and `spex coverage` skip what they match for that package only, a folder that matches with everything inside it, so vendored or generated code is no longer checked and its `@spec` annotations count for no spec; `vendor` or `**/vendor` skips a folder, `vendor/**` the files inside it, and `**/*.generated.ts` such files at any depth. The two sources are merged by union like the ignores, `spex check --files` skips a listed file its package excludes without an error, and contract files declared in `spex.yaml` are not affected. An `exclude` pattern that is not a valid glob stops the command naming the package. `spex info` lists each package's merged `exclude`, and `schemas/spex.schema.json` and `schemas/spex.package.schema.json` follow. No migration steps.

## [0.24.0] - 2026-10-16

- **Breaking**: `ignored_errors`, the baseline of pre-existing errors in a package entry of `spex.yaml` and in `spex.package.yaml`, is now `check_ignore`, named after `spex check` the way `coverage_ignore` is named after `spex coverage`. `spex check --fill-preexisting-errors` writes `check_ignore`, `spex info` reports it under that key, and `schemas/spex.schema.json` and `schemas/spex.package.schema.json` follow. Until the key is renamed, `spex.yaml` and `spex.package.yaml` fail schema validation. See the migration guide.
- **Breaking**: an ignore now applies to the package that declares it. `coverage_ignore` of a `spex.package.yaml` used to waive a spec's requirement for the whole project; it now waives it for its own package only, and `spex coverage` waives a requirement of a spec once every package the spec lists in `packages` waives it, or the spec's own `coverage_ignore` does. A waiver of `api` alone no longer hides a missing requirement from `frontend`, and `spex check` errors on a package waiver naming a spec that does not list that package. See the migration guide.
- **Breaking**: the root `coverage_ignore` of `spex.yaml` is removed, since it was a spec's own `coverage_ignore` kept in another file: a waiver for a whole spec goes in the spec. Until it is removed, `spex.yaml` fails schema validation, and `spex info` no longer prints it. See the migration guide.
- Each package entry of `spex.yaml` accepts `coverage_ignore`, next to the `check_ignore` it already had, and applies it to that package only; `spex.package.yaml` is merged into its own package entry. `spex info` lists each package's merged `check_ignore` and `coverage_ignore` under it, and `schemas/spex.schema.json` and `schemas/spex.package.schema.json` follow.
- `spex check --fill-preexisting-errors` writes the baseline to `spex.yaml` as before, and, with `--package`, to the `spex.package.yaml` of each selected package instead, creating it when it is missing and there is something to write, and leaves out the errors the other file already baselines for the package. An empty `spex.package.yaml` is valid and merges nothing.

## [0.23.4] - 2026-10-15

- `spex coverage` lists every `in-progress` spec under its status group, marked `✗` when it has a gap and `●` otherwise, where before only specs with pending work and no gap were listed. Each shows the criteria not implemented yet as `in-progress: impl(<ac-id>, ...)`; the test and contract entries that used to follow are dropped, and a spec with no impl left to name shows no such line. "Gaps" no longer repeats that line and shows only what is missing. With `--verbose`, each `in-progress` spec shows its description and the text of the criteria not implemented yet. No migration steps.

## [0.23.3] - 2026-10-15

- `schemas/spec.schema.json` now says in the `acceptance_criteria` description that a spec needs at least one criterion, which the schema already required. `spex check` also no longer treats a spec with no acceptance criteria as a special case for its `status` warning: such a spec, already a schema error, now also gets that warning. No migration steps.

## [0.23.2] - 2026-10-15

- `spex check` validates `@spec` annotations in markdown files like in any other file, so a markdown contract (e.g. a design reference) is checked. It now skips only the spex directory's `schemas/`, `specs/`, `rules/` and `docs/`, and the agent root's `AGENTS.md` and `.agents/skills/`, which hold illustrative example annotations. Markdown files elsewhere that carry `@spec` examples now report findings. No migration steps.

## [0.23.1] - 2026-10-15

- `spex check` treats the spex directory as the package it is in every layout: `spex-internal-conformance` and the spec-scoped checks run on every `spex check`, whatever `--package` or `--files` select, and are reported under the spex package's heading, which is added when that package is not selected. In `multi-repo` they no longer wait for the spex package to be selected, and in `single-repo` and `monorepo` they no longer get a separate last `spex` heading. The `@spec` annotation checks still run only on the selected packages' files. No migration steps.

## [0.23.0] - 2026-10-15

- `spex init` no longer links `.claude` to `.agents`: it creates `.claude` as a real directory and links only `.claude/skills` to `../.agents/skills`, so Claude Code's settings and other files no longer end up in `.agents`. Only new projects are affected; the `spex-update` skill fixes an existing project's `.claude` link, see the migration guide.

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
