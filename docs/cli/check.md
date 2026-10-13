# `spex check`

Runs the checks declared in `spex.yaml` against each package, reporting findings per package.

- Opens its report with the installed CLI's version (e.g. `spex 0.10.0`, matching `spex --version`), followed by a blank line, so the output identifies the binary that produced it.
- Skips files ignored by `.gitignore`, even in a package without a `.git` directory, plus well-known dependency lock files (e.g. `Cargo.lock`, `package-lock.json`).
- Visits each package's files sorted by name within each directory, so a check's findings, and the files a `per-file` script receives, come out in the same order on every run and machine.
- Renders every finding with a location as a terminal hyperlink to the absolute file it points at, so it opens correctly whatever the shell's current working directory is; the displayed text stays the short package-relative path.
- Exits non-zero when any `error`-severity finding is reported; `warn`-severity findings do not affect the exit status.

## Script checks

A `script` check runs the program named by its `script:` filename (e.g. `writing-style.sh`), resolved in the rule's script folder: a folder named after the rule file, beside it (`rules/<rule-id>.yaml` has its scripts at `rules/<rule-id>/<script>`, and a nested `rules/backend/<rule-id>.yaml` at `rules/backend/<rule-id>/<script>`). The `run` mode controls the invocation:

- `run: per-file` (default) — pass the matched files as arguments.
- `run: per-package` — run once per package, in that package's directory, with no arguments.
- `run: workspace` — run exactly once, with no arguments.

Every script runs with the environment variable `SPEX_DIR` set to the absolute path of the spex directory, so it finds `spex.yaml` and the other spex files whatever its working directory. To read data from them it uses [`spex jq`](jq.md), e.g. `spex jq -r .project_name "$SPEX_DIR/spex.yaml"`.

Violations are reported on stdout, one per line, as `<location><TAB><message>`: `<location>` is a package-relative file, optionally `<file>:<line>`; `<message>` describes the violation. spex parses each line into a separate finding, so a script gets the same per-file precision as a `regexp` or `glob` check. A line with no location (empty before the tab, or no tab at all) becomes a file-less finding, and a script that exits non-zero without printing anything reports a single file-less finding built from its stderr or exit status. A non-zero exit code still means the check failed.

## Baselines

A package can baseline pre-existing `error`-severity findings with `ignored_errors`, so a check can be enforced going forward without first fixing every existing violation. Each entry names the check by `rule_id` + `check`; an `error` finding from that check is dropped when its location file is listed in `files`, or always when `files` is omitted. New errors in other files still surface, and `warn`-severity findings are never affected:

```yaml
packages:
  - package_name: shopping-notes-frontend
    package_short_name: frontend
    path: ../shopping-notes-frontend
    rules: [writing-style]
    ignored_errors:
      - rule_id: writing-style
        check: no-cyrillic
        files: [README.md, docs/intro.md]
```

A package can also keep its baseline in its own [`spex.package.yaml`](../specification.md#package-config-spexpackageyaml), where `ignored_errors` takes the same entries without the surrounding `packages` block. It is merged with that package's `ignored_errors` in `spex.yaml`, so an error is dropped when either lists it:

```yaml
ignored_errors:
  - rule_id: writing-style
    check: no-cyrillic
    files: [README.md, docs/intro.md]
```

## Flags

- `--package <name>...` — limit the run to the named packages, matched by either full `package_name` or `package_short_name`. Defaults to all packages in `spex.yaml`; errors on a name matching no package.
- `--files <path>...` — limit the run to the given files (relative to each package). A `per-package` script then runs only in the packages that contain a listed file, unioned with any `--package` named; a `workspace` script always runs once. Defaults to all non-ignored files; errors when a given file doesn't exist in any package. Negated `regexp`, `glob`, and `ast-grep` checks are skipped, because a negated check needs the full file set to prove a forbidden pattern is absent — a subset would only produce false positives.
- `--fill-preexisting-errors` — record the current `error`-severity findings as an `ignored_errors` baseline in `spex.yaml`, then exit without gating. Use it to adopt a check on a package that already has violations: existing errors are baselined so they stop failing the run, while any new error still does.

  It runs every selected package's checks, groups the resulting `error` findings by check and file, and writes one `ignored_errors` entry per check into that package's block. A finding tied to a file is baselined for that file; a file-less error (e.g. a per-package script that inspects git history) is baselined as a whole-check entry that omits `files`. Each selected package's baseline is regenerated from the current findings — dropping entries whose errors are gone — while the rest of `spex.yaml` and any unselected package stay byte-stable. It reads and writes only `spex.yaml`: a package's `spex.package.yaml` is never modified, and its entries are not subtracted from the generated baseline. It always exits `0`. Package-level errors that belong to no named check (e.g. a misconfigured override) are not baselined; fix those directly.

```sh
spex check --package api frontend
spex check --package api --files src/main.rs src/orders.rs
spex check --package frontend --fill-preexisting-errors
```

## Self-validation

`spex check` also validates the spex directory directly, without a rule. In multi-repo it is reported with the spex repository, whenever it is in scope. In single-repo and monorepo no package is the spex directory, so it runs on every `spex check`, whatever `--package` or `--files` select, and is reported last under its own `spex` heading. It validates:

- `spex.yaml`, every `specs/<spec-id>.yaml`, every `rules/<rule-id>.yaml`, and the `spex.package.yaml` of every package that has one must satisfy their JSON Schema.
- A spec id, acceptance criterion id, or rule id outside the id grammar (lowercase letters, digits and hyphens only, at most 40 characters) cannot be loaded: the command stops with a parse error naming the file instead of reporting a finding.
- Each spec file must be named `<spec-id>.yaml` and each rule file `<rule-id>.yaml`.
- Every script a rule check references must exist in the rule's script folder, beside the rule file.
- Declared properties must follow their JSON Schema's declared order, at every nesting level the schema describes (e.g. a rule's `checks[]` items).
- Each spec's and rule's `acceptance_criteria` must be ordered by RFC 2119 commitment level (`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`), and each criterion's `id` must be unique within its spec.
- No id may repeat across `specs/` and `rules/`.
- Every `@spec` annotation must match the annotation grammar, with its `<spec-id>` and `<ac-id>` following the id grammar, and reference an existing Spec ID — and, when it carries `#<ac-id>`, an existing acceptance criterion of that spec. Markdown files are skipped, as are the spex directory's `schemas/`, `specs/` and `rules/` (with each rule's script folder), which hold illustrative example annotations rather than real references.
- A `blocked_by` reference must name an existing spec — a rule id errors — and be dropped once satisfied: referencing an already-`active` spec warns.
- A `scope` entry must match a package's `package_short_name` in `spex.yaml`.
- A `coverage_ignore` entry, whether in the spec, in `spex.yaml`, or in a package's `spex.package.yaml`, must match the spec's `contract` setting: a contract-layer entry errors under `contract: none` (there is no contract coverage to waive) and, when it carries an `ac`, under `contract: single` (contracts are spec-level there, so per-criterion contracts only exist under `contract: for-each-ac`).
- A spec whose acceptance criteria are all done (`is_done` `true` or absent) but whose `status` is neither `active` nor `deprecated` warns; a spec with no acceptance criteria does not warn. A spec whose `status` is `active` with any acceptance criterion still `is_done: false` errors, since `active` means fully implemented (use `in-progress` until then).

This reads only each criterion's `is_done` field — whether that is actually backed by impl/test annotations is `spex coverage`'s job.
