## Spex workspace

This repo (`spex`) is the workspace repository that holds the project's specs and config.

- `spex.yaml`: project description. Lists the workspace repositories, their paths, contracts, and which es-specs apply to each.
- `cs-specs/` and `es-specs/`: single source of truth for project behavior and conventions.
- `cs-specs/`: capability specs (`CS-*`). A capability the system must provide, with acceptance criteria.
- `es-specs/`: engineering specs (`ES-*`). A cross-cutting convention or rule that applies repo-wide, with acceptance criteria.
- `contracts/`: machine-readable interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) derived from capability specs.
- `plans/`: implementation plans derived from approved capability specs that serves as unchangeable history artifact after implementation is done.

Each fact lives in exactly one of these files — reference it, never copy it elsewhere.

Each artifact needs the human's approval before the next is generated: capability spec → contract (interface, if applicable) → plan (how to build it, optional) → implementation (code) → test (proof).

## Principles

Code should be simple enough that any teammate or AI — not just its author — can read, run, and safely change it.

- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. Don't explain WHAT the code does — well-named identifiers do that. Default to writing no comments.

## Rules

- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.
