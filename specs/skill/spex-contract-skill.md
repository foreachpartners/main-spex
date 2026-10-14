# `spex-contract` skill

Derives or updates the interface contract from one or more specs: OpenAPI for HTTP APIs, `.proto` for gRPC, GraphQL SDL, AsyncAPI for events, or, for a UI-facing spec, a design reference, a markdown document linking each spec to its visual design (e.g. a Figma frame). AI implements from the spec and its contract. Loaded when the user wants to create, extend, or update the contract for a spec.

## Acceptance criteria

### Purpose and paths

- MUST read the approved specs and the packages they touch, then write or update one contract per affected interface, annotated back to its spec.
- MUST cover every acceptance criterion that crosses a package boundary, or every UI part a design reference should visually pin down.
- MUST read each package's contracts from `contracts` in `<spex-dir>/spex.yaml`, each with a `path` relative to the `spex.yaml` file where it is declared.
- MUST put a contract shared across packages (e.g. an OpenAPI used by api and frontend) in `<spex-dir>/contracts/`, and a contract specific to one package (e.g. a figma design reference) in the package it belongs to (e.g. `contracts/figma.md` inside the frontend package).

### Step 0: establish the target specs

- MUST contract the spec IDs the user named.
- MUST contract, when the user said "the one I just made" or gave no ID but a spec was created earlier in the session, that spec, and confirm the ID.
- MUST ask which spec to contract when none is identified.
- MUST NOT invent a spec.

### Step 1: gather context

- MUST run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the targeted specs, and capture each spec's `title`, `description`, every `acceptance_criteria`, and its `scope`.
- MUST identify which acceptance criteria cross a package boundary and therefore need an interface.

### Step 2: resolve scope, existing contracts, and conventions

- MUST collect the union of every targeted spec's `scope` as the affected packages, reading each package's `description` to learn its role, since package names are arbitrary.
- MUST read, for each affected package, its `contracts` list and open every contract file already recorded there, at its declared `path`, so it extends the real document instead of starting over, noting its format.
- MUST run `spex info --package <package-name>` for each affected package to read its `rules` list, then `spex rules --id <rule-id>... --ai` for the union of those IDs plus the governing contract conventions, at minimum `openapi-conventions` (for OpenAPI) or `graphql-conventions` (for GraphQL SDL), so the contract respects naming, casing, error responses, and secret handling.

### Step 3: decide what the contract needs

- MUST map each acceptance criterion that crosses a boundary to the interface element that exposes it: an endpoint, a message, a schema field, an error, an auth requirement, or, for a UI-facing spec, the visual design it must match.
- MUST state that an affected package needs no new contract, and move on, when it exposes no API and no UI needing a design reference (e.g. a backend job with no consumer-facing surface).
- MUST say so plainly and skip to Step 6 when no targeted spec changes any contract.
- MUST reuse existing operations and schemas where the spec extends them, and add new ones only where the behavior is genuinely new.

### Step 4: draft the contract changes

- MUST draft each contract following the format of the existing document and the governing conventions: `openapi-conventions` for OpenAPI, `graphql-conventions` for GraphQL SDL.
- MUST declare, for an API, every endpoint with a unique `operationId`, a `summary`, and at least one response, including explicit error responses for non-trivial endpoints.
- MUST define request and response shapes in `components/schemas` (or the format's equivalent) and reference them with `$ref`.
- MUST annotate each operation with its spec: an OpenAPI `x-*` vendor extension (e.g. `x-spec`) whose value is `@spec contract <spec-id>`, or that same comment form in formats without an extension mechanism; the extension's key name is not itself checked, only that its value matches the grammar.
- MUST annotate once at the boundary, the operation, never on the nested schemas it references.
- MUST anchor the annotation, for a spec with `contract: for-each-ac`, to the acceptance criterion it satisfies, `@spec contract <spec-id>#<ac-id>`, one per criterion; under `contract: single` a single bare `@spec contract <spec-id>` covers the whole spec.
- MUST follow the format's casing, secret-handling, and discriminator rules from the governing conventions.
- MUST give, for a design reference, each spec its own `##` section with a `<!-- @spec contract <spec-id> -->` comment directly above the heading (under `contract: for-each-ac`, one comment per acceptance criterion the section covers, as `<spec-id>#<ac-id>`).
- MUST link, in a design reference, the specific design frame (e.g. a Figma node, not the whole file) and note what it covers in one or two sentences.
- MUST extend an existing design reference section instead of duplicating it when a spec's design changes.
- MUST keep the change minimal and consistent with the surrounding document.
- MUST describe only what a consumer observes and must satisfy: field formats, value ranges, required-ness, uniqueness, enum values, and error semantics, since the contract is the consumer-facing interface and may be published, served, or turned into client SDKs.
- MUST NOT inline request and response shapes.
- MUST NOT leak internal implementation or persistence details a consumer cannot observe and does not need: how a value is stored, hashed, encrypted, normalized, or which algorithm or datastore backs it (e.g. write "8–128 characters" for a password field, not "stored only as an Argon2 hash").

### Step 5: write, then offer edits

- MUST write to the declared `path` when the package already has a matching entry under `contracts` in `spex.yaml`.
- MUST otherwise ask the user where a new contract file should live, defaulting to inside the owning package itself (e.g. `<package-path>/contracts/api.openapi.yaml`, with `<package-path>` from `spex info --package <package-name>`, or `contracts/figma.md` for a design reference), offering `<spex-dir>/contracts/` as the alternative for a contract meant to be shared across packages, and write the file at the path the user confirms.
- MUST record any new contract file under its package's `contracts` in `<spex-dir>/spex.yaml`, with a `path` (relative to the spex directory) and a `description`, matching the existing entries' shape; an already-recorded contract needs no new entry.
- MUST validate `spex.yaml` against `<spex-dir>/schemas/spex.schema.json`.
- MUST tell the user the contract change it wrote (the added or modified operations, messages, and schemas) and where, ask whether they want to change anything, and apply any requested edits in place.

### Step 6: point to the next skill

- MUST offer to plan the implementation with `spex-plan <spec-id>` once the contract is written, noting that starting a new chat first saves context.
- MUST follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else, with no filler, hedging, motivational, or apologetic phrasing.
