# Contributing

## Prerequisites

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/)
- **dprint**: [Install dprint](https://dprint.dev/install/) (formats Markdown, YAML, and JSON)

## Steps

1. Create a feature branch from `dev`.
2. Make your changes.
3. Add tests covering your changes if applicable.
4. Update the examples in `examples/` if your changes affect them.
5. Run `dprint fmt` to format Markdown, YAML, and JSON files.

Before opening a pull request, make sure the checks pass:

```sh
cargo check  # Lint
cargo fmt    # Format Rust
cargo test   # Test
dprint check # Check md/yaml/json formatting
```

## Contributing new skills

Skills, even when not used, have their metadata loaded in context on every session start. To avoid context pollution from a large number of skills, we don't want to exceed 100 skills or ~10_000 metadata tokens. This means we should include the most valuable ones, that will be used often and don't duplicate other skills.

If you find a skill really useful, please share it with others. Please describe a real use case for it and how it helped.

## Improving existing skills

Improving existing skills helps all developers using them, so if you find a way to improve one, please contribute back:

Create a separate version of the skill. Make your changes and test them by comparing the output results of the old and new skill versions for the same prompt.

You can also disable upstream synchronization in the skill metadata if the changes are specific to your project and you just want to use a custom version.

```yaml
metadata:
  sync-with-upstream: false
```

## Adding new or improving existing rule

We can have as many rules as we want and just pick the applicable ones for a project, so as long as they are useful and don't duplicate others, we will add many.
