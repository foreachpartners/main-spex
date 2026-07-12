## Spex workspace

This repo (`spex`) is the workspace repository that holds the project's specs and config.

- `spex.yaml`: project description. Lists the workspace repositories, their paths, contracts, and which rs-specs apply to each.
- `fs-specs/` and `rs-specs/`: single source of truth for specs.
  - `fs-specs/`: functional specs (`FS-*`). A capability the system must provide, with acceptance criteria.
  - `rs-specs/`: rule specs (`RS-*`). A cross-cutting convention or rule that applies repo-wide, with acceptance criteria.
- `contracts/`: machine-readable interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) derived from functional specs.
- `plans/`: implementation plans derived from approved functional specs, at `plans/<spec-id>/<date>-<slug>.md`, with a status per step/task.

## Principles

- **Clean Code**: Simple code is clean if it can be understood easily – by everyone on the team. Clean code can be read and enhanced by a developer other than its original author. With understandability comes readability, changeability, extensibility, and maintainability.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. This project is not public-facing, so do NOT write doc comments (`///`, `//!`) and do NOT write comments that restate what the code does. The only acceptable comment is a short `//` note explaining a non-obvious "why" (a constraint, footgun, or workaround) that names alone cannot convey, e.g. a hard-exit that avoids a runtime segfault. When in doubt, leave it out.
