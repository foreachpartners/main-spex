---
name: spex-setup
description: Fill in the missing descriptions in the workspace config (spex.yaml), add each repository's applicable rs-specs, and record each repository's existing interface contracts, if any, in spex.yaml. Use this skill to complete spex setup after running `spex init`.
---

# Describe the Workspace (spex.yaml)

Add a short `description` for the project and for each workspace repository in `spex.yaml`. Inspect each repository to understand what it is, then write concise summaries. The result must stay valid against the workspace config schema.

`spex init` copies every bundled rs-spec into the spex repository's `rs-specs/` folder but leaves `rule_spec_ids` empty in `spex.yaml` for every repository. While inspecting each repository, decide which of those bundled rs-specs actually apply to it and add them (see Step 4c). Also ask the user about any interface contract it ships and record its location in `spex.yaml` (see Step 4b). `spex init` also seeds a dummy `SKILL.md` (name and description only) for each repository; fill in its body with the repository's real commands (see Step 5b). After describing the repositories, adding rs-specs, recording contracts, and filling in each repository's skill, fill in the folder icons in the editor workspace file (see the last step).

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name.

| What                    | Path                                                                 |
| ----------------------- | -------------------------------------------------------------------- |
| Spex repository         | `<workspace-root>/<project-name>-spex/`                              |
| Workspace config        | `<workspace-root>/<project-name>-spex/spex.yaml`                     |
| Contracts folder        | `<workspace-root>/<project-name>-spex/contracts/`                    |
| Workspace config schema | `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`      |
| Editor workspace file   | `<workspace-root>/<project-name>-spex/<project_name>.code-workspace` |

The schema at `<workspace-root>/<project-name>-spex/schemas/spex.schema.json` is authoritative for field rules and contains a worked example. Read it before editing.

## Step 1 — Read the config

Read `<workspace-root>/<project-name>-spex/spex.yaml`. Note the `project_name`, every repository's `repository_name` (full name), `repository_short_name`, and `path`, and which `description` fields are already present.

Decide what to fill in:

- Always propose a project `description` if it is missing.
- Propose a `description` for each repository that is missing one.
- By default, leave existing descriptions untouched. Only rewrite one if the user asks.

## Step 2 — Inspect each repository

For each repository missing a description, look at its real contents (resolve `path` relative to `<workspace-root>/<project-name>-spex/`). Gather just enough to summarize what it is and does:

- Read its `README` if present.
- Check manifest/metadata files to identify the stack and purpose (for example `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`).
- Skim the top-level directory layout and entry points when the above are missing.
- Note any interface contract files it ships (OpenAPI, proto, GraphQL SDL, AsyncAPI). You collect these in Step 4b.
- For each rs-spec file in the spex repository's `rs-specs/` folder, read it and judge whether it genuinely applies to this repository given its stack and role (for example `RS-OPENAPI-CONVENTIONS` applies only to a repository that exposes a REST API; `RS-GRAPHQL-CONVENTIONS` only to one that exposes GraphQL). You add the ones that apply in Step 4c.

Infer the project `description` from `project_name` and the combined picture of its repositories. If the repositories don't make the project's purpose clear, ask the user one short question rather than guessing.

## Step 3 — Write the descriptions

Each `description` must be a single, plain sentence that states what the thing is and does:

- Project: what the product is and the problem it solves.
- Repository: its role within the project (for example backend service, web client, shared library, infrastructure).

Keep each description to one line, concrete, and free of filler. Match the imperative, no-hype style of the existing specs. Do not restate the name.

## Step 4 — Confirm, then write

1. Show the proposed descriptions to the user and confirm them.
2. Edit `<workspace-root>/<project-name>-spex/spex.yaml` in place: add `project_name`'s `description` directly below it, and each repository's `description` directly below its `repository_name`. Preserve all existing fields, ordering, and formatting.
3. Validate the result against `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`.

## Step 4b — Record contracts

Each repository's interface contract, if it has one, is the single source of truth for its machine-readable interface. The user owns where a contract file actually lives — most commonly inside the owning repository itself. The `spex` repository's own `contracts/` folder is only a convention for contracts genuinely shared across repositories, never a default to copy into. Do not move or copy any contract file without the user explicitly deciding to. Record each repository's contract in `spex.yaml` regardless of where the file lives.

For each repository, using the contract files you noted in Step 2:

1. Identify its interface contract files. Look for OpenAPI (`openapi.yaml`, `openapi.json`, `*.openapi.yaml`), proto (`*.proto`), GraphQL SDL (`schema.graphql`, `*.graphql`), and AsyncAPI (`asyncapi.yaml`). Skip repositories that ship no contract.
2. Ask the user where each contract should live: in place inside its own repository (the default), or moved into the shared `<workspace-root>/<project-name>-spex/contracts/` folder because it's genuinely shared across repositories. Never decide this yourself.
3. Only if the user chooses the shared folder, move the file there, renaming it with the repository's short name as a prefix: `<repository-short-name>.<original-filename>` (for example the `api` repository's `openapi.yaml` becomes `contracts/api.openapi.yaml`). Do not double-prefix a filename that already starts with the repository short name.
4. Add an entry under that repository's `contracts` list in `spex.yaml`. Each entry needs a `path` relative to the spex directory (for example `../pepperoni-pizza-api/contracts/api.openapi.yaml` or `contracts/api.openapi.yaml`) and a one-line `description` of what it covers (for example `REST OpenAPI spec for the orders API.`).
5. Leave the original contract file in its repository untouched unless the user chose to move it in step 2.
6. Validate the result against `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`.

Confirm each repository's contract location and description with the user, the same way you confirm descriptions in Step 4.

Example `spex.yaml` repository entry after collecting a contract:

```yaml
- repository_name: pepperoni-pizza-api
  repository_short_name: api
  description: Backend service that handles orders.
  path: ../pepperoni-pizza-api
  contracts:
    - path: contracts/api.openapi.yaml
      description: REST OpenAPI spec for the orders API.
  rule_spec_ids:
    - RS-SECRET-HANDLING
```

## Step 4c — Add rs-specs that apply

`spex init` copies every bundled rs-spec into the spex repository's `rs-specs/` folder, but leaves `rule_spec_ids` empty in `spex.yaml` for every repository. Not every rs-spec applies to every repository — a stack- or interface-specific one (for example `RS-OPENAPI-CONVENTIONS`, `RS-GRAPHQL-CONVENTIONS`) only makes sense for a repository that actually has that interface, and some cross-cutting ones may not fit a given repository's role either.

1. For each repository, using the judgment from Step 2, list the rs-spec IDs whose file in `rs-specs/` genuinely applies to it.
2. Show the user, per repository, which rs-specs you propose to add and a one-line reason each (for example "RS-OPENAPI-CONVENTIONS: this repository exposes a REST API").
3. Only after the user confirms a given rs-spec applies, add its ID to that repository's `rule_spec_ids` list in `spex.yaml`. Leave out any rs-spec the user is unsure about or doesn't want.
4. Do not modify any rs-spec's file under `rs-specs/` while doing this; this only edits which repositories reference it in `spex.yaml`.
5. Validate the result against `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`.

### Remove rs-specs that apply to no repository

Once every repository's `rule_spec_ids` is settled, an rs-spec in `rs-specs/` that ended up referenced by no repository at all is dead weight in this workspace.

1. Compute the set of rs-spec IDs referenced by at least one repository's `rule_spec_ids`. Any bundled rs-spec in `rs-specs/` outside that set is unused workspace-wide.
2. Show the user the unused rs-spec IDs and confirm they should be removed. This is a deletion — no rs-spec is removed without the user's explicit confirmation.
3. For each confirmed ID: delete its file from `rs-specs/` (and any scripts under `scripts/<spec-id>/` it owns), and add the ID to the top-level `rule_spec_ids_removed` list in `spex.yaml` (create the list if absent). This records the deletion as deliberate so `spex update` never redeploys it.
4. Leave any rs-spec the user wants to keep untouched, even if unused today.
5. Validate the result against `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`.

## Step 5 — Fill in the workspace folder icons

`spex init` writes `<workspace-root>/<project-name>-spex/<project_name>.code-workspace` with a `folders` list. The `spex` folder already has its icon (🧰); every other folder starts without one. Give each remaining folder a single emoji icon that fits its role.

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

## Step 5b — Fill in each repository's skill file

`spex init` seeds `<workspace-root>/<project-name>-spex/.agents/skills/<repository-name>/SKILL.md` for every repository except the spex repository itself, with only its frontmatter (`name` and `description`) filled in. Give each an initial body from what you already learned inspecting the repository in Step 2, so an agent that loads the skill knows how to work in that repository without re-discovering the basics.

Fill in the body with four sections, in this order: `General`, `Paths`, `Commands`, `Rules`. Keep every section minimal — real facts pulled from the repository, never invented.

1. `# General` — one line pointing the agent at `spex info --repo <repository-short-name>` for repository metadata and applicable rule specs.
2. `# Paths` — a short bullet list of the directories/files an agent actually needs to know about to work in this repository (for example where routes, components, shared logic, tests, or config live). Pull these from the real directory layout, not a guess. Skip this section if the repository's layout is too flat or generic to be worth documenting.
3. `# Commands` — a one-line note that commands run from the repository root, then a bullet list of the commands an agent actually needs: dev server, running tests (and meaningful subsets, e.g. unit vs e2e), lint, format. Pull these from the repository's own `package.json` scripts, `Justfile`/`Makefile` targets, `README`, or CI config — do not invent commands that don't exist. If the repository has its own task runner (e.g. `just`), say so and prefer its commands over calling the underlying tool directly.
4. `# Rules` — a one-line note of which commands an agent must run after making changes (e.g. tests, lint, format), and after which kind of change (e.g. always vs. only for major changes). Pull this from CI config or existing contributor docs; if none exists, omit this section rather than inventing a policy.

See [`references/REPO-SKILL-TEMPLATE.md`](references/REPO-SKILL-TEMPLATE.md) for an example of the resulting body shape.

## Step 6 — Point to the next skill

Once `spex.yaml` is described and the workspace folders are iconified, the workspace is ready. Offer to author their first spec: run `spex-functional-spec` for a capability the system must provide, or `spex-rule-spec` for a cross-cutting convention or rule. Note that it is better to start a new chat first to save context.

## Field placement

Place `description` right after the name it describes, and the `contracts` list after `path`, mirroring the schema example:

```yaml
project_name: pepperoni-pizza
description: Online ordering for a pizzeria.
repositories:
  - repository_name: pepperoni-pizza-api
    repository_short_name: api
    description: Backend service that handles orders.
    path: ../pepperoni-pizza-api
    contracts:
      - path: contracts/api.openapi.yaml
        description: REST OpenAPI spec for the orders API.
    rule_spec_ids:
      - RS-SECRET-HANDLING
```

## Quality bar

- Project has a one-line, business-level `description`.
- Every repository has a one-line `description` describing its role.
- Descriptions are grounded in the repositories' real contents, not guesses.
- Every existing contract's location was confirmed with the user (in place by default, or moved into the shared `contracts/` folder only by their choice) and recorded under its repository's `contracts` in `spex.yaml`, each with a one-line `description`.
- Each repository's `rule_spec_ids` lists exactly the rs-specs that genuinely apply to it; every addition was confirmed by the user with a reason.
- Any rs-spec that applies to no repository was either removed (its file deleted from `rs-specs/` and its ID added to `rule_spec_ids_removed`) or deliberately kept, per the user's confirmation.
- Every repository's `SKILL.md` (other than the spex repository's) has a body with `General`, `Paths` (if applicable), `Commands`, and `Rules` sections grounded in that repository's real layout and commands, confirmed by the user.
- No existing field or path is changed, and no rs-spec is added or removed without user confirmation.
- `spex.yaml` stays valid against `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`.
