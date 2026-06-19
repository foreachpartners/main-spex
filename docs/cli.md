# CLI

The `spex` command-line tool manages a spex workspace.

```sh
spex --help
```

## Commands

### `spex init`

Initializes the spex workspace repository. Run it from the workspace root or from an existing `spex` directory.

It creates the `spex` directory (if missing), prompts to confirm the project name (defaulting to the workspace root folder name) and the discovered workspace repositories, then writes `spex.yaml` (recording the installed CLI's version as `cli_version`), a `<project-name>.code-workspace` file, and the bundled templates (schemas, skills, scripts, and default tech specs).

```sh
spex init
```

Finish setup afterwards by running the `/spex-setup` skill.

### `spex update`

Rewrites the bundled templates in the spex repository — skills, schemas, scripts, and tech specs (including their checks) — with the versions shipped in the installed `spex` binary. Run it from anywhere inside the workspace; spex locates the repository by walking up to the nearest `spex.yaml`.

Managed files are overwritten with the latest versions, while any additional skills, scripts, schemas, or tech specs you have added are left untouched. It also updates `cli_version` in `spex.yaml` to the installed binary's version. Use it after upgrading the `spex` binary to pull in framework updates; spex warns on every command when `cli_version` does not match the installed CLI.

```sh
spex update
```

### `spex check`

Runs the technical spec checks declared in `spex.yaml` against the workspace. Each workspace repository is checked against the TECH specs listed in its `tech_specs`, and every check's findings are reported per repository. Run it from anywhere inside the workspace; spex locates the config by walking up to the nearest `spex.yaml`.

```sh
spex check
```

The command exits with a non-zero status when any `error`-severity finding is reported; `warn`-severity findings do not affect the exit status.

A `script` check runs the program named by its `script:` filename (for example `writing-style.sh`), resolved at `scripts/<spec-id>/<script>` under the spex directory for checks inherited from tech specs, and under the repository directory for per-repo `checks_additional` and for a `checks_override` that sets its own `script`. Set `run: per-file` (the default) to pass the matched files as arguments, `run: per-repo` to run the script once in the repository directory with no arguments, or `run: workspace` to run the script once in the spex directory with the matched files from every repository (as paths relative to the spex directory) as arguments. A `workspace` script runs a single time even when several repositories inherit it, so it can reason about the whole workspace at once (for example, flagging spec-id annotations that point at a spec that no longer exists).

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

### `spex coverage`

Reports how well the active business specs are covered by spec-id annotations across the workspace. For every BIZ spec whose `status` is `active`, it requires that the code carries an implementation annotation (`@spec impl <spec-id>`), a test annotation (`@spec test <spec-id>`), and a contract annotation (`@spec contract <spec-id>`, or the OpenAPI `x-spec: <spec-id>` property). All three layers are required by default. A spec with no `status` field defaults to `planned`, so it is ignored. Run it from anywhere inside the workspace; spex locates the config by walking up to the nearest `spex.yaml`.

```sh
spex coverage
```

A single annotation of each kind anywhere in any workspace repository counts — the search is not scoped to the spec's `scope`. The `spex` repository itself is excluded from the search, and files ignored by `.gitignore` are skipped. Specs that are not `active` are listed only in the per-status totals and are otherwise ignored.

A spec can relax the requirement with the optional `coverage_ignore` field in its YAML, listing any of `contract`, `impl`, or `test`. A layer listed there is treated as ok even when its annotation is missing, so it never counts as a gap:

```yaml
id: BIZ-PLACE-PIZZA-ORDER
status: active
coverage_ignore: [contract]
```

The report prints the total number of business specs per status (omitting any status with no specs), then lists each active spec with its coverage:

- `✓` — implementation, test, and contract annotations are all present.
- `⚠` — every required layer is present, but an ignored layer is still missing.
- `✗` — a required layer is missing.

Each incomplete spec lists what is missing (for example `missing: test, contract (ignored)`). A missing layer listed in `coverage_ignore` is reported but never counts as a gap. The command exits with a non-zero status when any active spec has a gap (a missing required layer).

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
