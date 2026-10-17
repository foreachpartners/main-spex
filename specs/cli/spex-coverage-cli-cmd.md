# `spex coverage`

Reports specs coverage for a project: how well the specs are covered by `@spec` annotations across the project, per acceptance criterion.

## Acceptance criteria

- MUST report, for every spec whose `status` is `active` or `in-progress`, its coverage across the project per acceptance criterion.
- MUST require a contract per the spec's `contract` field: `single` needs one spec-level `@spec contract <spec-id>` anywhere in any package's file text, including as the string value of an OpenAPI vendor extension such as `x-spec`; `for-each-ac` needs a `@spec contract <spec-id>#<ac-id>` annotation per acceptance criterion; `none` needs none.
- MUST require, for every acceptance criterion, `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>`.
- MUST decide whether a missing requirement is a gap or only pending from the criterion's `is_done` flag and its impl annotation, as laid out in [coverage-states-spec.md](../spec/coverage-states-spec.md).
- MUST waive a requirement of a spec when the spec's own `coverage_ignore` waives it, or when every package listed in the spec's `packages` waives it in its own `coverage_ignore`, declared in its `spex.yaml` entry or its `spex.package.yaml` ([spec-file-spec.md](../spec/spec-file-spec.md), [spex-yaml-config.md](../config/spex-yaml-config.md), [spex-package-yaml-config.md](../config/spex-package-yaml-config.md)).
- MUST skip the spex directory's own files in every layout, except the contract files declared in `spex.yaml`, and files ignored by `.gitignore`, as well as well-known dependency lock files.
- MUST skip the files and folders matched by a package's `exclude`, read as [spex check](spex-check-cli-cmd.md) reads it, so an annotation in them counts for no spec, except in the contract files declared in `spex.yaml`, which `exclude` does not affect.
- MUST error on an `exclude` pattern that is not a valid glob, naming the package.
- MUST exit non-zero when an `active` or `in-progress` spec has a gap.
- MUST NOT let a bare, unqualified `@spec impl|test <spec-id>` satisfy a criterion's requirement, since a single bare annotation would otherwise trivially satisfy every criterion in the spec.
- MUST NOT limit the annotation search to the spec's `packages`: a spec kept in the report by `--package` is still traced across every package.
- MUST NOT waive anything for a `coverage_ignore` entry naming a spec that does not exist.
- MUST NOT waive a requirement of a spec because of the `coverage_ignore` of a package the spec does not list in `packages`.
- MUST NOT skip a file because of the `exclude` of another package.
- MUST NOT waive a requirement of a spec that lists several packages until each of them waives it: a waiver in `api` alone changes nothing for a spec that lists `api` and `frontend`.

### Info

- MUST open the report with the installed CLI's version (e.g. `spex 0.10.0`, matching `spex --version`), followed by a blank line, so the output identifies the binary that produced it.

### Status report

- MUST list specs grouped by status, each group headed by its status and its spec count, omitting any status with no specs.
- MUST show under `active` only the count, listing no spec.
- MUST list each `planned`, `blocked` and `deprecated` spec by its id.
- MUST list under `in-progress` every spec, showing its id, its progress as `done/total AC`, and the criteria not implemented yet as `in-progress: impl(<ac-id>, ...)`, marked `✗` when it has a gap (also listed in "Gaps") and `●` otherwise.
- MUST count a criterion in the progress once its required impl annotation is present, whether or not its required test annotation is; a missing test annotation on an implemented criterion is reported as a gap instead.
- MUST ignore a spec with no `status` field, which defaults to `planned`; its ID is still listed under `planned`.

### Gaps

- MUST head the section with `Gaps (<count>)`, counting only the gapped specs.
- MUST list in it only the `active` or `in-progress` specs that have a gap (`✗`), grouped by `packages`, sorted alphabetically; a spec with several packages appears under each, and the header counts it once.
- MUST show, for each gapped spec, its id, its progress as `done/total AC`, and only what is missing, naming the acceptance criterion ids involved, e.g. `missing: contract, impl(no-null-typed-string), test(no-null-typed-string, empty-string-for-unset)`; under `contract: for-each-ac` the missing contract is named per criterion, e.g. `contract(ac-one, ac-two)`.
- MUST end the report with a one-line summary of spec counts per status and the gap count, e.g. `planned (5), in-progress (3), active (2), gaps (1)`.
- MUST NOT show pending work in "Gaps"; it is shown under `in-progress` in the status report.
- MUST NOT list a spec with no gap in "Gaps" at all.

### Flags

- MUST limit the report to the specs whose `packages` include a package named in `--package <name>...`, matched by either full `package_name` or `package_short_name`; it defaults to all specs.
- MUST error on a name given to `--package` that matches no package.
- MUST add description detail with `--verbose` (`-v`): each `planned` spec also shows its `description`, each `in-progress` spec also shows its `description` plus the text of each criterion not implemented yet next to its id, and each gapped spec in "Gaps" also shows its `description` plus, for every acceptance criterion still missing a required layer, that criterion's own text next to its id.

## Examples

```sh
spex coverage --package api frontend
spex coverage --verbose
```
