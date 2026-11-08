# Migration guide

Steps required to move a project from one spex CLI version to a newer one, one section per version that needs them. A version not listed here needs no migration steps.

`spex update --migration-guide <from_version>` prints the sections after a version, newest first, and the `spex-update` skill reads them all and applies them from the oldest version to the newest before it runs `spex update`: a step must not rely on files `spex update` writes. Keep every version heading as `## MAJOR.MINOR.PATCH`.

## 0.34.0

A package skill now keeps `references/review.md`, what earlier reviews of the package kept running into, and says itself when to read and edit it. `spex update` never overwrites a package skill, so one made by an earlier version lacks these. In `<agent-root>/.agents/skills`, for every package skill `<package-name>/SKILL.md`:

- In the frontmatter `description`, change `testing, or running` to `testing, reviewing, or running`, unless `reviewing` is already there.
- Unless `SKILL.md` already has a `# Review` heading, insert this section right before `# Index`, or at the end when there is no `# Index`, followed by a blank line:

  ```markdown
  # Review

  Before reviewing changes in this package, read `references/review.md`: it holds what earlier reviews of this package kept running into.

  Add to it only when a review hit something that cost real time to resolve or made you ask the user, and the next review would hit it again. For example:

  - a pattern that looks like a violation but is intended, with the reason
  - a check or command that doesn't work as documented, with the workaround
  - a decision the user already made that a reviewer would otherwise ask about again

  Most reviews add nothing. Never add: a finding about the change under review; anything a spec, contract, rule or the code already states. Edit existing lines in place instead of appending, drop one that no longer holds, and when unsure leave it out and mention it in the report.
  ```

- Unless `# Index` already lists `references/review.md`, add the line "- `references/review.md` - what earlier reviews of this package kept running into" as the first entry of `# Index`. When there is no `# Index`, add this at the end of `SKILL.md` instead:

  ```markdown
  # Index

  Read individual files for detailed explanations and code examples:

  - `references/review.md` - what earlier reviews of this package kept running into
  ```

- Next to `SKILL.md`, create `references/review.md` containing only the heading `# Review notes`, unless it already exists.

The `spex-cavemen` skill is no longer bundled, and `spex update` never deletes a skill. If `<agent-root>/.agents/skills/spex-cavemen` exists, delete that folder.

Every `coverage_ignore` entry now needs a `reason`, one short clause saying why the requirement is waived. Until it has one, `spex check` and `spex coverage` fail on the file holding the entry. In `<spex-dir>`, find every `coverage_ignore` entry that has no `reason` yet: in the specs in `specs/`, in the package entries of `spex.yaml`, and in each package's `spex.package.yaml`. Add `reason: <why>` after the entry's other keys to each. Work the reason out from the spec, the code and the git history; when you cannot tell why the requirement is waived, ask the user instead of inventing one, and never write a placeholder such as `n/a`.

## 0.33.0

`spex-implement` no longer updates package skill documentation (its step "Update package skill documentation" is removed). Instead, the package skill itself says when and how to change it. `spex update` never overwrites a package skill, so one made by an earlier version lacks these rules and nothing keeps it current. In `<agent-root>/.agents/skills`, for every package skill `<package-name>/SKILL.md`:

- In the frontmatter `description`, insert `Keep it up to date with important changes.` before `MUST be loaded`, unless it is already there.
- Unless `# General` already starts with `Default: change nothing.`, insert this text at the start of `# General`, followed by a blank line, before its first line (the one that starts with "Run"):

  ```markdown
  Default: change nothing. A package skill is a short orientation for an agent
  entering the package, not a changelog or feature catalog. Specs, contracts,
  rules and code are the sources of truth for what the package does.

  Edit a package skill only when the work did one of these:

  - added or removed a top-level directory, bounded context, crate or entry point
    (one line, directory level)
  - introduced a package-wide convention or architectural pattern that every later
    feature must follow, and no spec or rule states it
  - hit a tooling quirk that cost real time and can't be discovered from the code

  Never add: a feature, endpoint, field, error code, type, function or file;
  anything a spec, contract, rule or the code already states; a how-to for one
  feature.

  Before writing a line, ask whether an agent working on an unrelated feature in
  this package would be misled or slowed without it. If not, skip it. Edit
  existing text in place instead of appending. When unsure, leave it out and
  mention it in the report.
  ```

- Wherever your own text (`AGENTS.md` below the managed section, own skills, docs, CI jobs) expects `spex-implement` to update package skills or to report which ones it updated, drop that.

## 0.31.1

`spex update` no longer deploys `docs/glossary.md`, since the managed section of `AGENTS.md` inlines the glossary. In `<spex-dir>`:

- Delete `docs/glossary.md`, which `spex update` no longer keeps in sync.
- Wherever your own text (`AGENTS.md` below the managed section, specs, rules, skills, docs, CI jobs) links to or reads `docs/glossary.md`, point it at the glossary in `AGENTS.md` instead.

## 0.31.0

`spex check` now errors on a blank line between the keys of a spec or rule file (`spex-internal-conformance/no-blank-lines`). In `<spex-dir>`:

- In every spec under `specs/**/*.yaml` and every rule under `rules/**/*.yaml`, delete each empty or whitespace-only line that is not inside a multi-line value (a `|` or `>` block or a multi-line quoted string). Blank lines inside such a value, e.g. in an example, stay.
- Run `spex check` and confirm it reports no `no-blank-lines` error.

`spex specs` and `spex rules` no longer take `--ai`, and the CLI errors on it. In every CI job, script, and own skill or doc of the project (the part of `AGENTS.md` below the managed section, package skills, docs):

- Drop `--ai` from every `spex specs` and `spex rules` command.
- Add `--fields id path title description acceptance_criteria examples` to a `spex rules` command whose reader needs no `checks`, as `--ai` did.

`rules_unused` is removed from `spex.yaml`, and `spex init` and `spex update` no longer deploy bundled rules: `schemas/spex.schema.json` rejects `rules_unused`, and `spex update` only keeps in sync the rules the project already holds. A project takes a bundled rule from the rules library instead. In `<spex-dir>`:

- Delete the root `rules_unused` key of `spex.yaml`, with its list.
- Delete the files and script folders of the rules in `rules/` that no package lists under `rules` (`spex check` warns on each), unless the user wants to keep one. `spex update` no longer deploys a deleted rule again.
- To take a bundled rule the project does not hold, such as one from the deleted `rules_unused` list, run `spex lib rules --id <rule-id>... --copy rules`, then list its id under `rules` of each package that follows it. `spex lib rules --fields id description` lists every bundled rule.
- In every CI job, script, skill, or doc of the project that reads `rules_unused` from the output of `spex info`, drop it: the key is gone.

## 0.28.0

`prd`, the path of the project's PRD relative to `<spex-dir>`, is now a required field of `spex.yaml`: `schemas/spex.schema.json` requires it, and the CLI cannot load a `spex.yaml` without it. The PRD is the short project overview that comes first, what is built and why, without details, which agents read before they write or review specs. In `<spex-dir>`:

- If `spex.yaml` sets `prd` and the file it names exists, change nothing.
- Otherwise ask the user whether the project already has a PRD or product brief file. If it does, set `prd` in `spex.yaml` to its path relative to `<spex-dir>`, after `description` (after `agent_root` when there is none).
- If it has none, set `prd: docs/prd.md` the same way, then write `<spex-dir>/docs/prd.md` by interviewing the user about the product, one question at a time, pre-filling every answer from `spex.yaml`, the package descriptions, and the specs (`spex specs --fields id title description`). Use the `spex-prd` skill when `<agent-root>/.agents/skills/spex-prd` exists; a project older than 0.27.0 has no such skill, so write `# Product requirements` with `## Problem`, `## Goal`, `## Users`, `## Success`, `## Capabilities` (the main blocks, one line each: a bold name and one sentence of user-visible outcome) and `## Non-goals`, short and without details.

`spex check` now also errors on every file `spex.yaml` declares that does not exist: the `prd` file, handled above, and the `path` of each `contracts` entry. In `<spex-dir>`, for every `contracts` entry whose `path` (relative to `<spex-dir>`) names no existing file, ask the user whether the path is wrong or the contract is not written yet. Fix the path in the first case; in the second, write the contract with the `spex-contract` skill when `<agent-root>/.agents/skills/spex-contract` exists, or drop the entry.

## 0.26.0

A spec must now set `packages`, at least one: `schemas/spec.schema.json` requires it. A spec that sets none fails `spex check` with a schema error until it lists the packages that implement it, and the "Gaps" section of `spex coverage` no longer has a `no package` heading. In `<spex-dir>`:

- For every spec under `specs/**/*.yaml` that sets no `packages`, find the packages that implement it from its description and acceptance criteria and from the package descriptions in `spex.yaml`. If it is unclear, ask the user. Add `packages`, listing each `package_short_name`, right after `acceptance_criteria`.
- In every CI job or script that reads the `spex coverage` report for the `no package` heading, drop that check.

## 0.24.0

The baseline of pre-existing errors, `ignored_errors`, is now `check_ignore`, like `coverage_ignore` for `spex coverage`. Each entry keeps its `rule_id`, `check` and `files`. Until it is renamed, `spex.yaml` and every `spex.package.yaml` fail schema validation. In `<spex-dir>`:

- In `spex.yaml`, rename every `ignored_errors` key, in the package entries, to `check_ignore`, keeping each entry and its position.
- In every package that has a `spex.package.yaml`, rename its root `ignored_errors` key to `check_ignore`.
- In every CI job, script, skill, or doc of the project that reads the `ignored_errors` key of `spex info` output, read `check_ignore` instead.
- Wherever your own text (`AGENTS.md` below the managed section, specs, rules, skills, docs) mentions `ignored_errors`, say `check_ignore`.

The root `coverage_ignore` of `spex.yaml` is removed, since it only kept a spec's own `coverage_ignore` in another file, and a `coverage_ignore` of a `spex.package.yaml`, which used to waive a spec's requirement for the whole project, now waives it for its own package only. A spec's requirement is waived once every package the spec lists in `packages` waives it, or the spec's own `coverage_ignore` does. A package waiver naming a spec that does not list the package no longer waives anything, and `spex check` reports it as an error. Until the root `coverage_ignore` is removed, `spex.yaml` fails schema validation. In `<spex-dir>`:

- For every entry of the root `coverage_ignore` of `spex.yaml`, add it, without its `spec_id`, to the `coverage_ignore` of the spec it names, skipping an entry the spec already has, then remove the root `coverage_ignore` key. This keeps the old effect.
- For every entry of the `coverage_ignore` of a `spex.package.yaml`, look up the spec it names. If the spec's `packages` lists only that package, change nothing. Otherwise add the entry, without its `spec_id`, to the spec's own `coverage_ignore`, which keeps the old effect, unless the user says it should stay package-specific: then keep it in the package and add the same entry to each other package the spec lists.
- In every CI job, script, skill, or doc of the project that reads the root `coverage_ignore` of `spex info` output, read the `coverage_ignore` of each entry of `packages` instead: the root key is gone.

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
