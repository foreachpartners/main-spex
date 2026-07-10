---
name: spex-contract
description: Derive or update the machine-readable interface contract from one or more capability specs. Use when the user wants to author, draft, update, or amend the contract (OpenAPI, proto, GraphQL SDL, AsyncAPI) for a CS spec — named, or the one just created. Reads the spec, the affected repositories, and the contract conventions that govern them, then writes the endpoints, messages, or schemas into the spex repository's contracts/ folder, annotates each with its spec, and records them in spex.yaml.
---

# Create an Interface Contract

A contract is the machine-readable _interface_ derived from a capability spec: endpoints, methods, status codes, errors, auth, and the request/response shapes — OpenAPI for HTTP APIs, `.proto` for gRPC, GraphQL SDL, AsyncAPI for events. AI implements from the spec and its contract, so the contract must cover every acceptance criterion that crosses a repository boundary. Your job is to read the approved specs and the repositories they touch, then write or update one contract per affected interface, annotated back to its spec.

Match the format and prose style of the existing contracts and write the result into the `spex` repository's `<workspace-root>/<project-name>-spex/contracts/` folder.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the spec's `id`.

Each repository's contracts live in the `spex` repository's `<workspace-root>/<project-name>-spex/contracts/` folder, named with the repository's prefix (for example `contracts/api.openapi.yaml`), and are declared per repository under `contracts` in `<workspace-root>/<project-name>-spex/spex.yaml`.

## When to use this skill

Use it when the user wants to create, draft, or update the contract for one or more CS specs — either named (for example `CS-PLACE-PIZZA-ORDER`) or the spec just created in this session. Do not use it to author a spec (use `spex-capability-spec`) or to plan implementation (use `spex-plan`). The spec must exist and be approved before its contract is drafted. An RS spec has no contract; do not use this skill for one.

## Step 0 — Establish the target specs

Determine which specs to contract before doing anything else:

- If the user named one or more spec IDs, contract those.
- If the user said "the one I just made" or gave no ID but a spec was created earlier in this session, contract that one and confirm the ID.
- If no spec is identified, ask which to contract. Do not invent a spec.

Before continuing, confirm each target spec's file exists at `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml`. If a spec does not exist yet, stop and create it first with `spex-capability-spec`, then return here. Do not contract a spec that has not been written and approved.

## Step 1 — Read the specs

For each targeted spec, read `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml` in full. Capture its `title`, `description`, every `acceptance_criteria`, and its `scope` (the in-scope repositories). The acceptance criteria are the rules the contract must let the implementation satisfy; identify which ones cross a repository boundary and therefore need an interface.

## Step 2 — Resolve scope, existing contracts, and conventions

From `<workspace-root>/<project-name>-spex/spex.yaml` (repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role):

- Collect the union of every targeted spec's `scope` — these are the affected repositories.
- For each affected repository, read its `contracts` list and open every contract file already in `<workspace-root>/<project-name>-spex/contracts/` so you extend the real document instead of starting over. Note its format (OpenAPI, proto, GraphQL SDL, AsyncAPI).
- For each affected repository, read its `rs_specs` list and read the governing contract conventions in `<workspace-root>/<project-name>-spex/rs-specs/` — at minimum `RS-OPENAPI-CONVENTIONS` (for OpenAPI), `RS-GRAPHQL-CONVENTIONS` (for GraphQL SDL), and `RS-SPEC-ID-ANNOTATIONS` — so the contract respects naming, casing, error responses, secret handling, and annotation rules.

## Step 3 — Decide what the contract needs

Map each acceptance criterion that crosses a boundary to the interface element that exposes it: an endpoint, a message, a schema field, an error, an auth requirement.

- If an affected repository exposes no machine-readable interface (for example a frontend that only consumes an existing API), it needs no new contract. State that and move on.
- If no targeted spec changes any contract, say so plainly and skip to Step 6 — there is nothing to write.
- Reuse existing operations and schemas where the spec extends them; add new ones only where the behavior is genuinely new.

## Step 4 — Draft the contract changes

Draft each contract following the format of the existing document and the governing conventions. For OpenAPI, apply `RS-OPENAPI-CONVENTIONS`; for GraphQL SDL, apply `RS-GRAPHQL-CONVENTIONS`:

- Declare every endpoint with a unique `operationId`, a `summary`, and at least one response, including explicit error responses for non-trivial endpoints.
- Define request and response shapes in `components/schemas` (or the format's equivalent) and reference them with `$ref`; never inline.
- Annotate each operation with its spec per `RS-SPEC-ID-ANNOTATIONS` — the OpenAPI `x-spec: <spec-id>` property, or a `@spec contract <spec-id>` comment in formats without an extension mechanism. Annotate once at the boundary (the operation), never on the nested schemas it references.
- Follow the format's casing, secret-handling, and discriminator rules from the governing conventions.

Keep the change minimal and consistent with the surrounding document.

The contract is the consumer-facing interface and may be published, served, or turned into client SDKs. Describe only what a consumer observes and must satisfy: field formats, value ranges, required-ness, uniqueness, enum values, and error semantics. Never leak internal implementation or persistence details a consumer cannot observe and does not need — how a value is stored, hashed, encrypted, normalized, or which algorithm or datastore backs it. For example, write "8–128 characters" for a password field, not "stored only as an Argon2 hash"; those details belong in the spec and the implementation, not the contract.

## Step 5 — Write, then offer edits

For each affected contract:

1. Write it into `<workspace-root>/<project-name>-spex/contracts/`, naming a new file with its repository's prefix (for example `contracts/api.openapi.yaml`); never overwrite another repository's contract.
2. Record any new contract file under its repository's `contracts` in `<workspace-root>/<project-name>-spex/spex.yaml`, with a `path` and a `description`, matching the existing entries' shape. An already-recorded contract needs no new entry.
3. Validate `spex.yaml` against `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`.
4. Tell the user the contract change you wrote (the added or modified operations, messages, and schemas) and where, then ask whether they want to change or edit anything. Apply any requested edits in place.

## Step 6 — Point to the next skill

After the contract is written, offer to plan the implementation: run `spex-plan <spec-id>` with the spec's `id`. Note that it is better to start a new chat first to save context.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
