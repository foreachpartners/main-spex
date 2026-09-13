# Changelog

Notable changes to spex, following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions match `Cargo.toml` (`cli_version` in `spex.yaml`). `spex update` prints the entries newer than the version it's updating from.

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
