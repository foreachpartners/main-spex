# `spex-setup` skill

Configures one or several new packages for the spex framework: defines each in `spex.yaml`, creates a package skill for each, and adds it to the editor workspace file. Loaded when the user wants to set up new packages in the spex framework, typically after [`spex init`](../cli/spex-init-cli-cmd.md).

## Acceptance criteria

### Paths

- MUST resolve `<spex-dir>` and `<agent-root>` from `spex info | spex jq -r .spex_dir` and `spex info | spex jq -r .agent_root`, and the project's layout (`multi-repo`, `single-repo`, or `monorepo`) from `spex info | spex jq -r .layout`.
- MUST read `<spex-dir>/schemas/spex.schema.json`, the authority for field rules with a worked example, before editing `spex.yaml`.

### Step 1: define the packages in spex.yaml

- MUST read `spex.yaml`.
- MUST find the new packages by layout: in `multi-repo`, the non-hidden directories directly under the project root (the parent of `<spex-dir>`), or the ones the user named; in `single-repo`, none, since `spex init` already listed the project's one package and only its entry is completed; in `monorepo`, the packages from the workspace manifest (`pnpm-workspace.yaml`, `workspaces` in `package.json`, `[workspace]` in `Cargo.toml`, `go.work`) or the conventional folders (`apps/*`, `packages/*`, `services/*`), or the ones the user named.
- MUST confirm the list with the user, and drop anything that is not a package or that `packages` already lists.
- MUST inspect each package: its `README`, manifest files (`package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`), top-level layout and entry points.
- MUST propose a description from `project_name` and what the packages do when the project `description`, or a package `description`, is `unknown`, and ask the user one short question instead of guessing when the purpose is unclear.
- MUST propose, for each package, a `package_name` (the directory name in `multi-repo`, `<project-name>-<package-short-name>` in `monorepo`).
- MUST propose, for each package, a `package_short_name` (in `multi-repo` the `package_name` without its `<project-name>-` prefix, in `monorepo` the package's folder name).
- MUST propose, for each package, a `description`: one concrete sentence stating the package's role in the project (e.g. backend service, web client, shared library), without restating its name.
- MUST propose, for each package, a `path` relative to `<spex-dir>`: `../<directory-name>` in `multi-repo`, `../<package-path>` (e.g. `../apps/api`) in `monorepo`.
- MUST propose, for each package, `contracts`: one entry per interface contract file the package ships (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI), each with a `path` relative to the spex directory and a one-line `description`, leaving the file where it is; omit `contracts` when the package ships none.
- MUST propose, for each package, `rules`: read every file in `rules/` and list only the rule IDs that genuinely apply given the package's stack and role (e.g. `openapi-conventions` only for a package that exposes a REST API), with a one-line reason for each.
- MUST show the proposal to the user, confirm it, and write only what the user confirms.
- MUST append the entries to `packages`, sorted by `package_name`, set the project `description` if it was proposed, and leave every existing field, entry and rule file untouched.
- MUST run `spex check-schema --schema schemas/spex.schema.json --files spex.yaml` from `<spex-dir>` and fix any violation.
- MUST NOT move or copy a contract file.

### Step 2: create each package skill

- MUST create `<agent-root>/.agents/skills/<package-name>/SKILL.md` for each new package from `references/GENERIC-PACKAGE-SKILL-TEMPLATE.md`, keeping the template's frontmatter, sections and order, and replacing its `<package-name>` and `<package-short-name>` placeholders ([package-skill-skill.md](package-skill-skill.md)).
- MUST use the package `description` from Step 1 in `# General`.
- MUST fill each section with real facts from the package (layout, manifest scripts, CI config, contributor docs).
- MUST move substantial package-specific material (a non-obvious workflow, a pattern worth a worked example) into its own file under the skill folder's `references/`, `scripts/` or `assets/`, one topic per file, and list each file in `# Index` with a one-line description.
- MUST omit `# Index` when there are no such files.
- MUST omit a section when the package has nothing genuine for it (e.g. `# Rules` when no test, lint or format policy exists).
- MUST NOT overwrite a `SKILL.md` that already exists.
- MUST NOT invent a path or command.

### Step 3: add the packages to the editor workspace file

- MUST skip this step in `single-repo` and `monorepo`, which have no editor workspace file.
- MUST read `<project-name>.code-workspace` and append a folder to `folders` for each new package, leaving existing folders untouched, as `{ "name": "<emoji> <package_short_name>", "path": "<the package's path from spex.yaml>" }`, e.g. `{ "name": "🔌 api", "path": "../shopping-notes-api" }`.
- MUST give `name` one emoji fitting the package's role, a space, and the `package_short_name`: 🔌 api, 🖥️ frontend, 📱 mobile, 💻 desktop, ⌨️ cli, 🛢️ database, 🏗️ infra, 📊 analytics, 🔐 admin, 🧩 sdk, 🤖 ml.
- MAY leave a trailing comma after the last folder in the file.

### Next

- MUST offer to author the first spec with `spex-spec` (a capability the system must provide) or `spex-rule` (a cross-cutting convention), noting that starting a new chat first saves context.
