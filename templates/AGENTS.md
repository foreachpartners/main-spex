# Spex workspace

This repo (`spex`) is the workspace repository that holds the project's specs and config.

- `spex.yaml`: project description. Lists the workspace repositories, their paths, contracts, and which rs-specs apply to each.
- `fs-specs/` and `rs-specs/`: single source of truth for project behavior and conventions.
- `fs-specs/`: functional specs (`FS-*`). A capability the system must provide, with acceptance criteria.
- `rs-specs/`: rule specs (`RS-*`). A cross-cutting convention or rule that applies repo-wide, with acceptance criteria.
- `contracts/`: machine-readable interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) derived from functional specs.
- `plans/`: implementation plans derived from approved functional specs that serves as unchangeable history artifact after implementation is done.

Each fact lives in exactly one of these files — reference it, never copy it elsewhere.

Each artifact needs the human's approval before the next is generated: functional spec → contract (interface, if applicable) → plan (how to build it, optional) → implementation (code) → test (proof).

## Tools

- `spex check` — Runs the rule spec checks declared in `spex.yaml` against the workspace. Use `--repo <name>` to limit to specific repositories, `--files <path>...` to target specific files, and `--fill-preexisting-errors` to baseline current errors.
- `spex coverage` — Reports how well the active functional specs are covered by spec-id annotations (`@spec impl`, `@spec test`, `@spec contract`) across the workspace, and tracks acceptance-criteria implementation progress.

# Rules

Code should be simple enough that any teammate or AI — not just its author — can read, run, and safely change it.

- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables instead of comments. Never add a comment — not for WHAT the code does, not for WHY it exists, not to note an extraction, refactor, or design rationale. Write no comments at all unless the user explicitly asks for one in that exact request.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.
