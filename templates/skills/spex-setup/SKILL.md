---
name: spex-setup
description: Configure one or multiple new project repositories for usage with spex gramework, define them in spex.yaml configuration, create repository skill for each. Use this skill when user wants to setup new project repositories in spex framework.
---

# Set up new repositories

Define each new workspace repository in `spex.yaml`, create its skill, and add it to the editor workspace file.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name.

| What                  | Path                                                                 |
| --------------------- | -------------------------------------------------------------------- |
| Workspace config      | `<workspace-root>/<project-name>-spex/spex.yaml`                     |
| Config schema         | `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`      |
| Rules                 | `<workspace-root>/<project-name>-spex/rules/`                        |
| Repository skills     | `<workspace-root>/<project-name>-spex/.agents/skills/`               |
| Editor workspace file | `<workspace-root>/<project-name>-spex/<project-name>.code-workspace` |

The schema is authoritative for field rules and contains a worked example. Read it before editing `spex.yaml`.

## Step 1 — Define the repositories in spex.yaml

1. Read `spex.yaml`.
2. Find the new repositories: the non-hidden directories directly under `<workspace-root>` that `repositories` does not list, or the ones the user named. Confirm the list with the user and drop any directory that is not a workspace repository.
3. Inspect each repository: its `README`, manifest files (`package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`), top-level layout and entry points.
4. If the project `description` is `unknown`, propose one from `project_name` and what the repositories do. If their purpose is unclear, ask the user one short question instead of guessing.
5. Propose an entry for each repository:
   - `repository_name`: the directory name
   - `repository_short_name`: the `repository_name` without its `<project-name>-` prefix
   - `description`: one concrete sentence stating the repository's role in the project (for example backend service, web client, shared library), without restating its name
   - `path`: `../<directory-name>`
   - `contracts`: one entry per interface contract file the repository ships (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI), each with a `path` relative to the spex directory and a one-line `description`. Leave the file where it is; never move or copy it. Omit `contracts` when the repository ships none.
   - `rules`: read every file in `rules/` and list only the rule IDs that genuinely apply given the repository's stack and role (for example `openapi-conventions` only for a repository that exposes a REST API). Give a one-line reason for each.
6. Show the proposal to the user and confirm it. Write only what the user confirms.
7. Append the entries to `repositories`, sorted by `repository_name`. Set the project `description` if it was proposed. Leave every existing field, entry and rule file untouched.
8. Run `spex check-schema --schema schemas/spex.schema.json --files spex.yaml` from the spex repository and fix any violation.

Example entry:

```yaml
- repository_name: shopping-notes-api
  repository_short_name: api
  description: Backend service that handles shopping items.
  path: ../shopping-notes-api
  contracts:
    - path: ../shopping-notes-api/contracts/api.openapi.yaml
      description: REST OpenAPI spec for the shopping items API.
  rules:
    - openapi-conventions
    - secret-handling
```

## Step 2 — Create each repository skill

Create `<workspace-root>/<project-name>-spex/.agents/skills/<repository-name>/SKILL.md` for each new repository from [`references/GENERIC-REPO-SKILL-TEMPLATE.md`](references/GENERIC-REPO-SKILL-TEMPLATE.md). Never overwrite a `SKILL.md` that already exists.

- Keep the template's frontmatter, sections and order. Replace its `<repository-name>` and `<repository-short-name>` placeholders.
- Use the repository `description` from Step 1 in `# General`.
- Fill each section with real facts from the repository (layout, manifest scripts, CI config, contributor docs). Never invent a path or command.
- Omit a section when the repository has nothing genuine for it (for example `# Rules` when no test, lint or format policy exists).
- Move substantial repository-specific material (a non-obvious workflow, a pattern worth a worked example) into its own file under the skill folder's `references/`, `scripts/` or `assets/`, one topic per file. List each file in `# Index` with a one-line description. Omit `# Index` when there are no such files.

## Step 3 — Add the repositories to the editor workspace file

Read `<project-name>.code-workspace`. For each new repository, append a folder to `folders`, leaving existing folders untouched:

```json
{ "name": "🔌 api", "path": "../shopping-notes-api" }
```

- `name`: one emoji fitting the repository's role, a space, and the `repository_short_name`
- `path`: the repository's `path` from `spex.yaml`

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
