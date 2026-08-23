---
name: pepperoni-pizza-api
description: Information, rules, commands and other pepperoni-pizza-api repository context. MUST be loaded before reading, writing, testing, or running any code in that repository.
---

# General

Run `spex info --repo api` for general repository information and applicable rules.

# Paths

- `src/main.rs` - the whole server: config loading, routes, handlers, in-memory state
- `src/tests.rs` - tests exercising the handlers directly

# Commands

Run from the `pepperoni-pizza-api` repository root.

- `cargo run` - run the dev server
- `cargo test` - run the whole test suite
- `cargo clippy --all-targets` - lint
- `cargo fmt` - format files
