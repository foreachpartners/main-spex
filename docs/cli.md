# CLI

The `spex` command-line tool manages a spex workspace.

```sh
spex --help
```

## Commands

### `spex init`

Initializes the spex workspace repository. Run it from the workspace root or from an existing `spex` directory.

It creates the `spex` directory (if missing), prompts to confirm the project name (defaulting to the workspace root folder name) and the discovered workspace repositories, then writes `spex.yaml`, a `<project-name>.code-workspace` file, and the bundled templates (schemas, skills, scripts, and default tech specs).

```sh
spex init
```

Finish setup afterwards by running the `/spex-setup` skill.

### `spex update`

Rewrites the bundled templates in the spex repository — skills, schemas, scripts, and tech specs (including their checks) — with the versions shipped in the installed `spex` binary. Run it from anywhere inside the workspace; spex locates the repository by walking up to the nearest `spex.yaml`.

Managed files are overwritten with the latest versions, while any additional skills, scripts, schemas, or tech specs you have added are left untouched. Use it after upgrading the `spex` binary to pull in framework updates.

```sh
spex update
```

### `spex check`

Runs the technical spec checks declared in `spex.yaml` against the workspace. Each workspace repository is checked against the TECH specs listed in its `tech-specs`, and every check's findings are reported per repository. Run it from anywhere inside the workspace; spex locates the config by walking up to the nearest `spex.yaml`.

```sh
spex check
```

The command exits with a non-zero status when any `error`-severity finding is reported; `warn`-severity findings do not affect the exit status.

A `script` check runs the program named by its `script:` filename (for example `writing-style.sh`), resolved at `scripts/<spec-id>/<script>` under the spex directory for checks inherited from tech specs, and under the repository directory for per-repo `checks_additional` and for a `checks_override` that sets its own `script`. Set `run: per-file` (the default) to pass the matched files as arguments, or `run: per-repo` to run the script once in the repository directory with no arguments.

A repository can adjust the checks it inherits, or add new ones, in `spex.yaml` via `checks_override` (a partial patch identified by `spec` + `name`) and `checks_additional` (full checks). See the [guide](guide.md#per-repository-checks) for details.

#### Flags

- `--repo <name>...` — limit the run to the named workspace repositories. Defaults to all repositories in `spex.yaml`.
- `--files <path>...` — limit the run to the given files (relative to each repository). Defaults to all non-ignored files.

```sh
spex check --repo api frontend
```

```sh
spex check --repo api --files src/main.rs src/orders.rs
```

### `spex check-schema`

Validates JSON or YAML files against a JSON Schema. It loads the schema, resolves any cross-file `$ref`s against sibling schema files in the schema's own directory (fully offline, no network access), and validates each file. Violations are printed to stderr as `<file>: <message>`, and the command exits non-zero when any file fails.

This is a general-purpose validator. The `TECH-SPEX-CONFORMANCE` tech spec uses it to keep the spex repository's own spec files (`spex.yaml`, `biz-specs/<spec-id>.yaml`, `tech-specs/<spec-id>.yaml`) valid against the bundled schemas.

#### Flags

- `--schema <path>` — the JSON Schema the files are validated against (required).
- `--files <path>...` — one or more JSON or YAML files to validate against the schema (required).

```sh
spex check-schema --schema schemas/tech-spec.schema.json --files tech-specs/TECH-NAMING-CONVENTION.yaml
```

```sh
spex check-schema --schema schemas/biz-spec.schema.json --files biz-specs/BIZ-LIST-PIZZA-MENU.yaml biz-specs/BIZ-PLACE-PIZZA-ORDER.yaml
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
