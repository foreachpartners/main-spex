# spex

AI-first specification-driven development framework with human in control.

## Specification

[`docs/specification.md`](docs/specification.md) is the single source of truth for how the framework behaves, written as specs (description + acceptance criteria). Consult it before changing behavior, and keep it in sync when behavior changes.

## Quick commands

```sh
cargo check # Lint
cargo fmt   # Format
cargo test  # Test
dprint fmt   # Format md/yaml/json
dprint check # Check md/yaml/json formatting
cargo install --path . --force  # Reinstall cli
spex --help # CLI command
```

## Project structure

```
src/                CLI source
  main.rs           Entry point, arg parsing
  config.rs         Config loading
  workspace.rs      Resolved spex dir + config, with spec loaders (shared by commands)
  rule.rs           Rule model
  spec.rs           Spec model (status, acceptance-criteria checkbox)
  style.rs          Terminal color styling (shared by commands)
  util.rs           Shared helpers (file walking, yaml-dir loading, pluralize)
  templates.rs      Bundled template scaffolding (shared by init/update)
  commands/         Subcommands (init, update, check, check_schema, coverage)
    check/          check command: orchestration (mod), runners, report
docs/               Glossary, conventions, CLI reference, FAQ
templates/          Assets scaffolded into projects
  schemas/          Spec schemas
  scripts/          Helper scripts
  skills/           Agent skills
  rules/            Reusable rule templates
examples/           Sample projects (pepperoni-pizza, pepperoni-pizza-no-spex)
tests/              Integration tests that build and run the spex binary
```

Unit tests live next to the code they cover as `*_test.rs` files. End-to-end
tests that drive the compiled binary live in `tests/`.

Keep this section in sync when the structure changes.

# Rules

Code should be simple enough that any teammate or AI — not just its author — can read, run, and safely change it.

- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables instead of comments. Never add a comment — not for WHAT the code does, not for WHY it exists, not to note an extraction, refactor, or design rationale. Write no comments at all unless the user explicitly asks for one in that exact request.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Systemic Fix Over Quick Fix**: Never apply temporary workarounds, hacks, or band-aids to bypass a structural issue. If a proper, clean fix requires refactoring, architectural changes, or expanding the scope of the current task, stop immediately. Present the proper solution and ask the user for explicit permission to make that change before proceeding.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless triggered by the "Systemic Fix Over Quick Fix" rule.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.
- **Bump the CLI version**: Whenever you change the CLI's behavior, bump `version` in `Cargo.toml`. Scaffolded `spex.yaml` records this as `cli_version`, and spex warns the user to run `spex update` when the installed CLI differs from the version that wrote their config.
- **Glossary Terms**: Use the terms defined in `docs/glossary.md` consistently across code, messages, schemas, and docs.
- **Writing style**: Follow the authoring conventions in `templates/rules/writing-style.yaml` (e.g. path placeholders) across docs, skills, and schemas.
- **Single source of truth**: `docs/specification.md` defines how the framework behaves. Treat it as authoritative, and update it whenever you change framework behavior.
