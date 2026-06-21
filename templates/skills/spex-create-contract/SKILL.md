---
name: spex-create-contract
description: Derive the machine-readable interface contract from one or more business specs. Use when the user wants to author, draft, or update the contract (OpenAPI, proto, GraphQL SDL, AsyncAPI) for a BIZ spec — named, or the one just created. Reads the spec, the affected repositories, and the contract conventions that govern them, then writes the endpoints, messages, or schemas into the spex repository's contracts/ folder, annotates each with its spec, and records them in spex.yaml.
---

# Create an Interface Contract

A contract is the machine-readable _interface_ derived from a business spec: endpoints, methods, status codes, errors, auth, and the request/response shapes — OpenAPI for HTTP APIs, `.proto` for gRPC, GraphQL SDL, AsyncAPI for events. AI implements from the spec and its contract, so the contract must cover every acceptance criterion that crosses a repository boundary. Your job is to read the approved specs and the repositories they touch, then write or update one contract per affected interface, annotated back to its spec.

Match the format and prose style of the existing contracts and write the result into the `spex` repository's `<workspace-root>/spex/contracts/` folder.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the spec's `id`.

Each repository's contracts live in the `spex` repository's `<workspace-root>/spex/contracts/` folder, named with the repository's prefix (for example `contracts/api.openapi.yaml`), and are declared per repository under `contracts` in `<workspace-root>/spex/spex.yaml`.

## When to use this skill

Use it when the user wants to create, draft, or update the contract for one or more BIZ specs — either named (for example `BIZ-PLACE-PIZZA-ORDER`) or the spec just created in this session. Do not use it to author a spec (use `spex-create-biz-spec`) or to plan implementation (use `spex-create-plan`). The spec must exist and be approved before its contract is drafted. A TECH spec has no contract; do not use this skill for one.

## Step 0 — Establish the target specs

Determine which specs to contract before doing anything else:

- If the user named one or more spec IDs, contract those.
- If the user said "the one I just made" or gave no ID but a spec was created earlier in this session, contract that one and confirm the ID.
- If no spec is identified, ask which to contract. Do not invent a spec.

Before continuing, confirm each target spec's file exists at `<workspace-root>/spex/biz-specs/<spec-id>.yaml`. If a spec does not exist yet, stop and create it first with `spex-create-biz-spec`, then return here. Do not contract a spec that has not been written and approved.

## Step 1 — Read the specs

For each targeted spec, read `<workspace-root>/spex/biz-specs/<spec-id>.yaml` in full. Capture its `title`, `description`, every `acceptance_criteria`, and its `scope` (the in-scope repositories). The acceptance criteria are the rules the contract must let the implementation satisfy; identify which ones cross a repository boundary and therefore need an interface.

## Step 2 — Resolve scope, existing contracts, and conventions

From `<workspace-root>/spex/spex.yaml` (repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role):

- Collect the union of every targeted spec's `scope` — these are the affected repositories.
- For each affected repository, read its `contracts` list and open every contract file already in `<workspace-root>/spex/contracts/` so you extend the real document instead of starting over. Note its format (OpenAPI, proto, GraphQL SDL, AsyncAPI).
- For each affected repository, read its `tech_specs` list and read the governing contract conventions in `<workspace-root>/spex/tech-specs/` — at minimum `TECH-OPENAPI-CONVENTIONS` (for OpenAPI) and `TECH-SPEC-ID-ANNOTATIONS` — so the contract respects naming, casing, error responses, secret handling, and annotation rules.

## Step 3 — Decide what the contract needs

Map each acceptance criterion that crosses a boundary to the interface element that exposes it: an endpoint, a message, a schema field, an error, an auth requirement.

- If an affected repository exposes no machine-readable interface (for example a frontend that only consumes an existing API), it needs no new contract. State that and move on.
- If no targeted spec changes any contract, say so plainly and skip to Step 6 — there is nothing to write.
- Reuse existing operations and schemas where the spec extends them; add new ones only where the behavior is genuinely new.

## Step 4 — Draft the contract changes

Draft each contract following the format of the existing document and the governing conventions. For OpenAPI, apply `TECH-OPENAPI-CONVENTIONS`:

- Declare every endpoint with a unique `operationId`, a `summary`, and at least one response, including explicit error responses for non-trivial endpoints.
- Define request and response shapes in `components/schemas` (or the format's equivalent) and reference them with `$ref`; never inline.
- Annotate each operation with its spec per `TECH-SPEC-ID-ANNOTATIONS` — the OpenAPI `x-spec: <spec-id>` property, or a `@spec contract <spec-id>` comment in formats without an extension mechanism. Annotate once at the boundary (the operation), never on the nested schemas it references.
- Follow the format's casing, secret-handling, and discriminator rules from the governing conventions.

Keep the change minimal and consistent with the surrounding document.

## Step 5 — Confirm, then write

For each affected contract:

1. Show the drafted contract change (the added or modified operations, messages, and schemas) to the user for confirmation. Apply edits if needed.
2. Write it into `<workspace-root>/spex/contracts/`, naming a new file with its repository's prefix (for example `contracts/api.openapi.yaml`); never overwrite another repository's contract.
3. Record any new contract file under its repository's `contracts` in `<workspace-root>/spex/spex.yaml`, with a `path` and a `description`, matching the existing entries' shape. An already-recorded contract needs no new entry.
4. Validate `spex.yaml` against `<workspace-root>/spex/schemas/spex.schema.json`.

## Step 6 — Point to the next skill

After the contract is written, offer to plan the implementation: run `spex-create-plan <spec-id>` with the spec's `id`. Note that it is better to start a new chat first to save context.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
