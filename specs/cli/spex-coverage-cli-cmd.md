# `spex coverage`

Reports how well the active and in-progress specs are covered by `@spec` annotations across the project, per acceptance criterion.

## Acceptance criteria

### Coverage

- MUST report, for every spec whose `status` is `active` or `in-progress`, its coverage across the project per acceptance criterion.
- MUST open the report with the installed CLI's version (e.g. `spex 0.10.0`, matching `spex --version`), followed by a blank line, so the output identifies the binary that produced it.
- MUST exit non-zero when an `active` or `in-progress` spec has a gap.
- MUST require a contract per the spec's `contract` field: `single` needs one spec-level `@spec contract <spec-id>` anywhere in any package's file text, including as the string value of an OpenAPI vendor extension such as `x-spec`; `for-each-ac` needs a `@spec contract <spec-id>#<ac-id>` annotation per acceptance criterion; `none` needs none.
- MUST require, for every acceptance criterion, `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>`.
- MUST decide whether a missing requirement is a gap or only pending from the criterion's `is_done` flag and its impl annotation, as laid out in [coverage-states-spec.md](../spec/coverage-states-spec.md).
- MUST ignore a spec with no `status` field, which defaults to `planned`; its ID is still listed under its status.
- MUST apply every waiver naming a spec, from the spec's own `coverage_ignore`, from `spex.yaml`, and from each package's `spex.package.yaml`, regardless of the packages in its `scope` ([spec-file-spec.md](../spec/spec-file-spec.md), [spex-package-yaml-config.md](../config/spex-package-yaml-config.md)).
- MUST skip the spex directory's own files in every layout, except the contract files declared in `spex.yaml`, and files ignored by `.gitignore`, as well as well-known dependency lock files.
- MUST NOT let a bare, unqualified `@spec impl|test <spec-id>` satisfy a criterion's requirement, since a single bare annotation would otherwise trivially satisfy every criterion in the spec.
- MUST NOT scope the annotation search to the spec's `scope`: a spec kept in the report by `--scope` is still traced across every package.
- MUST NOT waive anything for a `coverage_ignore` entry naming a spec that does not exist.

### Report

- MUST list specs grouped by status, omitting any status with no specs, each tracked spec showing its progress as `done/total AC`.
- MUST count a criterion in the progress once its required impl annotation is present, whether or not its required test annotation is; a missing test annotation on an implemented criterion is reported as a gap instead.
- MUST list in the "Gaps" section only the active or in-progress specs that have a gap (`✗`): a required contract annotation missing (spec-level under `contract: single`, per criterion under `contract: for-each-ac`), or an acceptance criterion missing a required impl or test annotation.
- MUST show, for each gapped spec in "Gaps", its progress and what is missing, naming the acceptance criterion ids involved, e.g. `missing: contract, impl(no-null-typed-string), test(no-null-typed-string, empty-string-for-unset)`; under `contract: for-each-ac` the missing contract is named per criterion, e.g. `contract(ac-one, ac-two)`.
- MUST report a gapped spec's pending criteria on a separate line labeled `in-progress`.
- MUST treat a missing requirement as pending, not a gap, only while the acceptance criterion is flagged `is_done: false` and has no impl annotation yet, which is expected on an in-progress spec still being worked on; the spec-level contract under `contract: single` is pending too while every criterion is.
- MUST list a pending-only `in-progress` spec (`●`) under its `in-progress` status group instead of "Gaps".
- MUST count only the gapped specs in the "Gaps" header.
- MUST group gapped specs by `scope`, sorted alphabetically with unscoped specs last under `unscoped`; a spec with several scopes appears under each, and the header counts it once.
- MUST end the report with a one-line summary of spec counts per status and the gap count, e.g. `planned (5), in-progress (3), active (2), gaps (1)`.
- MUST NOT list a spec with no gap in "Gaps" at all.

### Flags

- MUST limit the report to the specs whose `scope` includes a package short name given in `--scope <package-short-name>...` (`package_short_name`); it defaults to all specs.
- MUST error on an unrecognized name given to `--scope`.
- MUST add description detail with `--verbose` (`-v`): each `planned` spec listed under its status also shows its `description`, and each gapped spec in "Gaps" also shows its `description` plus, for every acceptance criterion still missing a required layer, that criterion's own text next to its id.

## Examples

```sh
spex coverage --scope api frontend
spex coverage --verbose
```
