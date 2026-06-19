# CLI

The `spex` command-line tool manages a spex workspace.

```sh
spex --help
```

## Install

### Prerequisites

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/) (required for building the spex CLI)
- **Git**: [Install Git](https://git-scm.com/install/)

### Installation

- **spex CLI**: Clone spex repository (repo) and install spex cli tool:
  ```sh
  git clone https://github.com/ziimakc/spex
  cd spex
  cargo install --path . --force
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

A `script` check runs the program named by its `script:` filename (for example `writing-style.sh`), always resolved at `scripts/<spec-id>/<script>` under the spex directory — including the scripts named by per-repo `checks_additional` and `checks_override`. Set `run: per-file` (the default) to pass the matched files as arguments, `run: per-repo` to run the script once in the repository directory with no arguments, or `run: workspace` to run the script once in the spex directory with the matched files from every repository (as paths relative to the spex directory) as arguments. A `workspace` script runs a single time even when several repositories inherit it, so it can reason about the whole workspace at once (for example, flagging spec-id annotations that point at a spec that no longer exists).

A failing `script` reports each violation on its own stdout line, following the script location contract: `<location><TAB><message>`, where `<location>` is a repository-relative file (optionally `<file>:<line>`) and `<message>` describes the violation. spex parses each line into a separate finding, so a script gets the same per-file precision as a `regexp` or `glob` check. A line with no location (empty before the tab, or no tab at all) becomes a file-less finding, and a script that exits non-zero without printing anything reports a single file-less finding built from its stderr or exit status. The exit code keeps its meaning: non-zero means the check failed.

A repository can adjust the checks it inherits, or add new ones, in `spex.yaml` via `checks_override` (a partial patch identified by `spec` + `name`) and `checks_additional` (full checks). See the [guide](guide.md#per-repository-checks) for details.

A repository can also baseline pre-existing `error`-severity findings with `checks_preexisting_errors`, so a check can be enforced going forward without first fixing every existing violation. Each entry names the check by `spec` + `check`; an `error` finding from that check is dropped when its location file is listed in `files`, or always when `files` is omitted. New errors in other files still surface, and `warn`-severity findings are never affected:

```yaml
repositories:
  - repository_name: frontend
    path: ../frontend
    tech_specs: [TECH-WRITING-STYLE]
    checks_preexisting_errors:
      - spec: TECH-WRITING-STYLE
        check: no-cyrillic
        files: [README.md, docs/intro.md]
```

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
