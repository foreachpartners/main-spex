# General

This project is governed by `spex` AI-first specification-driven development framework with human in control: the agent writes every artifact — specs, contracts, plans, code, and tests — and the human approves each one before the next is generated.

## Artifact chain

BRD (why) → PRD (what) → functional spec (FS, capability) → contract (interface, if applicable) → plan (how, optional) → implementation (code) → test (proof). Every behavioral change traces to a Spec ID (`FS-*`/`RS-*`), annotated at the layer boundary (`@spec contract|impl|test <spec-id>`, or `x-spec: <spec-id>` in OpenAPI).

- **Functional spec (FS)** — one coherent capability the system must provide, with acceptance criteria.
- **Rule spec (RS)** — one coherent cross-cutting convention (style, structure, security) enforced repo-wide, with acceptance criteria proven by `spex check`.

## Workspace layout

The workspace root holds this repo (`<project-name>-spex`, the single source of truth for specs, contracts, and plans) alongside the other workspace repositories. Check `spex.yaml` for project for all configuration.

## Everyday commands

Run from anywhere in the workspace — `spex` locates `spex.yaml` automatically.

- `spex check` — runs rule-spec checks and gates on `error`-severity findings (`--repo <name>`, `--files <path>...` to scope).
- `spex coverage` — reports `@spec` annotation and acceptance-criteria coverage for active functional specs.
- `spex info` — prints the resolved config (repos, paths, contracts, rule specs).

After all changes across the workspace, run `spex check` (optionally scoped with `--repo`) and `spex coverage`.

## Core discipline

- **Human in control**: no artifact ships unapproved — spec before contract, contract before plan, plan before code and tests.
- **Single source of truth**: reference a spec, contract, or convention — never copy it into code comments, docs, or agent rules.
- **Traceability**: a behavioral change with no Spec ID annotation is a violation.

# Rules

Code should be simple enough that any teammate or AI — not just its author — can read, run, and safely change it.

- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables instead of comments. Never add a comment — not for WHAT the code does, not for WHY it exists, not to note an extraction, refactor, or design rationale. Write no comments at all unless the user explicitly asks for one in that exact request.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Systemic Fix Over Quick Fix**: Never apply temporary workarounds, hacks, or band-aids to bypass a structural issue. If a proper, clean fix requires refactoring, architectural changes, or expanding the scope of the current task, stop immediately. Present the proper solution and ask the user for explicit permission to make that change before proceeding.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless triggered by the "Systemic Fix Over Quick Fix" rule.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/): `<type>[optional scope]: <description>`, with an optional body separated by a blank line. See `rs-specs/RS-COMMIT-MESSAGE-FORMAT.yaml`.

- Use the `fs` scope for functional spec changes (e.g. `feat(fs)`, `fix(fs)`).
- Use the `rs` scope for rule spec changes (e.g. `feat(rs)`, `fix(rs)`).
- End every `fs`-scoped description with at least one comma-separated Spec ID: `<Spec ID>[, <Spec ID>]`.

Examples:

```
style: format md files
feat(rs): pin dependencies as required by RS-DEPENDENCIRS-RULES
feat(fs): list available pizzas FS-LIST-PIZZA-MENU
fix(fs): reject negative quantities FS-CART-ADD-ITEM, FS-CART-VALIDATE
```
