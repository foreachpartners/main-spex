# CLI

The `spex` command-line tool manages a spex workspace.

```sh
spex --help
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
