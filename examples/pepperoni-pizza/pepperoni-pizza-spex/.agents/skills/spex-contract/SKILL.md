---
name: spex-contract
description: Derive or update the interface contract from one or more specs. Use when the user wants to create, extend, or update the contract (OpenAPI, proto, GraphQL SDL, AsyncAPI, or a design reference for UI-facing specs) for a spec.
---

# Create an Interface Contract

A contract is the _interface_ derived from a spec. For a machine-readable interface: endpoints, methods, status codes, errors, auth, and the request/response shapes — OpenAPI for HTTP APIs, `.proto` for gRPC, GraphQL SDL, AsyncAPI for events. For a UI-facing spec that exposes no machine-readable interface, it's a design reference instead: a markdown document linking each spec to its visual design (e.g. a Figma frame). AI implements from the spec and its contract, so the contract must cover every acceptance criterion that crosses a repository boundary, or every UI part a design reference should visually pin down. Your job is to read the approved specs and the repositories they touch, then write or update one contract per affected interface, annotated back to its spec.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the spec's `id`.

Each repository declares its contracts under `contracts` in `<workspace-root>/<project-name>-spex/spex.yaml`, each with a `path` relative to the spex directory. The user owns where a contract file actually lives — most commonly inside the owning repository itself (for example `../pepperoni-pizza-api/contracts/api.openapi.yaml`). The spex repository's own `<workspace-root>/<project-name>-spex/contracts/` folder is only a convention for contracts genuinely shared across repositories, not the default location.

## Step 0 — Establish the target specs

Determine which specs to contract before doing anything else:

- If the user named one or more spec IDs, contract those.
- If the user said "the one I just made" or gave no ID but a spec was created earlier in this session, contract that one and confirm the ID.
- If no spec is identified, ask which to contract. Do not invent a spec.

## Step 1 — Gather context first

Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the targeted specs. Capture each spec's `title`, `description`, every `acceptance_criteria`, and its `scope` (the in-scope repositories). The acceptance criteria are the rules the contract must let the implementation satisfy; identify which ones cross a repository boundary and therefore need an interface.

## Step 2 — Resolve scope, existing contracts, and conventions

From `<workspace-root>/<project-name>-spex/spex.yaml` (repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role):

- Collect the union of every targeted spec's `scope` — these are the affected repositories.
- For each affected repository, read its `contracts` list and open every contract file already recorded there, at its declared `path`, so you extend the real document instead of starting over. Note its format (OpenAPI, proto, GraphQL SDL, AsyncAPI, or design reference).
- For each affected repository, run `spex info --repo <repo>` to read its `rules` list, then run `spex rules --id <rule-id>... --ai` for the union of those IDs plus the governing contract conventions — at minimum `OPENAPI-CONVENTIONS` (for OpenAPI), `GRAPHQL-CONVENTIONS` (for GraphQL SDL), and `SPEC-ID-TRACING` — so the contract respects naming, casing, error responses, secret handling, and annotation rules.

## Step 3 — Decide what the contract needs

Map each acceptance criterion that crosses a boundary to the interface element that exposes it: an endpoint, a message, a schema field, an error, an auth requirement — or, for a UI-facing spec with no machine-readable interface, the visual design it must match.

- If an affected repository exposes no machine-readable interface and no UI needing a design reference (for example a backend job with no consumer-facing surface), it needs no new contract. State that and move on.
- If no targeted spec changes any contract, say so plainly and skip to Step 6 — there is nothing to write.
- Reuse existing operations and schemas where the spec extends them; add new ones only where the behavior is genuinely new.

## Step 4 — Draft the contract changes

Draft each contract following the format of the existing document and the governing conventions. For OpenAPI, apply `OPENAPI-CONVENTIONS`; for GraphQL SDL, apply `GRAPHQL-CONVENTIONS`.

For a machine-readable interface:

- Declare every endpoint with a unique `operationId`, a `summary`, and at least one response, including explicit error responses for non-trivial endpoints.
- Define request and response shapes in `components/schemas` (or the format's equivalent) and reference them with `$ref`; never inline.
- Annotate each operation with its spec per `SPEC-ID-TRACING` — the OpenAPI `x-spec: <spec-id>` property, or a `@spec contract <spec-id>` comment in formats without an extension mechanism. Annotate once at the boundary (the operation), never on the nested schemas it references.
- Follow the format's casing, secret-handling, and discriminator rules from the governing conventions.

For a design reference:

- Give each spec its own `##` section with a `<!-- @spec contract <spec-id> -->` comment directly above the heading.
- Link the specific design frame (e.g. a Figma node, not the whole file) and note what it covers in one or two sentences.
- Extend an existing section instead of duplicating it when a spec's design changes.

Keep the change minimal and consistent with the surrounding document.

The contract is the consumer-facing interface and may be published, served, or turned into client SDKs. Describe only what a consumer observes and must satisfy: field formats, value ranges, required-ness, uniqueness, enum values, and error semantics. Never leak internal implementation or persistence details a consumer cannot observe and does not need — how a value is stored, hashed, encrypted, normalized, or which algorithm or datastore backs it. For example, write "8–128 characters" for a password field, not "stored only as an Argon2 hash"; those details belong in the spec and the implementation, not the contract.

## Step 5 — Write, then offer edits

For each affected contract:

1. If the repository already has a matching entry under `contracts` in `spex.yaml`, write to its declared `path`.
2. Otherwise, this is a new contract file: ask the user where it should live, defaulting to inside the owning repository itself (for example `<workspace-root>/<repository-name>/contracts/api.openapi.yaml`, or `contracts/figma.md` for a design reference); offer `<workspace-root>/<project-name>-spex/contracts/` as the alternative for a contract meant to be shared across repositories. Write the file at whichever path the user confirms.
3. Record any new contract file under its repository's `contracts` in `<workspace-root>/<project-name>-spex/spex.yaml`, with a `path` (relative to the spex directory) and a `description`, matching the existing entries' shape. An already-recorded contract needs no new entry.
4. Validate `spex.yaml` against `<workspace-root>/<project-name>-spex/schemas/spex.schema.json`.
5. Tell the user the contract change you wrote (the added or modified operations, messages, and schemas) and where, then ask whether they want to change or edit anything. Apply any requested edits in place.

## Example

Acceptance criterion: "POST /orders rejects an empty `items` array with a 422." Resulting OpenAPI fragment in `contracts/api.openapi.yaml`:

```yaml
paths:
  /orders:
    post:
      operationId: createOrder
      summary: Create an order
      x-spec: ORDER-CREATE
      requestBody:
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/CreateOrderRequest"
      responses:
        "201":
          description: Order created
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Order"
        "422":
          description: Validation error
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ValidationError"
components:
  schemas:
    CreateOrderRequest:
      type: object
      required: [items]
      properties:
        items:
          type: array
          minItems: 1
          items:
            $ref: "#/components/schemas/OrderItem"
```

The `x-spec` annotation sits on the operation, not on `CreateOrderRequest` or `OrderItem`.

Design reference example, for a spec with no machine-readable interface, in `contracts/figma.md`:

```md
<!-- @spec contract DELIVERY-TIME -->

## Delivery time

https://figma.com/file/pepperoni-pizza/cart-and-delivery?node-id=42%3A100

Cart delivery estimate (live, updates as pizzas are added) and the
in-progress order countdown.
```

## Step 6 — Point to the next skill

After the contract is written, offer to plan the implementation: run `spex-plan <spec-id>` with the spec's `id`. Note that it is better to start a new chat first to save context.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
