# `spex-contract` skill

Derives or updates the interface contract from one or more specs: OpenAPI for HTTP APIs, `.proto` for gRPC, GraphQL SDL, AsyncAPI for events, or, for a UI-facing spec, a design reference linking each spec to its visual design. Loaded when the user wants to create, extend, or update the contract for a spec.

## Acceptance criteria

- MUST read the targeted specs (`spex specs --ai --id ...`) and the union of their `packages`, then write or update one contract per affected interface, annotated back to its spec, covering every acceptance criterion that crosses a package boundary, or every UI part a design reference must pin down.
- MUST NOT invent a spec: contract the ones the user named, one created earlier in the session (confirming the ID), or ask which to contract.
- MUST read each affected package's `contracts` in `spex.yaml`, opening every contract file already recorded at its declared `path`, and read the governing contract conventions plus each package's rules (`spex info --package`, `spex rules --id ... --ai`).
- MUST put a contract shared across packages in `<spex-dir>/contracts/` and one specific to a package in that package, defaulting a new contract inside its owning package and asking the user where it should live.
- MUST map each boundary-crossing acceptance criterion to the interface element that exposes it, reuse existing operations and schemas, and add new ones only where the behavior is genuinely new.
- MUST declare every endpoint with a unique `operationId`, a `summary`, and at least one response, including error responses for non-trivial endpoints, define request and response shapes in `components/schemas` referenced by `$ref`, and never inline them.
- MUST annotate each operation with `@spec contract <spec-id>` (or `#<ac-id>` under `contract: for-each-ac`), once at the boundary, never on nested schemas; for a design reference, give each spec its own `##` section with the annotation comment above it.
- MUST describe only what a consumer observes and must satisfy, never leaking internal implementation or persistence details.
- MUST record any new contract file under its package's `contracts` in `spex.yaml`, validate `spex.yaml` against `schemas/spex.schema.json`, and tell the user the change it wrote.
- MUST offer to plan the implementation with `spex-plan <spec-id>`.
