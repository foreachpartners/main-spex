# spex

AI-first specification-driven development framework with human in control.

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
  tech_spec.rs      Tech-spec model
  biz_spec.rs       Biz-spec model
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
  tech-specs/       Reusable tech-spec templates
examples/           Sample projects (pepperoni-pizza)
tests/              Integration tests that build and run the spex binary
```

Unit tests live next to the code they cover as `*_test.rs` files. End-to-end
tests that drive the compiled binary live in `tests/`.

Keep this section in sync when the structure changes.

## Principles

- **Clean Code**: Simple code is clean if it can be understood easily – by everyone on the team. Clean code can be read and enhanced by a developer other than its original author. With understandability comes readability, changeability, extensibility, and maintainability.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. This project is not public-facing, so do NOT write doc comments (`///`, `//!`) and do NOT write comments that restate what the code does. The only acceptable comment is a short `//` note explaining a non-obvious "why" (a constraint, footgun, or workaround) that names alone cannot convey, e.g. a hard-exit that avoids a runtime segfault. When in doubt, leave it out.
- **YAGNI** (You Aren't Gonna Need It): Implement only what the current requirements demand. Don't add functionality, configuration, or generalization for hypothetical future needs.
-

## Rules

- **Bump the CLI version**: Whenever you change the CLI's behavior, bump `version` in `Cargo.toml`. Scaffolded `spex.yaml` records this as `cli_version`, and spex warns the user to run `spex update` when the installed CLI differs from the version that wrote their config.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Glossary Terms**: Use the terms defined in `docs/glossary.md` consistently across code, messages, schemas, and docs.
- **Writing style**: Follow the authoring conventions in `templates/tech-specs/TECH-WRITING-STYLE.yaml` (e.g. path placeholders) across docs, skills, and schemas.
