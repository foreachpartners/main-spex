# Changelog

Notable changes to spex, following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions match `Cargo.toml` (`cli_version` in `spex.yaml`). `spex update` prints the entries newer than the version it's updating from.

## [0.10.1] - 2026-09-30

- `init`: now only creates the spex repository. It seeds `spex.yaml` and `<project-name>.code-workspace` from bundled templates with the project name filled in (both list only the spex repository), and no longer discovers, confirms, or configures other workspace repositories or seeds their stub skills.
- `spex-setup` skill: now adds the other workspace repositories to `spex.yaml` and the workspace file, and creates every repository skill, including the spex repository's. This unlocks a flow where we start with spex only and add other repositories later.

## [0.10.0] - 2026-09-20

- **Breaking**: specs now have a required `contract` field (`none`, `single`, or `for-each-ac`) setting the contract coverage granularity. `single` requires a single `@spec contract <spec-id>` annotation for the whole spec (the previous behavior); `for-each-ac` requires a `@spec contract <spec-id>#<ac-id>` annotation per acceptance criterion; `none` disables contract coverage. Existing specs must add `contract: single` (see the migration guide).
- `coverage`: under `contract: for-each-ac`, contract coverage is now per acceptance criterion — a missing per-criterion contract annotation is reported as a gap (`contract(ac-one, ac-two)`), and a contract annotation anchored to one criterion no longer satisfies the others.
- `coverage_ignore`: a contract-layer entry may now carry `ac` to waive one criterion's contract under `contract: for-each-ac`; previously any `ac` was invalid with `layer: contract`.
- `check`: now verifies every `coverage_ignore` entry matches the spec's `contract` setting — a contract-layer entry errors under `contract: none` (nothing to waive), and a per-criterion contract entry errors under `contract: single` (contracts are spec-level there).

## [0.9.1] - 2026-09-20

- Added the `spex-update` skill: to automatically update spex based on migration guide using AI.

## [0.9.0] - 2026-09-20

- **Breaking:** removed the separate `x-spec` annotation grammar; contract coverage now uses `@spec contract <spec-id>`, matched anywhere in file text including as any `x-*` vendor extension's string value — the extension key name is no longer significant. Existing `x-spec: <spec-id>` annotations must be rewritten as `x-spec: "@spec contract <spec-id>"` (or any other `x-*` key).

## [0.8.9] - 2026-09-19

- `coverage`: spec ids are now clickable terminal hyperlinks (OSC 8) to their spec file — in the status totals and in every "Gaps"/pending row; nothing changes when output is not a terminal.

## [0.8.8] - 2026-09-17

- `coverage`: an acceptance criterion flagged `is_done: false` and still missing a required impl or test annotation is now reported as pending, not a gap — it no longer counts toward the exit code or the "Gaps" section, and is instead listed (●, blue, "in-progress: ...") under its spec's `in-progress` status group.
- `coverage`: the `X/Y AC` counter now tracks implementation only — a criterion counts as done once its impl annotation is present, regardless of whether its test annotation is; a still-missing test annotation is reported separately as a gap or as pending.
- `coverage`: the status summary and totals now list `planned` before `in-progress`, matching a spec's natural lifecycle order.
- `coverage`: improve `--verbose` coverage report.

## [0.8.7] - 2026-09-17

- `coverage`: long text — a gapped row's "missing: ..." detail, `--verbose` spec/acceptance-criterion descriptions — now wraps to the terminal width with continuation lines indented to align, instead of relying on the terminal's own wrap and losing alignment with the row.
- `coverage --verbose`: each planned spec's and each gapped spec's detail is now followed by a blank line, separating it from the next entry.

## [0.8.6] - 2026-09-17

- `cli`: `--verbose` is no longer a global flag; it's now local to the subcommands that use it.
- `coverage`: added `--verbose` — each listed `planned` spec additionally shows its `description`, and each gapped spec in the "Gaps" section additionally shows its `description` plus the text of each acceptance criterion still missing a required layer.
- `check`: removed `--verbose` and the stderr progress logging it drove; `spex check` was the only user of the shared `Log` helper, which is also removed.

## [0.8.5] - 2026-09-16

- `ac-id`/`spec-id`/`rule-id` schemas: update id naming guidance now emphasizes readability.
- `AGENTS.md` template: added `Skill` and `Traceability` to the core-concepts list, and an example annotation pair to the "Definition of done" tracing bullet to reflect their bigger role in framework.

## [0.8.4] - 2026-09-16

- `check`: spec-id annotation checks (malformed `@spec`/`x-spec` grammar, orphan Spec ID references) are now built into `spex check` itself reflecting it's more important role — replacing the removed `spec-id-tracing` rule and its scripts.
- `check`: the orphan annotation check now also flags a `#<ac-id>` suffix that names no acceptance criterion of its spec, and no longer misses an unknown Spec ID that shares a prefix with a known one (e.g. `delivery-time2` silently matching `delivery-time`).

## [0.8.3] - 2026-09-15

- `AGENTS.md` template: added a `# Project` marker splitting the file into a managed section and a user-owned section. `update` now overwrites the managed section with the bundled template while preserving everything from the marker onward; a local `AGENTS.md` predating the marker is fully overwritten instead.

## [0.8.2] - 2026-09-15

- `coverage`: the "Gaps" section is now grouped by `scope`, sorted alphabetically with unscoped specs listed last; a spec with more than one scope is duplicated under each of its scope groups.
- `coverage`: added `--scope <name>...`, limiting the report to specs whose `scope` includes one of the given repository short names; errors on an unrecognized name.

## [0.8.1] - 2026-09-14

- `init`, `update`: now scaffold a bundled `docs/` folder (`glossary.md`, `framework-principles.md`, `cli.md`) into the spex repository, always overwritten on `update` like `schemas/`.
- `AGENTS.md` template: replaced the inline `spex commands` list with a link to `docs/cli.md`, and added a `## Docs` index linking `docs/glossary.md`, `docs/framework-principles.md`, and `docs/cli.md`.

## [0.8.0] - 2026-09-14

- **Breaking**: `acceptance_criteria` is now a list of `{id, must|must_not|should|should_not|may, is_done}` objects instead of checkbox-prefixed strings. `id` is a kebab-case slug, unique within the spec, naming the criterion's behavior rather than its position — immutable once an annotation references it. Exactly one of `must`, `must_not`, `should`, `should_not`, `may` is present, its value the criterion text. `is_done` is optional (absent counts as `true`); a newly authored criterion is written `is_done: false` and cleared once done. Existing specs must migrate.
- **Breaking**: `@spec impl|test <spec-id>` annotations may now carry the acceptance criterion they satisfy as `<spec-id>#<ac-id>`.
- **Breaking**: `coverage_ignore` entries are now always `{layer, ac}` objects; `ac` is optional (waives the whole spec's layer when absent) and invalid with `layer: contract`. The old bare layer-name string form (`- contract`) no longer parses.
- `coverage`: now requires an impl and a test annotation per acceptance criterion, each qualified with `#<ac-id>` — a bare, unqualified annotation no longer satisfies any criterion's requirement, closing the gap where a single annotation anywhere trivially passed every criterion.
- `check`: a spec's `active`/`in-progress` status validation now reads each acceptance criterion's `is_done` field instead of impl+test annotation coverage or the old `[ ]`/`[x]` checkbox — `spex check` validates spec correctness, `spex coverage` proves `is_done` is actually backed by annotations; also errors on a duplicate acceptance criterion id within one spec.
- `spex-test` now finalizes specs it tests: clears `is_done: false` on criteria `spex coverage` shows fully impl+test traced, and sets `status: active` once every criterion is done — `spex-implement`'s finalize step no longer sets `active`, only `in-progress`.
- `spec-id-tracing` rule, and the `spex-spec`/`spex-test`/`spex-implement` skills: updated for the `id`/`must`|`must_not`|`should`|`should_not`|`may`/`is_done` acceptance-criteria format and `#<ac-id>` annotation extension.

## [0.7.95] - 2026-09-13

- `check`: `--repo` help text now reads "to include", matching `rules`/`specs`/`info`.
- `init`, `update`: success messages now use the shared terminal styling, matching every other command.
- `check`, `coverage`: both now return `ExitCode::FAILURE` on a failing report, instead of mixing it with `ExitCode::from(1)`.
- `docs/cli.md`: command order now matches `--help` order (`check-schema` and `info` were out of sequence).
- the CLI now warns on stderr when `--verbose` is passed to a command other than `check`, where it has no effect.

## [0.7.94] - 2026-09-13

- `info`, `specs`: reworded "unknown repository" error messages to match `check`/`rules`' style (`unknown ...`{name}`— not listed in spex.yaml`).

## [0.7.93] - 2026-09-13

- `check`: `--files` now errors when a given file doesn't exist in any workspace repository, instead of silently dropping it, matching `check-schema --files`.

## [0.7.92] - 2026-09-13

- `rules`, `specs`: `--id` now errors on an unrecognized ID instead of silently returning no output, matching how `--repo`/`--scope` already behave.

## [0.7.91] - 2026-09-13

- `commit-message-format` (`commit-spec-id-trace`): trigger the `@spec` ID check off the commit's Conventional Commits type (only `feat` requires it) instead of the diff content, avoiding false positives.

## [0.7.90] - 2026-09-13

- `check`: add a built-in `property-order` conformance check that errors when a spec file, rule file, or `spex.yaml` declares its top-level properties out of order relative to their JSON Schema.

## [0.7.89] - 2026-09-12

- `spex-review-rule`: improve review skill to avoid unnecessary abstractions and give clear main review goal.
- replace `→` with `->` for better tokenization.

## [0.7.88] - 2026-09-10

- `coverage`: only exit non-zero for gaps on `active` specs; gaps on `in-progress` specs are still reported but no longer fail the command.

## [0.7.87] - 2026-09-10

- `coverage`: rename the "Tracked spec coverage" section to "Gaps", and list only tracked specs that actually have a gap (fully covered specs, and ones missing only an ignored layer, are counted in the section header instead of listed individually).
- `coverage`: drop the `spex coverage · <project>` header line and the trailing `<project>: N gaps`/`no gaps` line, replacing them with a one-line summary of spec counts per status and the gap count, e.g. `in-progress (3), planned (5), active (2), gaps (1)`.

## [0.7.86] - 2026-09-10

- Add `in-progress` spec `status`, for a spec being implemented but not yet complete.
- `check`: error when a spec's `status` is `active` but not every acceptance criterion is checked off — `active` now means fully implemented; use `in-progress` until then.
- `coverage`: track and report `in-progress` specs alongside `active` ones, renaming the "Active spec coverage" section to "Tracked spec coverage".

## [0.7.85] - 2026-09-10

- `check`: fix the `status`, `blocked_by` and `scope` findings' location to point at the spec's actual file, honoring specs nested in subdirectories under `specs/` instead of assuming `specs/<id>.yaml` — the wrong, non-existent path made the finding's hyperlink dead.
- `check`: fix the conformance `missing script` finding's location the same way for rules nested under `rules/`.
- `check`: canonicalize a finding's location hyperlink so it never contains a `/./` segment (e.g. for the spex repository itself, whose `path` is `.`), which some terminals refused to recognize as a clickable link.

## [0.7.84] - 2026-09-08

- `check`: render each finding's location as a terminal hyperlink to the absolute file, so it opens correctly regardless of the shell's current directory, while still displaying the short repository-relative path.

## [0.7.83] - 2026-09-06

- `commit-message-format` rule: replace the `spec`/`rule` scope restriction with free-form scope plus `@spec`/`@rule` ID tags on spec/rule-related commits.

## [0.7.82] - 2026-09-06

- `spex-spec` skill: for a multi-repository spec, name each repository in its acceptance criteria; omit the repository name for a single-scoped spec since it's already redundant with `scope`.
