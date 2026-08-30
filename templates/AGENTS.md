# General

This project is governed by `spex` AI-first specification-driven development framework with human in control: the agent writes every artifact — specs, contracts, plans, code, and tests — and the human approves each one before the next is generated.

## Artifact chain

BRD (why) → PRD (what) → spec (capability) → contract (interface, if applicable) → plan (how, optional) → implementation (code) → test (proof). Every behavioral change traces to a Spec ID, annotated at the layer boundary (`@spec contract|impl|test <spec-id>`, or `x-spec: <spec-id>` in OpenAPI).

- **Spec** — one coherent capability the system must provide, with acceptance criteria.
- **Rule** — one coherent cross-cutting convention (style, structure, security) enforced repo-wide, with acceptance criteria proven by `spex check`.

## Workspace layout

The workspace root holds this repo (`<project-name>-spex`, the single source of truth for specs, contracts, and plans) alongside the other workspace repositories. Check `spex.yaml` for all project configuration.

Skills live under `.agents/skills`; `.claude` is a symlink to `.agents`, so `.claude/skills` is the same directory — edit skill files under `.agents/skills` only.

## spex commands

Run from anywhere in the workspace — `spex` locates `spex.yaml` automatically. Global flag: `--verbose`/`-v` (stream progress to stderr; used by `check`).

- `spex init` — initialize the spex workspace repository.
- `spex update` — rewrite bundled skills, schemas, scripts, and rules with the versions shipped in the installed CLI.
- `spex check` — run rule checks and gate on `error`-severity findings.
  - `--repo <name>...` — limit to named repositories (default: all).
  - `--files <path>...` — limit to given files (default: all non-ignored files).
  - `--fill-preexisting-errors` — record current error findings as a per-repository baseline in `spex.yaml`.
  - `--verbose`/`-v` — stream per-repository/per-check progress to stderr.
- `spex check-schema` — validate JSON or YAML files against a JSON Schema.
  - `--schema <path>` — schema to validate against (required).
  - `--files <path>...` — files to validate (required).
- `spex coverage` — report `@spec` annotation and acceptance-criteria coverage for active specs.
- `spex info` — print the resolved config (repos, paths, contracts, rules).
  - `--print <field>` — print a single top-level scalar field (e.g. `project_name`).
  - `--repo <name>` — limit output to the general fields plus the named repository.
- `spex rules` — print the active rules per repository, as YAML.
  - `--ai` — drop fields an agent doesn't need (`sync_with_upstream`, `checks`); adds a `path` field for relative to spex filename location.
  - `--id <id>...` — limit to given Rule IDs (default: all).
  - `--repo <name>...` — limit to named repositories (default: all).
  - `--output <path>` — write the rendered YAML to a file instead of stdout (the full rules list is large and can get truncated when printed to a terminal or piped through a shell).
- `spex specs` — print the workspace specs, as YAML.
  - `--ai` — add a `path` field for relative to spex filename location.
  - `--id <id>...` — limit to given Spec IDs (default: all).
  - `--scope <name>...` — limit to specs whose `scope` includes the given repository short name (default: all).
  - `--output <path>` — write the rendered YAML to a file instead of stdout (the full specs list is large and can get truncated when printed to a terminal or piped through a shell).

After all changes across the workspace, run `spex check` (optionally scoped with `--repo`) and `spex coverage`.

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

Follow [Conventional Commits](https://www.conventionalcommits.org/): `<type>[optional scope]: <description>`, with an optional body separated by a blank line. See `rules/COMMIT-MESSAGE-FORMAT.yaml`.

- Use the `spec` scope for spec changes (e.g. `feat(spec)`, `fix(spec)`).
- Use the `rule` scope for rule changes (e.g. `feat(rule)`, `fix(rule)`).
- End every `spec`-scoped description with at least one comma-separated Spec ID: `<Spec ID>[, <Spec ID>]`.

Examples:

```
style: format md files
feat(rule): pin dependencies as required by DEPENDENCIES
feat(spec): list available pizzas LIST-PIZZA-MENU
fix(spec): reject negative quantities CART-ADD-ITEM, CART-VALIDATE
```
