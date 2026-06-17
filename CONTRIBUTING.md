# Contributing

1. Create a feature branch from `dev`.
2. Make your changes.
3. Add tests covering your changes.
4. Update the examples in `examples/` if your changes affect them.
5. Run `dprint fmt` to format Markdown, YAML, and JSON files.

Before opening a pull request, make sure the checks pass:

```sh
cargo check  # Lint
cargo fmt    # Format Rust
cargo test   # Test
dprint check # Check md/yaml/json formatting
```
