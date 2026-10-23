# `spex check`

Runs the rule checks declared in `spex.yaml` against each package, reporting findings per package, and the framework checks that validate the spex directory itself, without a rule (self-validation).

## Acceptance criteria

### Run and report

- MUST run the checks declared in `spex.yaml` against each package, reporting findings per package.
- MUST open its report with the installed CLI's version (e.g. `spex 0.10.0`, matching `spex --version`), followed by a blank line, so the output identifies the binary that produced it.
- MUST skip files ignored by `.gitignore`, even in a package without a `.git` directory, plus well-known dependency lock files (e.g. `Cargo.lock`, `package-lock.json`).
- MUST skip the files and folders matched by a package's `exclude` ([spex.yaml](../config/spex-yaml-config.md), merged with its [`spex.package.yaml`](../config/spex-package-yaml-config.md)) the same way, so no check, including the `@spec` annotation checks, reports a finding in them and no `per-file` script receives them.
- MUST match each `exclude` pattern, a glob read as in a check's `exclude`, against the path of every file and folder relative to its package's root, and skip a folder that matches with everything inside it: `vendor` or `**/vendor` skips a folder, `vendor/**` the files inside it, and `**/*.generated.ts` such files at any depth.
- MUST error on an `exclude` pattern that is not a valid glob, naming the package.
- MUST NOT skip a file because of the `exclude` of another package.
- MUST visit each package's files sorted by name within each directory, so a check's findings, and the files a `per-file` script receives, come out in the same order on every run and machine.
- MUST render every finding with a location as a terminal hyperlink to the absolute file it points at, so it opens correctly whatever the shell's current working directory is; the displayed text stays the short package-relative path.
- MUST exit non-zero when any `error`-severity finding is reported.
- MUST NOT let `warn`-severity findings affect the exit status.

### Flags

- MUST limit the run to the packages named in `--package <name>...`, matched by either full `package_name` or `package_short_name`; it defaults to all packages in `spex.yaml`.
- MUST error on a name given to `--package` that matches no package.
- MUST limit the run to the files given in `--files <path>...`, relative to each package; it defaults to all files that are neither ignored nor excluded.
- MUST skip, with `--files`, a listed file that the `exclude` of its package matches, without an error, and MUST NOT select a package for such a file alone.
- MUST run a `per-package` script, with `--files`, only in the packages that contain a listed file, unioned with any `--package` named.
- MUST run a `project` script exactly once, whatever `--files` selects.
- MUST error when a file given to `--files` does not exist in any package.
- MUST skip negated `regexp`, `glob`, and `ast-grep` checks with `--files`, because a negated check needs the full file set to prove a forbidden pattern is absent; a subset would only produce false positives.
- MUST record, with `--fill-preexisting-errors`, the current `error`-severity findings as a `check_ignore` baseline in `spex.yaml`, or in the `spex.package.yaml` of each selected package when `--package` is given, then exit without gating, so a check can be adopted on a package that already has violations: existing errors are baselined so they stop failing the run, while any new error still does.
- MUST run, with `--fill-preexisting-errors`, every selected package's checks, group the resulting `error` findings by check and file, and write one `check_ignore` entry per check into that package's block of `spex.yaml`, or, when `--package` is given, into the top-level `check_ignore` of that package's `spex.package.yaml`, creating the file when it is missing and there is something to write.
- MUST baseline, with `--fill-preexisting-errors`, a finding tied to a file for that file, and a file-less error (e.g. a per-package script that inspects git history) as a whole-check entry that omits `files`.
- MUST regenerate, with `--fill-preexisting-errors`, each selected package's baseline from the current findings, dropping entries whose errors are gone, while the rest of the written file and any unselected package stay byte-stable.
- MUST leave out of the generated baseline, with `--fill-preexisting-errors`, an error that the other file already baselines for that package, so no error is listed in both: its `spex.package.yaml` when `spex.yaml` is written, its entry in `spex.yaml` when `--package` is given.
- MUST read and write only `spex.yaml` with `--fill-preexisting-errors`, or, when `--package` is given, only the `spex.package.yaml` of each selected package.
- MUST always exit `0` with `--fill-preexisting-errors`.
- MUST NOT baseline, with `--fill-preexisting-errors`, package-level errors that belong to no named check (e.g. a misconfigured override); they are fixed directly.

### Script checks

- MUST run the program named by a `script` check's `script:` filename (e.g. `writing-style.sh`), resolved in the rule's script folder: a folder named after the rule file, beside it (`rules/<rule-id>.yaml` has its scripts at `rules/<rule-id>/<script>`, and a nested `rules/backend/<rule-id>.yaml` at `rules/backend/<rule-id>/<script>`).
- MUST pass the matched files as arguments with `run: per-file` (the default).
- MUST run once per package, in that package's directory, with no arguments, with `run: per-package`.
- MUST run exactly once, with no arguments, with `run: project`.
- MUST run every script with the environment variable `SPEX_DIR` set to the absolute path of the spex directory, so it finds `spex.yaml` and the other spex files whatever its working directory; to read data from them it uses [`spex jq`](spex-jq-cli-cmd.md), e.g. `spex jq -r .project_name "$SPEX_DIR/spex.yaml"`.
- MUST read violations from a script's stdout, one per line, as `<location><TAB><message>`: `<location>` is a package-relative file, optionally `<file>:<line>`; `<message>` describes the violation.
- MUST parse each such line into a separate finding, so a script gets the same per-file precision as a `regexp` or `glob` check.
- MUST turn a line with no location (empty before the tab, or no tab at all) into a file-less finding.
- MUST report a single file-less finding built from stderr or the exit status when a script exits non-zero without printing anything.
- MUST treat a non-zero exit code as the check failed.

### Baselines

- MUST let a package baseline pre-existing `error`-severity findings with `check_ignore`, so a check can be enforced going forward without first fixing every existing violation.
- MUST name the check in each `check_ignore` entry by `rule_id` + `check`.
- MUST drop an `error` finding from that check when its location file is listed in `files`, or always when `files` is omitted.
- MUST merge the `check_ignore` of a package's own [`spex.package.yaml`](../config/spex-package-yaml-config.md) with its `check_ignore` in `spex.yaml`, so an error is dropped when either lists it; the entries there are the same, without the surrounding `packages` block.
- MUST NOT drop new errors in other files.
- MUST NOT drop an error because of the `check_ignore` of another package.
- MUST NOT affect `warn`-severity findings with a baseline.

### Self-validation

- MUST run `spex-internal-conformance` and the spec-scoped checks (reported as `<spec-id>/<check>`) on every `spex check`, whatever `--package` or `--files` select, since they validate the spex directory as a whole, and report them under the spex package's heading, which is added when that package is not selected.
- MUST run the `@spec` annotation checks (`annotation-format`, `no-orphan-annotations`) only on the files of the selected packages: `--package api` checks the annotations in `api`, and `--package spex` does not check them in `api`.
- MUST validate `spex.yaml`, every `specs/<spec-id>.yaml`, every `rules/<rule-id>.yaml`, and the `spex.package.yaml` of every package that has one against their JSON Schema.
- MUST require every file `spex.yaml` declares to exist: the `prd` file and each `contracts` entry's `path`, erroring on every missing one with its path as written relative to the spex directory, and, for a contract, the package that declares it.
- MUST stop with a parse error naming the file, instead of reporting a finding, when a spec id, acceptance criterion id, or rule id is outside the id allowed schema.
- MUST require each spec file to be named `<spec-id>.yaml` and each rule file `<rule-id>.yaml`.
- MUST require every script a rule check references to exist in the rule's script folder, beside the rule file.
- MUST require declared properties to follow their JSON Schema's declared order, at every nesting level the schema describes (e.g. a rule's `checks[]` items).
- MUST require each spec's and rule's `acceptance_criteria` to be ordered by RFC 2119 commitment level (`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`), and each criterion's `id` to be unique within its spec.
- MUST require every `@spec` annotation to match the annotation grammar, with its `<spec-id>` and `<ac-id>` following the id grammar, and to reference an existing Spec ID, and, when it carries `#<ac-id>`, an existing acceptance criterion of that spec.
- MUST skip, for annotation validation, the spex directory's `schemas/`, `specs/`, `rules/` and `docs/` and the agent root's `.agents/skills/` with all their subfolders, and the agent root's `AGENTS.md`, since they hold illustrative example annotations rather than real references.
- MUST require every `blocked_by` entry to be the id of an existing spec, and error on anything else, such as a rule id or an unknown id.
- MUST warn when a `blocked_by` reference names an already-`active` spec, since a satisfied blocker is dropped.
- MUST require a `packages` entry to match a package's `package_short_name` in `spex.yaml`.
- MUST require a `coverage_ignore` entry, whether in the spec, in a package entry of `spex.yaml`, or in a package's `spex.package.yaml`, to match the spec's `contract` setting: a contract-layer entry errors under `contract: none` (there is no contract coverage to waive) and, when it carries an `ac`, under `contract: single` (contracts are spec-level there, so per-criterion contracts only exist under `contract: for-each-ac`).
- MUST error on a `coverage_ignore` entry of a package, in its `spex.yaml` entry or its `spex.package.yaml`, that names a spec whose `packages` does not list that package, since it could never waive anything.
- MUST warn on a spec whose acceptance criteria are all done (`is_done` `true` or absent) but whose `status` is neither `active` nor `deprecated`.
- MUST error on a spec whose `status` is `active` with any acceptance criterion still `is_done: false`, since `active` means fully implemented (`in-progress` applies until then).
- MUST read only each criterion's `is_done` field for status checks; whether it is backed by impl/test annotations is `spex coverage`'s job.
- MUST NOT allow an spec-id to repeat across `specs/` and `rules/`.

## Examples

A package baselines pre-existing errors in `spex.yaml`:

```yaml
packages:
  - package_name: shopping-notes-frontend
    package_short_name: frontend
    path: ../shopping-notes-frontend
    rules: [writing-style]
    check_ignore:
      - rule_id: writing-style
        check: no-cyrillic
        files: [README.md, docs/intro.md]
```

or in its own `spex.package.yaml`:

```yaml
check_ignore:
  - rule_id: writing-style
    check: no-cyrillic
    files: [README.md, docs/intro.md]
```

```sh
spex check --package api frontend
spex check --package api --files src/main.rs src/orders.rs
spex check --fill-preexisting-errors
spex check --package frontend --fill-preexisting-errors
```
