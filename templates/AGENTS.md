# spex

This project is governed by `spex`. The AI-first development framework that turns natural specifications into deterministic code — keeping AI running at full speed, with humans safely in the driver’s seat. AI operates fast and getting even faster — the framework's job is unleash it speed while ensuring slop-free results.

## Artifact chain

BRD (why, optional) -> PRD (what, optional) -> spec (capability) -> contract (interface, if applicable) -> plan (how to implement, optional) -> implementation (code) -> test (proof). Every behavioral change traces to a Spec ID, annotated at the layer boundary (`@spec contract|impl|test <spec-id>[#<ac-id>]`, or `x-spec: <spec-id>` in OpenAPI). Tag `impl` and `test` annotations with the specific acceptance criterion id are required for each so `spex coverage` can verify each one individually.

- **Spec** — one coherent capability the system must provide, with acceptance criteria proven by `spex coverage`.
- **Rule** — one coherent cross-cutting convention (style, structure, security) enforced repo-wide, with acceptance criteria proven by `spex check`.
- **Skill** — the instructions an AI agent follows to do the job, taking all applicable rules and implementing a spec.
- **Traceability** — the `@spec`/`@rule` annotations stitching together spec, contract, implementations, and tests, used to ensure coverage and to look up any of spec, contract, implementation, or test from any other, in either direction.

## Definition of done

A change isn't finished until:

- `spex check` passes (no error-severity findings) for every affected repository.
- `spex coverage` shows no gap for any spec or acceptance criterion the change touches.
- Every acceptance criterion touched carries `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>` tracing annotations, e.g. `@spec impl place-pizza-order#auth-required` and `@spec test place-pizza-order#auth-required`.

## Workspace layout

The workspace root holds this repo (the single source of truth for specs, rules, contracts, and plans) alongside the other workspace repositories. Check `spex.yaml` for all project configurarition.

Skills live under `.agents/skills`; `.claude` is a symlink to `.agents`, so `.claude/skills` is the same directory — edit skill files under `.agents/skills` only.

## Docs

- [docs/glossary.md](docs/glossary.md) — terms used across specs, rules, schemas, and skills.
- [docs/framework-principles.md](docs/framework-principles.md) — the non-negotiable principles spex is built on.
- [docs/cli.md](docs/cli.md) — full `spex` command reference.

## Core discipline

Code should be simple enough that any teammate or AI — not just its author — can read, run, and safely change it.

- **Human in control**: no artifact ships unapproved — spec before contract, contract before plan, plan before code and tests.
- **Single source of truth**: reference a spec, contract, or convention — never copy it into code comments, docs, or agent rules.
- **Traceability**: a behavioral change with no Spec ID annotation is a violation.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables instead of comments. Never add a comment — not for WHAT the code does, not for WHY it exists, not to note an extraction, refactor, or design rationale. Write no comments at all unless the user explicitly asks for one in that exact request.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Systemic Fix Over Quick Fix**: Never apply temporary workarounds, hacks, or band-aids to bypass a structural issue. If a proper, clean fix requires refactoring, architectural changes, or expanding the scope of the current task, stop immediately. Present the proper solution and ask the user for explicit permission to make that change before proceeding.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless triggered by the "Systemic Fix Over Quick Fix" rule.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/): `<type>[optional scope]: <description>`, with an optional body separated by a blank line. See `rules/commit-message-format.yaml`.

Tag every spec- or rule-related commit with its ID(s) using @spec or @rule: `@spec <Spec ID>[#<AC ID>][, <Spec ID>[#<AC ID>]]` or `@rule <Rule ID>[, <Rule ID>]`.

Examples:

```
style: format md files
feat(deps): pin dependencies @rule dependencies-rules
feat(ui): list available pizzas @spec list-pizza-menu
fix(cart): reject negative quantities @spec cart-add-item, cart-validate
feat(orders): set status on order creation @spec place-pizza-order#created-order-status
```

---

# Project

Everything above `Project` is overrided on `spex update`, any project spicific data should be added here
