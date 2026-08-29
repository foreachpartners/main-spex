# CLI

The `spex` command-line tool manages a spex workspace.

```sh
spex --help
```

## Global flags

- `--verbose` (`-v`) — print progress to stderr while the command executes, so stdout still carries only the command's actual output. Currently the `check` command uses it, logging each repository and each check as it starts and finishes along with how long each took. Useful for spotting which check is slow on a large workspace.

## Install

### Prerequisites

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/) (required for building the spex CLI)
- **Git**: [Install Git](https://git-scm.com/install/)

### Installation

- **spex CLI**: Clone spex repository (repo) and install spex cli tool:
  ```sh
  git clone https://github.com/foreachpartners/spex
  cd spex
  cargo install --path . --force
  ```

## Commands

### `spex init`

Initializes the spex workspace repository. Run it from the workspace root or from an existing spex repository directory (`<project-name>-spex`).

It prompts to confirm the project name (defaulting to the workspace root folder name), creates the spex repository directory `<project-name>-spex` (if missing), prompts to confirm the discovered workspace repositories, then writes `spex.yaml` (recording the installed CLI's version as `cli_version`), a `<project-name>.code-workspace` file, and the bundled templates (schemas, skills, scripts, and default rules).

```sh
spex init
```

Finish setup afterwards by running the `/spex-setup` skill.

### `spex update`

Rewrites the bundled templates in the spex repository — skills, schemas, scripts, and rules (including their checks) — with the versions shipped in the installed `spex` binary. Run it from anywhere inside the workspace; spex locates the repository by walking up to the nearest `spex.yaml`.

Managed files are overwritten with the latest versions, while any additional skills, scripts, schemas, or rules you have added are left untouched. It also updates `cli_version` in `spex.yaml` to the installed binary's version. Use it after upgrading the `spex` binary to pull in framework updates; spex warns on every command when `cli_version` does not match the installed CLI.

```sh
spex update
```

### `spex check`

Runs the rule checks declared in `spex.yaml` against the workspace. Each workspace repository is checked against the rules listed in its `rules`, and every check's findings are reported per repository. Run it from anywhere inside the workspace; spex locates the config by walking up to the nearest `spex.yaml`.

```sh
spex check
```

The command exits with a non-zero status when any `error`-severity finding is reported; `warn`-severity findings do not affect the exit status.

A `script` check runs the program named by its `script:` filename (for example `writing-style.sh`), always resolved at `scripts/<rule-id-lowercase>/<script>` under the spex directory. Set `run: per-file` (the default) to pass the matched files as arguments. Set `run: per-repo` to run the script once per repository, in that repository's directory, with no arguments. Set `run: workspace` to run the script exactly once, with no arguments.

A failing `script` reports each violation on its own stdout line, following the script location contract: `<location><TAB><message>`, where `<location>` is a repository-relative file (optionally `<file>:<line>`) and `<message>` describes the violation. spex parses each line into a separate finding, so a script gets the same per-file precision as a `regexp` or `glob` check. A line with no location (empty before the tab, or no tab at all) becomes a file-less finding, and a script that exits non-zero without printing anything reports a single file-less finding built from its stderr or exit status. The exit code keeps its meaning: non-zero means the check failed.

A repository can also baseline pre-existing `error`-severity findings with `ignored_errors`, so a check can be enforced going forward without first fixing every existing violation. Each entry names the check by `rule_id` + `check`; an `error` finding from that check is dropped when its location file is listed in `files`, or always when `files` is omitted. New errors in other files still surface, and `warn`-severity findings are never affected:

```yaml
repositories:
  - repository_name: pepperoni-pizza-frontend
    repository_short_name: frontend
    path: ../pepperoni-pizza-frontend
    rules: [WRITING-STYLE]
    ignored_errors:
      - rule_id: WRITING-STYLE
        check: no-cyrillic
        files: [README.md, docs/intro.md]
```

#### Flags

- `--repo <name>...` — limit the run to the named workspace repositories, matched by either full `repository_name` or `repository_short_name`. Defaults to all repositories in `spex.yaml`.
- `--files <path>...` — limit the run to the given files (relative to each repository). A `per-repo` script then runs only in the repositories that contain a listed file, unioned with any `--repo` named; a `workspace` script always runs once. Defaults to all non-ignored files.
- `--fill-preexisting-errors` — record the current `error`-severity findings as an `ignored_errors` baseline in `spex.yaml`, then exit without gating. Use it to adopt a check on a repository that already has violations: the existing errors are baselined so they stop failing the run, while any new error still does.
- `--verbose` (`-v`) — stream progress to stderr as the run proceeds: each repository, then each check it runs (with the time each took), and the workspace checks at the end. The report on stdout is unchanged, so piping or redirecting it still works.

```sh
spex check --verbose
```

```sh
spex check --repo api frontend
```

```sh
spex check --repo api --files src/main.rs src/orders.rs
```

`--fill-preexisting-errors` runs every selected repository's checks, groups the resulting `error` findings by check and file, and writes one `ignored_errors` entry per check into that repository's block in `spex.yaml`. A finding tied to a file is baselined for that file; a file-less error (for example a per-repo script that inspects git history) is baselined as a whole-check entry that omits `files`. The command regenerates each selected repository's baseline from the current findings — dropping entries whose errors are gone — and leaves the rest of `spex.yaml`, and any unselected repository, byte-stable. It always exits `0`. Repository-level errors that belong to no named check (for example a misconfigured override) are not baselined; fix those directly.

```sh
spex check --repo frontend --fill-preexisting-errors
```

When the `spex` repository itself is in scope, `spex check` also validates it directly, without a rule: `spex.yaml`, every `specs/<spec-id>.yaml`, and every `rules/<rule-id>.yaml` must satisfy its JSON Schema; each spec file must be named `<spec-id-lowercase>.yaml` and each rule file `<rule-id-lowercase>.yaml`; every script a rule check references must exist at `scripts/<rule-id-lowercase>/<script>`; and each spec's `acceptance_criteria` must be ordered by RFC 2119 commitment level (`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`). It also checks that a `blocked_by` reference names an existing spec and is dropped once satisfied, and that a spec with every acceptance criterion checked off has its `status` set to `active` or `deprecated`.

### `spex coverage`

Reports how well the active specs are covered by spec-id annotations across the workspace. For every spec whose `status` is `active`, it requires that the code carries an implementation annotation (`@spec impl <spec-id>`), a test annotation (`@spec test <spec-id>`), and a contract annotation (`@spec contract <spec-id>`, or the OpenAPI `x-spec: <spec-id>` property). All three layers are required by default, though any of them — including `contract` — can be waived per spec via `coverage_ignore` (see below). A spec with no `status` field defaults to `planned`, so it is ignored. Run it from anywhere inside the workspace; spex locates the config by walking up to the nearest `spex.yaml`.

```sh
spex coverage
```

A single annotation of each kind anywhere in any workspace repository counts — the search is not scoped to the spec's `scope`. The `spex` repository itself is excluded from the search, and files ignored by `.gitignore` are skipped. Specs that are not `active` are listed by ID under their status and are otherwise ignored.

A spec can relax the requirement with the optional `coverage_ignore` field in its YAML, listing any of `contract`, `impl`, or `test`. A layer listed there is treated as ok even when its annotation is missing, so it never counts as a gap:

```yaml
id: PLACE-PIZZA-ORDER
status: active
coverage_ignore: [contract]
```

Each active spec's acceptance criteria carry an implementation checkbox — `[ ]` not implemented, `[x]` implemented — and the report shows how many are implemented as `done/total AC`. An active spec with any unimplemented criterion has a gap; `coverage_ignore` does not relax the checkboxes.

The report lists specs grouped by status (omitting any status with no specs), each spec ID shown under its status, then lists each active spec with its acceptance-criteria progress and coverage:

- `✓` — every required layer is annotated and every acceptance criterion is implemented.
- `⚠` — every required layer is present and every criterion is implemented, but an ignored layer is still missing.
- `✗` — a required layer is missing, or an acceptance criterion is unimplemented.

Each incomplete spec lists what is missing (for example `missing: test, contract (ignored)`). A missing layer listed in `coverage_ignore` is reported but never counts as a gap. The command exits with a non-zero status when any active spec has a gap (a missing required layer or an unimplemented acceptance criterion).

### `spex rules`

Prints, for every repository in `spex.yaml`, the full content of each rule listed in its `rules` — as YAML. Run it from anywhere inside the workspace; spex locates the config the same way `spex check` and `spex coverage` do.

```sh
spex rules
```

```yaml
- repository_name: pepperoni-pizza-api
  rules:
    - id: SECRET-HANDLING
      title: Secret protection
      # ...
```

#### Flags

- `--ai` — drop fields an agent doesn't need to follow a rule: `sync_with_upstream` and `checks`.
- `--id <id>...` — limit output to the given Rule IDs.
- `--repo <name>...` — limit output to the named repositories, matched by either `repository_name` or `repository_short_name`. Defaults to all repositories in `spex.yaml`; errors on an unrecognized name.

```sh
spex rules --ai --repo api
```

```sh
spex rules --id SECRET-HANDLING NAMING-CONVENTION
```

### `spex check-schema`

Validates JSON or YAML files against a JSON Schema. It loads the schema, resolves any cross-file `$ref`s against sibling schema files in the schema's own directory (fully offline, no network access), and validates each file. Violations are printed to stderr as `<file>: <message>`, and the command exits non-zero when any file fails.

This is a general-purpose validator. `spex check` uses the same validation, built in, to keep the spex repository's own spec files (`spex.yaml`, `specs/<spec-id-lowercase>.yaml`, `rules/<rule-id-lowercase>.yaml`) valid against the bundled schemas.

#### Flags

- `--schema <path>` — the JSON Schema the files are validated against (required).
- `--files <path>...` — one or more JSON or YAML files to validate against the schema (required).

```sh
spex check-schema --schema schemas/rule.schema.json --files rules/naming-convention.yaml
```

```sh
spex check-schema --schema schemas/spec.schema.json --files specs/list-pizza-menu.yaml specs/place-pizza-order.yaml
```

### `spex info`

Prints the resolved `spex.yaml` configuration, for scripts to consume instead of parsing `spex.yaml` by hand. Run it from anywhere inside the workspace; spex locates the config the same way `spex check` and `spex coverage` do.

`spex info` prints the fully-resolved configuration as pretty JSON, the general-purpose form scripts pipe into `jq` for whatever field they need:

```sh
spex info | jq -r '.repositories[].path'
```

`--print <field>` is a convenience for the common case of needing exactly one top-level scalar field, printing it bare with no quoting or JSON escaping:

```sh
spex info --print project_name
```

`--repo <name>` limits output to the general fields (`project_name`, `cli_version`) plus the named repository, matched by either `repository_name` or `repository_short_name`:

```sh
spex info --repo api
```

`--print` errors (non-zero exit, message to stderr) when the field does not exist or is not a scalar (for example `repositories`, which is a list — use plain `spex info` for that); `--repo` errors when no repository matches the given name.

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
