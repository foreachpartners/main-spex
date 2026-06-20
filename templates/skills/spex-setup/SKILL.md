---
name: spex-setup
description: Fill in the missing descriptions in the workspace config (spex.yaml), collect each repository's interface contracts into the spex repository's contracts/ folder, and fill the missing folder icons in the editor workspace file (<project-name>.code-workspace). Use when the user wants to set up, describe, or document their spex project — adds a short description for the whole project and for each workspace repository by inspecting their contents, copies any existing contract files (OpenAPI, proto, GraphQL SDL, AsyncAPI) into contracts/ with a repository prefix and records them in spex.yaml, and picks a fitting emoji icon for each workspace folder. Writes a schema-valid spex.yaml.
---

# Describe the Workspace (spex.yaml)

Add a short `description` for the project and for each workspace repository in `spex.yaml`. Inspect each repository to understand what it is, then write concise summaries. The result must stay valid against the workspace config schema.

While inspecting each repository, collect any interface contract it ships into the `spex` repository's `contracts/` folder and record it in `spex.yaml` (see Step 4b). After describing the repositories and collecting contracts, fill in the folder icons in the editor workspace file (see the last step).

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name.

| What                    | Path                                                  |
| ----------------------- | ----------------------------------------------------- |
| Spex repository         | `<workspace-root>/spex/`                              |
| Workspace config        | `<workspace-root>/spex/spex.yaml`                     |
| Contracts folder        | `<workspace-root>/spex/contracts/`                    |
| Workspace config schema | `<workspace-root>/spex/schemas/spex.schema.json`      |
| Editor workspace file   | `<workspace-root>/spex/<project_name>.code-workspace` |

The schema at `<workspace-root>/spex/schemas/spex.schema.json` is authoritative for field rules and contains a worked example. Read it before editing.

## When to use this skill

Use it when the user wants to set up, describe, or document their spex project — typically right after `spex init`, when descriptions are missing and contracts have not yet been collected. Do not use it to add repositories or change paths; it only fills in `description` fields, collects contracts, and sets folder icons.

## Step 1 — Read the config

Read `<workspace-root>/spex/spex.yaml`. Note the `project_name`, every repository's `repository_name` and `path`, and which `description` fields are already present.

Decide what to fill in:

- Always propose a project `description` if it is missing.
- Propose a `description` for each repository that is missing one.
- By default, leave existing descriptions untouched. Only rewrite one if the user asks.

## Step 2 — Inspect each repository

For each repository missing a description, look at its real contents (resolve `path` relative to `<workspace-root>/spex/`). Gather just enough to summarize what it is and does:

- Read its `README` if present.
- Check manifest/metadata files to identify the stack and purpose (for example `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`).
- Skim the top-level directory layout and entry points when the above are missing.
- Note any interface contract files it ships (OpenAPI, proto, GraphQL SDL, AsyncAPI). You collect these in Step 4b.

Infer the project `description` from `project_name` and the combined picture of its repositories. If the repositories don't make the project's purpose clear, ask the user one short question rather than guessing.

## Step 3 — Write the descriptions

Each `description` must be a single, plain sentence that states what the thing is and does:

- Project: what the product is and the problem it solves.
- Repository: its role within the project (for example backend service, web client, shared library, infrastructure).

Keep each description to one line, concrete, and free of filler. Match the imperative, no-hype style of the existing specs. Do not restate the name.

## Step 4 — Confirm, then write

1. Show the proposed descriptions to the user and confirm them.
2. Edit `<workspace-root>/spex/spex.yaml` in place: add `project_name`'s `description` directly below it, and each repository's `description` directly below its `repository_name`. Preserve all existing fields, ordering, and formatting.
3. Validate the result against `<workspace-root>/spex/schemas/spex.schema.json`.

## Step 4b — Collect contracts

Each repository's interface contract is the single source of truth for its machine-readable interface. spex keeps every contract in the `spex` repository's `contracts/` folder so one Spec ID can trace across the contract and its implementations. Copy each repository's existing contract into that folder and record it in `spex.yaml`.

For each repository, using the contract files you noted in Step 2:

1. Identify its interface contract files. Look for OpenAPI (`openapi.yaml`, `openapi.json`, `*.openapi.yaml`), proto (`*.proto`), GraphQL SDL (`schema.graphql`, `*.graphql`), and AsyncAPI (`asyncapi.yaml`). Skip repositories that ship no contract.
2. Copy each contract file into `<workspace-root>/spex/contracts/`, renaming it with the repository's name as a prefix: `<repository-name>.<original-filename>` (for example `api`'s `openapi.yaml` becomes `contracts/api.openapi.yaml`). Do not double-prefix a filename that already starts with the repository name.
3. Add the copy under that repository's `contracts` list in `spex.yaml`. Each entry needs a `path` relative to the spex directory (for example `contracts/api.openapi.yaml`) and a one-line `description` of what it covers (for example `REST OpenAPI spec for the orders API.`).
4. Leave the original contract file in its repository untouched; the copy under `contracts/` is what spex tracks.
5. Validate the result against `<workspace-root>/spex/schemas/spex.schema.json`.

Confirm the copied contracts and their descriptions with the user, the same way you confirm descriptions in Step 4.

Example `spex.yaml` repository entry after collecting a contract:

```yaml
- repository_name: api
  description: Backend service that handles orders.
  path: ../api
  contracts:
    - path: contracts/api.openapi.yaml
      description: REST OpenAPI spec for the orders API.
  tech_specs:
    - TECH-SECRET-HANDLING
```

## Step 5 — Fill in the workspace folder icons

`spex init` writes `<workspace-root>/spex/<project_name>.code-workspace` with a `folders` list. The `spex` folder already has its icon (🧰); every other folder starts without one. Give each remaining folder a single emoji icon that fits its role.

1. Read the `.code-workspace` file.
2. For each folder whose `name` has no leading emoji, prepend one emoji plus a space (for example `🦀 backend`, `🛍️ storefront`). Base the choice on the repository's stack and role, which you already learned in Step 2.
3. Leave the `spex` folder's icon and every `path` untouched. Change only the `name` fields.
4. Keep the file valid JSON-with-comments (it may keep its trailing comma after the last folder).

Example result:

```json
{
  "folders": [
    { "name": "📜 proto", "path": "../moodmart-proto" },
    { "name": "🦀 backend", "path": "../moodmart-backend" },
    { "name": "🛍️ storefront", "path": "../moodmart-storefront" },
    { "name": "📋 analysis", "path": "../moodmart-analysis" },
    { "name": "🧰 spex", "path": "." }
  ]
}
```

## Step 6 — Point to the next skill

Once `spex.yaml` is described and the workspace folders are iconified, the workspace is ready. Prompt the user to author their first spec in a new chat window to save context: run `spex-create-biz-spec` for a product capability or behavior, or `spex-create-tech-spec` for a cross-cutting technical convention. Do not offer to run it in the current chat.

## Field placement

Place `description` right after the name it describes, and the `contracts` list after `path`, mirroring the schema example:

```yaml
project_name: pepperoni-pizza
description: Online ordering for a pizzeria.
repositories:
  - repository_name: api
    description: Backend service that handles orders.
    path: ../api
    contracts:
      - path: contracts/api.openapi.yaml
        description: REST OpenAPI spec for the orders API.
    tech_specs:
      - TECH-SECRET-HANDLING
```

## Quality bar

- Project has a one-line, business-level `description`.
- Every repository has a one-line `description` describing its role.
- Descriptions are grounded in the repositories' real contents, not guesses.
- Every existing contract is copied into `contracts/` with a repository-prefixed name and recorded under its repository's `contracts` in `spex.yaml`, each with a one-line `description`.
- No existing field, path, or tech-spec is changed or removed.
- `spex.yaml` stays valid against `<workspace-root>/spex/schemas/spex.schema.json`.
