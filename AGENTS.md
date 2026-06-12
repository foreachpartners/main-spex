# spex

AI-first specification-driven development framework with human validation.

## Quick commands

```sh
cargo check # Lint
cargo fmt   # Format
cargo test  # Test
cargo install --path . --force  # Reinstall cli
spex --help # CLI command
```

## Main dependencies

- **rust** `1.96.0` — language toolchain (edition 2024).
- **clap** `4.6.1` — command-line argument parsing (derive).
- **error-stack** `0.7.1` — contextual error reporting.
- **thiserror** `2.0.18` — derive macro for error type definitions.

## Principles

- **Clean Code**: Simple code is clean if it can be understood easily – by everyone on the team. Clean code can be read and enhanced by a developer other than its original author. With understandability comes readability, changeability, extensibility and maintainability.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. This project is not public-facing, so do NOT write doc comments (`///`, `//!`) and do NOT write comments that restate what the code does. The only acceptable comment is a short `//` note explaining a non-obvious "why" (a constraint, footgun, or workaround) that names alone cannot convey, e.g. a hard-exit that avoids a runtime segfault. When in doubt, leave it out.
- **YAGNI** (You Aren't Gonna Need It): Implement only what the current requirements demand. Don't add functionality, configuration, or generalization for hypothetical future needs.
