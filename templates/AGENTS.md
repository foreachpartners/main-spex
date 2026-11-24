# spex

This project is governed by `spex`. The AI-first development framework that turns specifications into verified code — keeping AI running at full speed while ensuring slop-free results.

## Artifact chain

PRD (why and what) -> spec (capability) -> contract (interface, if applicable) -> implementation (code) -> test (proof).

## Traceability

Every behavioral change traces to a spec acceptance criterion via a trace comment `spec:impl|test|contract <spec-id>#<ac-id>` e.g `spec:impl place-pizza-order#auth-required` for implementation acceptance criterion.

For `spec:test` and `spec:impl` ac-id for each acceptance criterion is required. Contract type is defined in spec, e.g `contract: for-each-ac`:

- `for-each-ac` - contract for each ac is required, same as for `spec:impl` and `spec:test`.
- `single` - one contract required for entire spec, traced by `spec-id` only, e.g. `spec:contract place-pizza-order`.
- `none` - no contract required.

Never trace a layer that `coverage_ignore` waives, for a whole spec or for one acceptance criterion: leave it untraced instead of adding a placeholder annotation or test.

Place the trace comment on the code that performs the behavior the AC describes: the statement, match arm or function that checks, rejects, returns, logs or changes something. Pick the smallest such unit.

- If the AC is a behavior (must require|reject|return|log…), never trace a declaration that only holds data (struct, field, parameter). Trace the code that acts on it. When a framework enforces it implicitly, such as a missing body field or an extractor parameter, trace the code that handles the failure: the rejection arm or the function.
- Trace a declaration only when the AC is about the declaration itself, meaning what a type holds, stores or is named.
- Never trace the file, the test suite's top-level describe, or the enclosing type. Widen only when the AC's scope genuinely spans that boundary (e.g. an endpoint, contract).

Target minimal tracing.

## Definition of done

A change isn't finished until:

- `spex check` passes (no error-severity findings) for every affected package.
- `spex coverage` shows no gap for any spec acceptance criterion the change touches.
- Every acceptance criterion touched carries `spec:impl <spec-id>#<ac-id>` and `spec:test <spec-id>#<ac-id>` tracing annotations, e.g. `spec:impl place-pizza-order#auth-required` and `spec:test place-pizza-order#auth-required`.
- Every acceptance criterion `spex coverage` shows fully traced has its `is_done: false` removed, and every spec whose criteria are all done has `status: active`.

## CLI commands

Run from anywhere inside the project; `spex <command> --help` lists every option.

- `spex info [--package <name>...]` — print `spex.yaml` merged with each `spex.package.yaml`: project and per-package configuration, including paths.
- `spex check [--package <name>...] [--files <path>...]` — run the rule checks; fails on error findings.
- `spex coverage [--package <name>...] [-v]` — report `spec:` annotation coverage per acceptance criterion.
- `spex specs [--id <spec-id>...] [--package <name>...] [--search <keyword>...] [--fields <field>...] [--output <file>]` — print specs, each with its file `path` after its `id`; only those containing any keyword, or only the named fields of each (e.g. `--fields id title description` for a catalog).
- `spex rules [--id <rule-id>...] [--package <name>...] [--search <keyword>...] [--fields <field>...] [--output <file>]` — print the active rules per package the same way; `--fields id path title description acceptance_criteria examples` leaves out the `checks`.
- `spex jq [-r] [-c] <filter> [file]` — run a jq filter on JSON or YAML from a file or stdin, e.g. `spex info | spex jq -r .project_name`.
- `spex check-schema --schema <schema> --files <path>...` — validate JSON or YAML files against a JSON Schema.
- `spex lib <specs|rules> [--id <id>...] [--search <keyword>...] [--fields <field>...] [--output <file>]` — print the bundled template specs or rules as a YAML list, in full, only the named ones, only those containing any keyword, or only the named fields of each (e.g. `--search password --fields id description` for a catalog of matches).
- `spex lib rules --id <rule-id>... --copy <dir>` — write the named bundled rules, with their script folders, into `<dir>` (e.g. `<spex-dir>/rules`) to take them into the project; never overwrites.

Prefer `--output <file>` for `specs` and `rules`: long output gets truncated in a terminal. `spex init` and `spex update` run through the `spex-setup` and `spex-update` skills.

## Core discipline

The most important principles, guiding every change beyond what rules and specs enforce.

- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works and that anyone can quickly understand and safely change. Avoid unnecessary abstraction, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Master every business rule, behavior, contract, configuration value, and piece of shared logic in exactly one place; everything else references, reuses, or generates from it. Repeating wording for readability is not duplication.
- **Maintainability**: Optimize for long-term maintainability over short-term speed, so code stays easy to change and extend as the application grows. Never apply a workaround or hack unless explicitly asked; if the proper fix goes beyond the task, propose it first.
- **Consistency**: Follow the patterns already in the codebase. When a pattern must change, change it everywhere it is used; never mix old and new.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. Add a comment only when the name cannot carry the meaning alone — a non-obvious why, hidden constraint, or surprising behavior — in one short line. Never comment what the code does, restate a name, or narrate an extraction or refactor.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless the "Maintainability" rule requires it.
- **Stop on major findings**: When investigation reveals a major problem — a bug, a spec/code mismatch, a failing check, a design flaw — related to the task or not, stop. Report it with evidence and ask the user how to proceed. Never fix it silently, work around it, or ignore it. Report minor problems, and major ones a small change can fix, at the end of the task instead of stopping.
- **Propose major improvements**: When, at any step, a substantially simpler or better approach than the requested one emerges, stop, propose it, and wait for the user's decision before continuing. Propose a behavior change as a spec change first, since the spec is the source of truth. Report minor improvements at the end of the task.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/): `<type>[optional scope]: <description>`, with an optional body separated by a blank line. See `rules/commit-message-format.yaml`.

Tag every spec-related commit with its ID(s) using `spec:ref <Spec ID>[#<AC ID>][, <Spec ID>[#<AC ID>]]`.

Examples:

```
style: format md files
feat(ui): list available pizzas spec:ref list-pizza-menu
fix(cart): reject negative quantities spec:ref cart-add-item, cart-validate
feat(orders): set status on order creation spec:ref place-pizza-order#created-order-status
```

## Glossary

- [docs/glossary.md](docs/glossary.md) inlined.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
