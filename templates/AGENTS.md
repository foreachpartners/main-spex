## Spex workspace

This repo (`spex`) is the workspace repository that holds the project's specs and config.

- `spex.yaml`: project description. Lists the workspace repositories, their paths, contracts, and which es-specs apply to each.
- `cs-specs/` and `es-specs/`: single source of truth for specs.
  - `cs-specs/`: capability specs (`CS-*`). A capability the system must provide, with acceptance criteria.
  - `es-specs/`: engineering specs (`ES-*`). A cross-cutting convention or rule that applies repo-wide, with acceptance criteria.
- `contracts/`: machine-readable interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) derived from capability specs.
- `plans/`: implementation plans derived from approved capability specs, at `plans/<spec-id>/<date>-<slug>.md`, with a status per step/task.

## Principles

- **Clean Code**: Simple code is clean if it can be understood easily – by everyone on the team. Clean code can be read and enhanced by a developer other than its original author. With understandability comes readability, changeability, extensibility, and maintainability.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. Don't explain WHAT the code does — well-named identifiers do that. Don't reference the current task, fix, or callers ("used by X", "added for Y", "handles case from issue #123"). Default to writing no comments.
