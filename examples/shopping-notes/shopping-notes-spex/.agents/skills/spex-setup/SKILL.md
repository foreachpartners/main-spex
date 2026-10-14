---
name: spex-setup
description: Configure one or multiple new packages for usage with spex framework, define them in spex.yaml configuration, create package skill for each. Use this skill when user wants to setup new packages in spex framework.
---

# Set up new packages

Define each new package in `spex.yaml`, create its skill, and add it to the editor workspace file.

## Paths

Replace `<spex-dir>` and `<agent-root>` with the spex directory and the agent root, printed by `spex info | spex jq -r .spex_dir` and `spex info | spex jq -r .agent_root`. The project's layout is `spex info | spex jq -r .layout`: `multi-repo`, `single-repo`, or `monorepo`.

| What                  | Path                                                         |
| --------------------- | ------------------------------------------------------------ |
| Project config        | `<spex-dir>/spex.yaml`                                       |
| Config schema         | `<spex-dir>/schemas/spex.schema.json`                        |
| Rules                 | `<spex-dir>/rules/`                                          |
| Package skills        | `<agent-root>/.agents/skills/`                               |
| Editor workspace file | `<spex-dir>/<project-name>.code-workspace` (multi-repo only) |

The schema is authoritative for field rules and contains a worked example. Read it before editing `spex.yaml`.

## Step 1 — Define the packages in spex.yaml

1. Read `spex.yaml`.
2. Find the new packages, by layout. Confirm the list with the user and drop anything that is not a package, or that `packages` already lists.
   - `multi-repo`: the non-hidden directories directly under the project root (the parent of `<spex-dir>`), or the ones the user named.
   - `single-repo`: none. `spex init` already listed the project's one package; only complete its entry.
   - `monorepo`: the packages, from the workspace manifest (`pnpm-workspace.yaml`, `workspaces` in `package.json`, `[workspace]` in `Cargo.toml`, `go.work`) or the conventional folders (`apps/*`, `packages/*`, `services/*`), or the ones the user named.
3. Inspect each package: its `README`, manifest files (`package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`), top-level layout and entry points.
4. If the project `description`, or a package `description`, is `unknown`, propose one from `project_name` and what the packages do. If their purpose is unclear, ask the user one short question instead of guessing.
5. Propose an entry for each package:
   - `package_name`: the directory name in multi-repo, `<project-name>-<package-short-name>` in monorepo
   - `package_short_name`: in multi-repo the `package_name` without its `<project-name>-` prefix, in monorepo the package's folder name
   - `description`: one concrete sentence stating the package's role in the project (for example backend service, web client, shared library), without restating its name
   - `path`, relative to `<spex-dir>`: `../<directory-name>` in multi-repo, `../<package-path>` (for example `../apps/api`) in monorepo
   - `contracts`: one entry per interface contract file the package ships (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI), each with a `path` relative to the spex directory and a one-line `description`. Leave the file where it is; never move or copy it. Omit `contracts` when the package ships none.
   - `rules`: read every file in `rules/` and list only the rule IDs that genuinely apply given the package's stack and role (for example `openapi-conventions` only for a package that exposes a REST API). Give a one-line reason for each.
6. Show the proposal to the user and confirm it. Write only what the user confirms.
7. Append the entries to `packages`, sorted by `package_name`. Set the project `description` if it was proposed. Leave every existing field, entry and rule file untouched.
8. Run `spex check-schema --schema schemas/spex.schema.json --files spex.yaml` from `<spex-dir>` and fix any violation.

Example entry:

```yaml
- package_name: shopping-notes-api
  package_short_name: api
  description: Backend service that handles shopping items.
  path: ../shopping-notes-api
  contracts:
    - path: ../shopping-notes-api/contracts/api.openapi.yaml
      description: REST OpenAPI spec for the shopping items API.
  rules:
    - openapi-conventions
    - secret-handling
```

## Step 2 — Create each package skill

Create `<agent-root>/.agents/skills/<package-name>/SKILL.md` for each new package from [`references/GENERIC-PACKAGE-SKILL-TEMPLATE.md`](references/GENERIC-PACKAGE-SKILL-TEMPLATE.md). Never overwrite a `SKILL.md` that already exists.

- Keep the template's frontmatter, sections and order. Replace its `<package-name>` and `<package-short-name>` placeholders.
- Use the package `description` from Step 1 in `# General`.
- Fill each section with real facts from the package (layout, manifest scripts, CI config, contributor docs). Never invent a path or command.
- Omit a section when the package has nothing genuine for it (for example `# Rules` when no test, lint or format policy exists).
- Move substantial package-specific material (a non-obvious workflow, a pattern worth a worked example) into its own file under the skill folder's `references/`, `scripts/` or `assets/`, one topic per file. List each file in `# Index` with a one-line description. Omit `# Index` when there are no such files.

## Step 3 — Add the packages to the editor workspace file

Skip this step in `single-repo` and `monorepo`: they have no editor workspace file.

Read `<project-name>.code-workspace`. For each new package, append a folder to `folders`, leaving existing folders untouched:

```json
{ "name": "🔌 api", "path": "../shopping-notes-api" }
```

- `name`: one emoji fitting the package's role, a space, and the `package_short_name`
- `path`: the package's `path` from `spex.yaml`

Icons:

- 🔌 api
- 🖥️ frontend
- 📱 mobile
- 💻 desktop
- ⌨️ cli
- 🛢️ database
- 🏗️ infra
- 📊 analytics
- 🔐 admin
- 🧩 sdk
- 🤖 ml

The file may keep a trailing comma after the last folder.

## Next

Offer to author the first spec with `spex-spec` (a capability the system must provide) or `spex-rule` (a cross-cutting convention). Note that it is better to start a new chat first to save context.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
