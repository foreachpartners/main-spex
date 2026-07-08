## Spex workspace

This repo (`spex`) is the workspace repository that holds the project's specs and config.

- `spex.yaml`: project description. Lists the workspace repositories, their paths, contracts, and which es-specs apply to each.
- `cs-specs/` and `es-specs/`: single source of truth for specs.
  - `cs-specs/`: capability specs (`CS-*`). A capability the system must provide, with acceptance criteria.
  - `es-specs/`: engineering specs (`ES-*`). A cross-cutting convention or rule that applies repo-wide, with acceptance criteria.
- `contracts/`: machine-readable interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) derived from capability specs.
- `plans/`: implementation plans derived from approved capability specs, at `plans/<spec-id>/<date>-<slug>.md`, with a status per step/task.

Each fact lives in exactly one of these files — reference it, never copy it elsewhere.

Each artifact needs the human's approval before the next is generated: spec before contract, contract before plan, plan before code and tests. Never skip a gate.

Mark a capability spec `active` only when its acceptance criteria are fully implemented and `spex check` plus its tests pass. Otherwise leave the unmet criteria `[ ]` so `spex coverage` flags them — never report a spec done when a required step was skipped.

Never invent an acceptance criterion. When the work needs a rule no spec states, add it with `spex-capability-spec` or `spex-engineering-spec`, get it approved, then implement.

## Principles

- **Clean Code**: Simple code is clean if it can be understood easily – by everyone on the team. Clean code can be read and enhanced by a developer other than its original author. With understandability comes readability, changeability, extensibility, and maintainability.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. Don't explain WHAT the code does — well-named identifiers do that. Don't reference the current task, fix, or callers ("used by X", "added for Y", "handles case from issue #123"). Default to writing no comments.

## Rules

- Never delete, skip, or loosen a failing test to make a run green — debug it and fix it.
- Validate only at system boundaries (user input, external APIs). Trust internal code and framework guarantees; don't add defensive checks for scenarios that can't happen.
- Change only what the task requires. Don't refactor, rename, or reformat adjacent code that isn't part of it.
- Never use a destructive command (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.
