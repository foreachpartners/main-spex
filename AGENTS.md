# spex

The AI-first development framework that turns natural specifications into deterministic code — keeping AI running at full speed, with humans safely in the driver’s seat.

## Goal: unleash AI's full speed without slop

AI operates fast — the framework's job is unleash it speed while ensuring slop-free verifiable results.

## Core Concepts

spex is built on three simple pieces:

- **Spec** — one coherent capability the system must provide, with acceptance criteria proven by `spex coverage`. Specs are single source of truth for project - all behaviour, contracts, code, tests and other artifacts come from it and must adhere to it.
- **Rule** — one coherent cross-cutting convention (style, structure, security), with acceptance criteria proven by `spex check`. Each repository must adhere to a list of rules applicable to it defined in `spex.yaml`
- **Skill** — the instructions an AI agent follows to do the job consistently and correctly.
- **Traceability** — the `@spec` annotations stitching together spec, contracts, implementations, and tests, used to ensure coverage and to look up any of spec, contract, implementation, or test artifacts.

## Specification

`docs/specification.md` is the single source of truth for how the framework behaves, written as specs (description + acceptance criteria), and `docs/cli.md` for the CLI's command behavior. Consult them before changing behavior, and keep them in sync when behavior changes. Also on changes update if needed `docs/tutorial.md` and `examples/shopping-notes`.

## Quick commands

```sh
just lint       # cargo check
just fmt        # Format everything (fmt-docs + cargo fmt)
just test       # cargo test
just fmt-docs   # dprint fmt (md/yaml/json)
just check-docs # dprint check (md/yaml/json)
just install    # Reinstall cli
just help       # spex --help
```

## Rules

On version change fill `CHANGELOG.md`

On CLI command or option changes, sync the CLI commands list in `templates/AGENTS.md`

When a change requires migration steps in a spex project, fill `templates/skills/spex-update/references/migration_guide.md` and update `templates/spex.yaml`

## Project structure

```
src/                CLI source
  main.rs           Entry point, arg parsing
  config.rs         Config loading
  workspace.rs      Resolved spex dir + config, with spec loaders (shared by commands)
  rule.rs           Rule model
  spec.rs           Spec model (status, acceptance criteria as {id, must|..., is_done} objects)
  annotations.rs    @spec impl|test annotation parsing (source for coverage)
  style.rs          Terminal color styling (shared by commands)
  util.rs           Shared helpers (file walking, yaml-dir loading, pluralize)
  templates.rs      Bundled template scaffolding (shared by init/update)
  commands/         Subcommands (init, update, check, check_schema, coverage, rules, info, specs)
    check/          check command: orchestration (mod), report
      conformance/  Structural checks on specs/rules themselves (ids, ac order, schema, script refs)
      runners/      One runner per check type (ast_grep, glob, regexp, script)
docs/                          Framework reference docs, bundled into projects by init/update (single source of truth)
  glossary.md              Terms used across code, messages, schemas, and docs
  framework-principles.md  Non-negotiable foundation spex is built on
  conventions.md           Engineering values spex follows but didn't invent
  specification.md         Framework behavior, minus the CLI
  cli.md                   Single source of truth for CLI behavior
  tutorial.md              Hands-on walkthrough of adopting spex
  faq.md                   Frequently asked questions
templates/                     Assets scaffolded into projects
  schemas/                     Spec schemas
  scripts/                     Helper scripts
  skills/                      Agent skills
  rules/                       Reusable rule templates
  docs/                        Framework docs shipped to projects (symlinks into docs/)
  spex.yaml                    Workspace config seeded by init (project name filled in)
  project-name.code-workspace  Editor workspace file seeded by init
examples/                      Sample example projects with/without spex(shopping-notes, shopping-notes-no-spex)
tests/                         Integration tests that build and run the spex binary
```

Unit tests live next to the code they cover as `*_test.rs` files. End-to-end
tests that drive the compiled binary live in `tests/`.

Keep this section in sync when the structure changes.

# Core discipline

- **Simplicity**: All technical assets, tools, and processes must be simple and intuitive for any teammate or AI to quickly understand, execute, and safely change.
- **Single source of truth**: every piece of data, logic, or business rule must be mastered in exactly one place. Any other system that needs this information must reference, link to, or generate from this primary source rather than duplicating it.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables instead of comments. Never add a comment — not for WHAT the code does, not for WHY it exists, not to note an extraction, refactor, or design rationale. Write no comments at all unless the user explicitly asks for one in that exact request.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Systemic Fix Over Quick Fix**: Never apply temporary workarounds, hacks, or band-aids to bypass a structural issue. If a proper, clean fix requires refactoring, architectural changes, or expanding the scope of the current task, stop immediately. Present the proper solution and ask the user for explicit permission to make that change before proceeding.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless triggered by the "Systemic Fix Over Quick Fix" rule.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.
- **Bump the CLI version**: Whenever you change the CLI's behavior, bump `version` in `Cargo.toml`. Scaffolded `spex.yaml` records this as `cli_version`, and spex warns the user to run `spex update` when the installed CLI differs from the version that wrote their config.
- **Glossary Terms**: Use the terms defined in `docs/glossary.md` consistently across code, messages, schemas, and docs.
- **Single source of truth**: `docs/specification.md` defines how the framework behaves and `docs/cli.md` how the CLI behaves. Treat both as authoritative, and update them whenever behavior changes.
