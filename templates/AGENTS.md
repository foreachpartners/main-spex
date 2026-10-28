# spex

This project is governed by `spex`. The AI-first development framework that turns specifications into verified code — keeping AI running at full speed while ensuring slop-free results.

## Artifact chain

PRD (why and what) -> spec (capability) -> contract (interface like OpenAPI schema for data or Figma mockups for UI, if applicable) -> implementation (code) -> test (proof).

- **PRD** — the short project overview that comes first: what we are building and why. Kept high level, without details: those live in specs. Agents read it to write specs that fit the project's purpose.
- **Spec** — one coherent capability the system must provide, with acceptance criteria proven by `spex coverage`. Specs are single source of truth for project - all behaviour, contracts, code, tests and other artifacts come from it and must adhere to it.
- **Rule** — one coherent cross-cutting convention (style, structure, security), with acceptance criteria proven by `spex check`. Each package must adhere to a list of rules applicable to it defined in `spex.yaml`
- **Skill** — the instructions an AI agent follows to do the job consistently and correctly.
- **Traceability** — the `@spec` annotations stitching together spec, contracts, implementations, and tests, used to ensure coverage and to look up any of spec, contract, implementation, or test artifacts.

## Traceability

Every behavioral change traces to a spec acceptance criterion via a trace comment `@spec impl|test|contract <spec-id>#<ac-id>` e.g `@spec impl place-pizza-order#auth-required` for implementation acceptance criterion.

For `@spec test` and `@spec impl` ac-id for each acceptance criterion is required. Contract type is defined in spec, e.g `contract: for-each-ac`:

- `for-each-ac` - contract for each ac is required, same as for `@spec impl` and `@spec test`.
- `single` - one contract required for entire spec, traced by `spec-id` only, e.g. `@spec contract place-pizza-order`.
- `none` - no contract required.

Trace comment should be placed on the smallest unit that actually satisfies the AC — a specific field, function, or test case — not on the file, the test suite's top-level describe, or the enclosing type. Widen the placement only when the AC's scope genuinely spans that boundary (e.g. an endpoint, contract). Target minimal tracing amount that make sense.

## Definition of done

A change isn't finished until:

- `spex check` passes (no error-severity findings) for every affected package.
- `spex coverage` shows no gap for any spec acceptance criterion the change touches.
- Every acceptance criterion touched carries `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>` tracing annotations, e.g. `@spec impl place-pizza-order#auth-required` and `@spec test place-pizza-order#auth-required`.

## CLI commands

Run from anywhere inside the project; `spex <command> --help` lists every option.

- `spex info [--package <name>...]` — print `spex.yaml` merged with each `spex.package.yaml`: project and per-package configuration, including paths.
- `spex check [--package <name>...] [--files <path>...]` — run the rule checks; fails on error findings.
- `spex coverage [--package <name>...] [-v]` — report `@spec` annotation coverage per acceptance criterion.
- `spex specs [--id <spec-id>...] [--package <name>...] [--ai] [--output <file>]` — print specs.
- `spex rules [--id <rule-id>...] [--package <name>...] [--ai] [--output <file>]` — print the active rules per package.
- `spex jq [-r] [-c] <filter> [file]` — run a jq filter on JSON or YAML from a file or stdin, e.g. `spex info | spex jq -r .project_name`.
- `spex check-schema --schema <schema> --files <path>...` — validate JSON or YAML files against a JSON Schema.

Pass `--ai` to `specs` and `rules`: it drops fields AI doesn't need and adds ones it does. Prefer `--output <file>` for them too: long output gets truncated in a terminal. `spex init` and `spex update` run through the `spex-setup` and `spex-update` skills.

## Docs

- [docs/glossary.md](docs/glossary.md) — terms used across specs, rules, schemas, and skills.

## Core discipline

- **Simplicity**: All technical assets, tools, and processes must be simple and intuitive for any teammate or AI to quickly understand, execute, and safely change.
- **Single source of truth**: every business rule, behavior, contract, and configuration value must be mastered in exactly one place. Any other system that needs this information must reference, link to, or generate from this primary source rather than duplicating it. Repeating wording or an idea for readability is not duplication.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. Add a comment only when the name cannot carry the meaning alone — a non-obvious why, hidden constraint, or surprising behavior — in one short line. Never comment what the code does, restate a name, or narrate an extraction or refactor.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Systemic Fix Over Quick Fix**: Always implement the proper, maintainable solution. Never apply a workaround, hack, band-aid, or temporary fix, unless the user explicitly asks for one in that request. If the proper fix requires refactoring, architectural changes, or expanding the scope of the current task, stop immediately, present the proper solution, and ask the user for explicit permission before proceeding.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless triggered by the "Systemic Fix Over Quick Fix" rule.
- **Stop on major findings**: When investigation reveals a major problem — a bug, a spec/code mismatch, a failing check, a design flaw — related to the task or not, stop. Report it with evidence and ask the user how to proceed. Never fix it silently, work around it, or ignore it. Report minor problems at the end of the task.
- **Propose major improvements**: When, at any step, a substantially simpler or better approach than the requested one emerges, stop, propose it, and wait for the user's decision before continuing. Propose a behavior change as a spec change first, since the spec is the source of truth. Report minor improvements at the end of the task.
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

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
