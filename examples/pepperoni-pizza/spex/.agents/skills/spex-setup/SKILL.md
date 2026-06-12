---
name: spex-setup
description: Fill in the missing descriptions in the workspace config (spex.yml). Use when the user wants to set up, describe, or document their spex project — adds a short description for the whole project and for each workspace repository by inspecting their contents. Writes a schema-valid spex.yml.
---

# Describe the Workspace (spex.yml)

Add a short `description` for the project and for each workspace repository in `spex.yml`. Inspect each repository to understand what it is, then write concise summaries. The result must stay valid against the workspace config schema.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace_root>` with the real folder name.

| What                  | Path                                            |
| --------------------- | ----------------------------------------------- |
| Spex repository       | `<workspace_root>/spex/`                         |
| Workspace config      | `<workspace_root>/spex/spex.yml`                 |
| Workspace config schema | `<workspace_root>/spex/schemas/spex.schema.json` |

The schema at `<workspace_root>/spex/schemas/spex.schema.json` is authoritative for field rules and contains a worked example. Read it before editing.

## When to use this skill

Use it when the user wants to set up, describe, or document their spex project — typically right after `spex init`, when descriptions are missing. Do not use it to add repositories or change paths; it only fills in `description` fields.

## Step 1 — Read the config

Read `<workspace_root>/spex/spex.yml`. Note the `project_name`, every repository's `repository_name` and `path`, and which `description` fields are already present.

Decide what to fill in:

- Always propose a project `description` if it is missing.
- Propose a `description` for each repository that is missing one.
- By default, leave existing descriptions untouched. Only rewrite one if the user asks.

## Step 2 — Inspect each repository

For each repository missing a description, look at its real contents (resolve `path` relative to `<workspace_root>/spex/`). Gather just enough to summarize what it is and does:

- Read its `README` if present.
- Check manifest/metadata files to identify the stack and purpose (for example `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`).
- Skim the top-level directory layout and entry points when the above are missing.

Infer the project `description` from `project_name` and the combined picture of its repositories. If the repositories don't make the project's purpose clear, ask the user one short question rather than guessing.

## Step 3 — Write the descriptions

Each `description` must be a single, plain sentence that states what the thing is and does:

- Project: what the product is and the problem it solves.
- Repository: its role within the project (for example backend service, web client, shared library, infrastructure).

Keep each description to one line, concrete, and free of filler. Match the imperative, no-hype style of the existing specs. Do not restate the name.

## Step 4 — Confirm, then write

1. Show the proposed descriptions to the user and confirm them.
2. Edit `<workspace_root>/spex/spex.yml` in place: add `project_name`'s `description` directly below it, and each repository's `description` directly below its `repository_name`. Preserve all existing fields, ordering, and formatting.
3. Validate the result against `<workspace_root>/spex/schemas/spex.schema.json`.

## Step 5 — Point to the next skill

Once `spex.yml` is described, the workspace is ready. Prompt the user to author their first spec: run `spex-create-biz-spec` for a business rule, or `spex-create-tech-spec` for a cross-cutting technical rule.

## Field placement

Place `description` right after the name it describes, mirroring the schema example:

```yaml
project_name: pepperoni-pizza
description: Online ordering for a pizzeria.
repositories:
  - repository_name: api
    description: Backend service that handles orders.
    path: ../api
    tech-specs:
      - TECH-SECRET-HANDLING
```

## Quality bar

- Project has a one-line, business-level `description`.
- Every repository has a one-line `description` describing its role.
- Descriptions are grounded in the repositories' real contents, not guesses.
- No existing field, path, or tech-spec is changed or removed.
- `spex.yml` stays valid against `<workspace_root>/spex/schemas/spex.schema.json`.
