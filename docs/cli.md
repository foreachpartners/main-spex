# CLI

The `spex` command-line tool manages a spex workspace.

```sh
spex --help
```

## Commands

### `spex init`

Initializes the spex workspace repository. Run it from the workspace root or from an existing `spex` directory.

It creates the `spex` directory (if missing), prompts to confirm the project name (defaulting to the workspace root folder name) and the discovered workspace repositories, then writes `spex.yml`, a `<project-name>.code-workspace` file, and the bundled templates (schemas, skills, scripts, and default tech specs).

```sh
spex init
```

Finish setup afterwards by running the `/spex-setup` skill.

### `spex check`

Runs the technical spec checks declared in `spex.yml` against the workspace. Each workspace repository is checked against the TECH specs listed in its `tech-specs`, and every check's findings are reported per repository. Run it from anywhere inside the workspace; spex locates the config by walking up to the nearest `spex.yml`.

```sh
spex check
```

The command exits with a non-zero status when any `error`-severity finding is reported; `warn`-severity findings do not affect the exit status.

#### Flags

- `--repo <name>...` — limit the run to the named workspace repositories. Defaults to all repositories in `spex.yml`.
- `--files <path>...` — limit the run to the given files (relative to each repository). Defaults to all non-ignored files.

```sh
spex check --repo api frontend
```

```sh
spex check --repo api --files src/main.rs src/orders.rs
```

## Install

Install the `spex` binary globally from a source checkout. `cargo install` builds an optimized binary and copies it to `~/.cargo/bin/`, which is on your `PATH` when Rust is installed via `rustup`.

```sh
cargo install --path . # Build and install
```

```sh
cargo install --path . --force  # Reinstall after changes
```

```sh
cargo uninstall spex # Uninstall
```

There is no need to run `cargo build` first; `cargo install` compiles the release binary itself.

## Development

These commands operate on the `spex` source checkout itself.

### Build

```sh
cargo build            # Debug build
```

```sh
cargo build --release  # Optimized build
```

### Test

```sh
cargo test
```

### Lint and format

```sh
cargo check  # Type-check and lint
```

```sh
cargo fmt    # Format
```
