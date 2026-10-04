# CLI

The `spex` command-line tool manages a spex workspace.

```sh
spex --help
```

## Command overview

| Command             | Purpose                                                                |
| ------------------- | ---------------------------------------------------------------------- |
| `spex init`         | Create the spex workspace repository                                   |
| `spex update`       | Refresh the bundled templates in the spex repository                   |
| `spex check`        | Run the rule checks declared in `spex.yaml`; fails on `error` findings |
| `spex check-schema` | Validate JSON or YAML files against a JSON Schema                      |
| `spex coverage`     | Report `@spec` annotation coverage per acceptance criterion            |
| `spex info`         | Print the resolved `spex.yaml` as JSON, for scripts                    |
| `spex rules`        | Print the rules governing the selected repositories, as YAML           |
| `spex specs`        | Print the workspace specs, as YAML                                     |

Every command except `init` finds the spex repository from the current directory — walking up and checking each directory it passes, plus its immediate subdirectories named `spex` or ending in `-spex`, for a `spex.yaml` — so any of them can run from anywhere inside the workspace.

## Install

### Prerequisites:

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/)
- **Git**: [Install Git](https://git-scm.com/install/)

### Install the `spex` binary from source.

```sh
git clone https://github.com/foreachpartners/spex
cd spex
cargo install --path . --force
```

## Commands

### `spex init`

Initializes the spex workspace. Run it from the workspace root or from an existing spex repository directory (`<project-name>-spex`), which is reused instead of created.

It prompts to confirm the project name (defaulting to the workspace root folder name), then writes:

- `spex.yaml`, seeded from the bundled template with the project name filled in and the installed CLI's version as `cli_version`. It lists only the spex repository (`<project-name>-spex`) and the rules that apply to it
- `<project-name>.code-workspace`, seeded from the bundled template with a folder for the spex repository only
- the bundled templates: schemas, docs, scripts, skills, and every bundled rule, plus empty `specs/`, `plans/`, and `contracts/` folders
- `.agents/skills/` with the bundled skills, and a `.claude` -> `.agents` symlink, so different agent tools detect the same skills
- a root `AGENTS.md` and a `CLAUDE.md` -> `AGENTS.md` symlink, seeded only when absent. A `# Project` marker splits `AGENTS.md`: content above it is managed and kept in sync by `spex update`, content from the marker onward is the user's own and is never touched

It does not look for, ask about, or configure the other workspace repositories.

```sh
spex init
```

Finish setup afterwards by running the `/spex-setup` skill: it adds the other workspace repositories to `spex.yaml` and the workspace file, and creates a skill for every workspace repository, including the spex repository.

### `spex update`

Rewrites the bundled templates in the spex repository — skills, schemas, docs, scripts, and rules (including their checks) — with the versions shipped in the installed `spex` binary.

- Rewrites a bundled skill already present locally only when its `metadata.sync-with-upstream` is not explicitly `false`; a skill that opts out keeps its local changes, so consumers can override a global skill.
- Rewrites a synced skill's `SKILL.md` above its `# Project` marker and the skill's other bundled files, preserving everything from that marker onward and any files you added to the skill folder; a local `SKILL.md` predating the marker is fully overwritten, since managed and user content cannot be told apart in it.
- Rewrites a bundled rule (and its scripts) already present locally only when it has `sync_with_upstream: true`; a local rule without it is left untouched, so consumers can override a global rule.
- Deploys a bundled rule missing locally, unless its id is listed in `rules_unused` in `spex.yaml`; listing an id there records a deliberate deletion, so it stays deleted on future updates.
- Rewrites the managed section of the root `AGENTS.md` (everything above the `# Project` marker), preserving everything from that marker onward; a local `AGENTS.md` predating the marker is fully overwritten, since managed and user content cannot be told apart in it.
- Overwrites managed files only; any skills, scripts, schemas, docs, or rules you added, and plans, are left untouched.
- Updates `cli_version` in `spex.yaml` to the installed binary's version.
- Prints the `CHANGELOG.md` entries newer than the previous `cli_version`, newest first; prints nothing if there are none, and every entry if the previous `cli_version` is not found in the changelog.
- Run it after upgrading the `spex` binary to pull in framework updates; `check`, `coverage` and `info` warn when `cli_version` does not match the installed CLI.

```sh
spex update
```

### `spex check`

Runs the checks declared in `spex.yaml` against each workspace repository, reporting findings per repository.

- Opens its report with the installed CLI's version (e.g. `spex 0.10.0`, matching `spex --version`), followed by a blank line, so the output identifies the binary that produced it.
- Skips files ignored by `.gitignore`, even in a repository without a `.git` directory, plus well-known dependency lock files (e.g. `Cargo.lock`, `package-lock.json`).
- Renders every finding with a location as a terminal hyperlink to the absolute file it points at, so it opens correctly whatever the shell's current working directory is; the displayed text stays the short repository-relative path.
- Exits non-zero when any `error`-severity finding is reported; `warn`-severity findings do not affect the exit status.

#### Script checks

A `script` check runs the program named by its `script:` filename (e.g. `writing-style.sh`), resolved at `scripts/<rule-id>/<script>` under the spex directory. The `run` mode controls the invocation:

- `run: per-file` (default) — pass the matched files as arguments.
- `run: per-repo` — run once per repository, in that repository's directory, with no arguments.
- `run: workspace` — run exactly once, with no arguments.

Violations are reported on stdout, one per line, as `<location><TAB><message>`: `<location>` is a repository-relative file, optionally `<file>:<line>`; `<message>` describes the violation. spex parses each line into a separate finding, so a script gets the same per-file precision as a `regexp` or `glob` check. A line with no location (empty before the tab, or no tab at all) becomes a file-less finding, and a script that exits non-zero without printing anything reports a single file-less finding built from its stderr or exit status. A non-zero exit code still means the check failed.

#### Baselines

A repository can baseline pre-existing `error`-severity findings with `ignored_errors`, so a check can be enforced going forward without first fixing every existing violation. Each entry names the check by `rule_id` + `check`; an `error` finding from that check is dropped when its location file is listed in `files`, or always when `files` is omitted. New errors in other files still surface, and `warn`-severity findings are never affected:

```yaml
repositories:
  - repository_name: shopping-notes-frontend
    repository_short_name: frontend
    path: ../shopping-notes-frontend
    rules: [writing-style]
    ignored_errors:
      - rule_id: writing-style
        check: no-cyrillic
        files: [README.md, docs/intro.md]
```

#### Flags

- `--repo <name>...` — limit the run to the named workspace repositories, matched by either full `repository_name` or `repository_short_name`. Defaults to all repositories in `spex.yaml`; errors on a name matching no repository.
- `--files <path>...` — limit the run to the given files (relative to each repository). A `per-repo` script then runs only in the repositories that contain a listed file, unioned with any `--repo` named; a `workspace` script always runs once. Defaults to all non-ignored files; errors when a given file doesn't exist in any workspace repository. Negated `regexp`, `glob`, and `ast-grep` checks are skipped, because a negated check needs the full file set to prove a forbidden pattern is absent — a subset would only produce false positives.
- `--fill-preexisting-errors` — record the current `error`-severity findings as an `ignored_errors` baseline in `spex.yaml`, then exit without gating. Use it to adopt a check on a repository that already has violations: existing errors are baselined so they stop failing the run, while any new error still does.

  It runs every selected repository's checks, groups the resulting `error` findings by check and file, and writes one `ignored_errors` entry per check into that repository's block. A finding tied to a file is baselined for that file; a file-less error (e.g. a per-repo script that inspects git history) is baselined as a whole-check entry that omits `files`. Each selected repository's baseline is regenerated from the current findings — dropping entries whose errors are gone — while the rest of `spex.yaml` and any unselected repository stay byte-stable. It always exits `0`. Repository-level errors that belong to no named check (e.g. a misconfigured override) are not baselined; fix those directly.

```sh
spex check --repo api frontend
spex check --repo api --files src/main.rs src/orders.rs
spex check --repo frontend --fill-preexisting-errors
```

#### Self-validation

When the `spex` repository itself is in scope, `spex check` also validates it directly, without a rule:

- `spex.yaml`, every `specs/<spec-id>.yaml`, and every `rules/<rule-id>.yaml` must satisfy their JSON Schema.
- Each spec file must be named `<spec-id>.yaml` and each rule file `<rule-id>.yaml`.
- Every script a rule check references must exist at `scripts/<rule-id>/<script>`.
- Declared properties must follow their JSON Schema's declared order, at every nesting level the schema describes (e.g. a rule's `checks[]` items).
- Each spec's and rule's `acceptance_criteria` must be ordered by RFC 2119 commitment level (`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`), and each criterion's `id` must be unique within its spec.
- No id may repeat across `specs/` and `rules/`.
- Every `@spec` annotation must match the annotation grammar and reference an existing Spec ID — and, when it carries `#<ac-id>`, an existing acceptance criterion of that spec. Markdown files are skipped, as are the spex directory's `schemas/` and the spex repository's `specs/`, `rules/`, and `scripts/`, which hold illustrative example annotations rather than real references.
- A `blocked_by` reference must name an existing spec and be dropped once satisfied — referencing an already-`active` spec, or any rule, warns, since rules always apply.
- A `scope` entry must match a repository's `repository_short_name` in `spex.yaml`.
- A `coverage_ignore` entry must match the spec's `contract` setting: a contract-layer entry errors under `contract: none` (there is no contract coverage to waive) and, when it carries an `ac`, under `contract: single` (contracts are spec-level there, so per-criterion contracts only exist under `contract: for-each-ac`).
- A spec whose acceptance criteria are all done (`is_done` `true` or absent) but whose `status` is neither `active` nor `deprecated` warns; a spec with no acceptance criteria does not warn. A spec whose `status` is `active` with any acceptance criterion still `is_done: false` errors, since `active` means fully implemented (use `in-progress` until then).

This reads only each criterion's `is_done` field — whether that is actually backed by impl/test annotations is `spex coverage`'s job.

### `spex check-schema`

Validates JSON or YAML files against a JSON Schema.

- Cross-file `$ref`s resolve against sibling schema files in the schema's own directory — fully offline, no network access.
- Violations are printed to stderr as `<file>: <message>`; the command exits non-zero when any file fails.
- `spex check` uses the same validation, built in, to keep the spex repository's own files (`spex.yaml`, `specs/<spec-id>.yaml`, `rules/<rule-id>.yaml`) valid against the bundled schemas.

#### Flags

- `--schema <path>` — the JSON Schema the files are validated against (required).
- `--files <path>...` — one or more JSON or YAML files to validate against the schema (required).

```sh
spex check-schema --schema schemas/rule.schema.json --files rules/naming-convention.yaml
spex check-schema --schema schemas/spec.schema.json --files specs/list-pizza-menu.yaml specs/place-pizza-order.yaml
```

### `spex coverage`

Reports how well the active and in-progress specs are covered by `@spec` annotations across the workspace, per acceptance criterion. The report opens with the installed CLI's version (e.g. `spex 0.10.0`, matching `spex --version`), followed by a blank line, so the output identifies the binary that produced it. Exits non-zero when an `active` spec has a gap; gaps on `in-progress` specs are reported but do not affect the exit code.

For every spec whose `status` is `active` or `in-progress`, coverage requires:

- **Contract** — per the spec's `contract` field:
  - `single` — one spec-level `@spec contract <spec-id>` anywhere in any workspace repository's file text, including as the string value of an OpenAPI vendor extension such as `x-spec`.
  - `for-each-ac` — a `@spec contract <spec-id>#<ac-id>` annotation per acceptance criterion.
  - `none` — none.
- **Impl and test** — for every acceptance criterion: `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>`.

A bare, unqualified `@spec impl|test <spec-id>` never satisfies a criterion's requirement, since a single bare annotation would otherwise trivially satisfy every criterion in the spec. A spec with no `status` field defaults to `planned` and is ignored (its ID is still listed under its status).

The annotation search is not scoped to the spec's `scope` — a spec kept in the report by `--scope` is still traced across every workspace repository. The spex repository's own files are skipped, except the contract files it declares, and files ignored by `.gitignore` are skipped, as are well-known dependency lock files.

#### Waivers

A spec can relax the requirements with the optional `coverage_ignore` field in its YAML. Each entry is `{layer, ac}`: `layer` is required; `ac` is optional — omitted, it waives that layer for the whole spec; given (an acceptance criterion id), it waives just that one criterion's impl or test requirement, or its contract requirement under `contract: for-each-ac`. The entry must match the spec's `contract` setting — whole-spec contract waives work under `single` and `for-each-ac`, per-criterion contract waives only under `for-each-ac`, and under `contract: none` a contract-layer entry is pointless; `spex check` errors on a mismatch:

```yaml
id: place-pizza-order
status: active
coverage_ignore:
  - layer: contract
  - layer: impl
    ac: no-null-typed-string
```

#### Report

- Lists specs grouped by status (omitting any status with no specs), each tracked spec showing its progress as `done/total AC` — a criterion counts once its required impl annotation is present, whether or not its required test annotation is; a missing test annotation is reported as a gap or as pending instead.
- The "Gaps" section lists only the active or in-progress specs that have a gap (`✗` — a required contract annotation missing: spec-level under `contract: single`, per criterion under `contract: for-each-ac`; or an acceptance criterion missing a required impl or test annotation while its `is_done` is `true` or absent), each with its progress and what's missing, naming the acceptance criterion ids involved (e.g. `missing: contract, impl(no-null-typed-string), test(no-null-typed-string, empty-string-for-unset)`; under `contract: for-each-ac` the missing contract is named per criterion, e.g. `contract(ac-one, ac-two)`). A gapped spec with pending criteria reports them on a separate line labeled `in-progress`.
- An acceptance criterion missing a required annotation while flagged `is_done: false` is pending, not a gap — expected on an in-progress spec still being worked on. A pending-only `in-progress` spec (`●`) is listed under its `in-progress` status group instead of "Gaps".
- The "Gaps" header counts the gapped specs only, and a spec with no gap is not listed at all. Gapped specs are grouped by `scope`, sorted alphabetically with unscoped specs last under `unscoped`; a spec with several scopes appears under each, and the header counts it once.
- The report ends with a one-line summary of spec counts per status and the gap count, e.g. `planned (5), in-progress (3), active (2), gaps (1)`.

#### Flags

- `--scope <repository-short-name>...` — limit the report to specs whose `scope` includes the given repository short name (`repository_short_name`). Defaults to all specs; errors on an unrecognized name.
- `--verbose` (`-v`) — add description detail: each `planned` spec listed under its status also shows its `description`, and each gapped spec in the "Gaps" section also shows its `description` plus, for every acceptance criterion still missing a required layer, that criterion's own text next to its id.

```sh
spex coverage --scope api frontend
spex coverage --verbose
```

### `spex info`

Prints the resolved `spex.yaml` configuration as pretty JSON — every declared field, descriptions included, with an absent `contracts`, `rules`, `ignored_errors`, or `rules_unused` printed as an empty list. It is the general-purpose form scripts pipe into `jq` for whatever field they need:

```sh
spex info | jq -r '.repositories[].path'
```

#### Flags

- `--print <field>` — print one top-level scalar field bare, with no quoting or JSON escaping. Errors (non-zero exit, message to stderr) when the field does not exist or is not a scalar (e.g. `repositories`, which is a list — use plain `spex info` for that).
- `--repo <name>...` — limit output to the general fields (`project_name`, `cli_version`) plus the named repositories, each matched by either `repository_name` or `repository_short_name`. Errors when any given name matches no repository.

```sh
spex info --print project_name
spex info --repo api frontend
```

### `spex rules`

Prints, for the selected repositories, the full content of each rule listed in their `rules`, as YAML — each rule once, however many repositories list it:

```yaml
- id: secret-handling
  title: Secret protection
  # ...
```

#### Flags

- `--ai` — drop fields an agent doesn't need to follow a rule (`sync_with_upstream` and `checks`), and add a `path` field after `id` with the rule file's location relative to the workspace root (e.g. `rules/secret-handling.yaml`).
- `--id <id>...` — limit output to the given rule IDs. Defaults to all rules; errors on an unrecognized ID.
- `--repo <name>...` — limit output to the named repositories, matched by either `repository_name` or `repository_short_name`. Defaults to all repositories in `spex.yaml`; errors on an unrecognized name.
- `--output <path>` — write the rendered YAML to this file instead of stdout, since the full rules list is large and can get truncated when printed to a terminal or piped through a shell.

```sh
spex rules --ai --repo api
spex rules --id secret-handling naming-convention
```

### `spex specs`

Prints the workspace specs as YAML:

```yaml
- id: place-pizza-order
  title: Place a pizza order
  # ...
```

#### Flags

- `--ai` — add a `path` field after `id` with the spec file's location relative to the workspace root (e.g. `specs/place-pizza-order.yaml`).
- `--id <id>...` — limit output to the given spec IDs. Defaults to all specs; errors on an unrecognized ID.
- `--scope <repository-short-name>...` — limit output to specs whose `scope` includes the given repository short name (`repository_short_name`). Defaults to all specs; errors on an unrecognized name.
- `--output <path>` — write the rendered YAML to this file instead of stdout, since the full specs list is large and can get truncated when printed to a terminal or piped through a shell.

```sh
spex specs --ai --scope api
spex specs --id place-pizza-order list-pizza-menu
```

## Development

These commands operate on the `spex` source checkout itself.

```sh
cargo build            # Debug build
cargo build --release  # Optimized build
cargo test             # Test
cargo check            # Type-check and lint
cargo fmt              # Format
dprint check           # Format check for md/yaml/json
```
