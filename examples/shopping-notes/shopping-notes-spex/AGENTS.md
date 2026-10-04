# spex

This project is governed by `spex`. The AI-first development framework that turns natural specifications into deterministic code — keeping AI running at full speed, with humans safely in the driver’s seat. AI operates fast — the framework's job is unleash it speed while ensuring slop-free verifiable results.

## Artifact chain

BRD (why, optional) -> PRD (what, optional) -> spec (capability) -> contract (interface like OpenAPI schema for data or Figma mockups for UI, if applicable) -> implementation (code) -> test (proof).

- **Spec** — one coherent capability the system must provide, with acceptance criteria proven by `spex coverage`. Specs are single source of truth for project - all behaviour, contracts, code, tests and other artifacts come from it and must adhere to it.
- **Rule** — one coherent cross-cutting convention (style, structure, security), with acceptance criteria proven by `spex check`. Each repository must adhere to a list of rules applicable to it defined in `spex.yaml`
- **Skill** — the instructions an AI agent follows to do the job consistently and correctly.
- **Traceability** — the `@spec` annotations stitching together spec, contracts, implementations, and tests, used to ensure coverage and to look up any of spec, contract, implementation, or test artifacts.

## Traceability

Every behavioral change traces to a spec acceptance creterion via a trace comment `@spec impl|test|contract <spec-id>#<ac-id>` e.g `@spec impl place-pizza-order#auth-required` for implementation acceptance criterion.

For `@spec test` and `@spec impl` ac-id for each acceptance criterion is required. Contract type is defined in spec, e.g `contract: for-each-ac`:

- `for-each-ac` - contract for each ac is requred, same as for `@spec impl` and `@spec test`/
- `single` - one contract required for entire spec, traced by `spec-id` only, e.g. `@spec impl place-pizza-order`.
- `none` - no contract required.

Trace comment should be placed on the smallest unit that actually satisfies the AC — a specific field, function, or test case — not on the file, the test suite's top-level describe, or the enclosing type. Widen the placement only when the AC's scope genuinely spans that boundary (e.g. an endpoint, contract). Target minimal tracing amount that make sense.

## Definition of done

A change isn't finished until:

- `spex check` passes (no error-severity findings) for every affected repository.
- `spex coverage` shows no gap for any spec acceptance criterion the change touches.
- Every acceptance criterion touched carries `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>` tracing annotations, e.g. `@spec impl place-pizza-order#auth-required` and `@spec test place-pizza-order#auth-required`.

## Workspace layout

The workspace root holds this repo (the single source of truth for specs, rules, contracts, and plans) alongside the other workspace repositories. Check `spex.yaml` for all project configurarition.

Skills live under `.agents/skills`; `.claude` is a symlink to `.agents`, so `.claude/skills` is the same directory — edit skill files under `.agents/skills` only.

## CLI commands

Run from anywhere inside the workspace; `spex <command> --help` lists every option.

- `spex check [--repo <name>...] [--files <path>...]` — run the rule checks; fails on error findings.
- `spex coverage [--scope <repository-short-name>...] [-v]` — report `@spec` annotation coverage per acceptance criterion.
- `spex specs [--id <spec-id>...] [--ai] [--output <file>]` — print specs as YAML.
- `spex rules [--id <rule-id>...] [--repo <name>...] [--ai] [--output <file>]` — print the active rules per repository as YAML.
- `spex info [--print <field>]` — print `spex.yaml`: project and per-repository configuration.
- `spex check-schema --schema <schema> --files <path>...` — validate JSON or YAML files against a JSON Schema.

Pass `--ai` to `specs` and `rules`: it drops fields AI doesn't need and adds ones it does. Prefer `--output <file>` for them too: long output gets truncated in a terminal. `spex init` and `spex update` run through the `spex-setup` and `spex-update` skills.

## Docs

- [docs/glossary.md](docs/glossary.md) — terms used across specs, rules, schemas, and skills.

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

Tag every spec-related commit with its ID(s) using @spec: `@spec <Spec ID>[#<AC ID>][, <Spec ID>[#<AC ID>]]`.

Examples:

```
style: format md files
feat(ui): list available pizzas @spec list-pizza-menu
fix(cart): reject negative quantities @spec cart-add-item, cart-validate
feat(orders): set status on order creation @spec place-pizza-order#created-order-status
```

---

# Project

Everything above `Project` is overrided on `spex update`, any project spicific data should be added here
