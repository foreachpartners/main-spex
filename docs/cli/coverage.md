# `spex coverage`

Reports how well the active and in-progress specs are covered by `@spec` annotations across the workspace, per acceptance criterion. The report opens with the installed CLI's version (e.g. `spex 0.10.0`, matching `spex --version`), followed by a blank line, so the output identifies the binary that produced it. Exits non-zero when an `active` or `in-progress` spec has a gap.

For every spec whose `status` is `active` or `in-progress`, coverage requires:

- **Contract** — per the spec's `contract` field:
  - `single` — one spec-level `@spec contract <spec-id>` anywhere in any workspace repository's file text, including as the string value of an OpenAPI vendor extension such as `x-spec`.
  - `for-each-ac` — a `@spec contract <spec-id>#<ac-id>` annotation per acceptance criterion.
  - `none` — none.
- **Impl and test** — for every acceptance criterion: `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>`.

Whether a missing requirement is a gap or only pending depends on the criterion's `is_done` flag and its impl annotation, as laid out in [Coverage states](../specification.md#coverage-states).

A bare, unqualified `@spec impl|test <spec-id>` never satisfies a criterion's requirement, since a single bare annotation would otherwise trivially satisfy every criterion in the spec. A spec with no `status` field defaults to `planned` and is ignored (its ID is still listed under its status).

The annotation search is not scoped to the spec's `scope` — a spec kept in the report by `--scope` is still traced across every workspace repository. The spex repository's own files are skipped, except the contract files it declares, and files ignored by `.gitignore` are skipped, as are well-known dependency lock files.

## Waivers

A spec can relax the requirements with the optional `coverage_ignore` field in its YAML, and `spex.yaml` and each repository's `spex.repo.yaml` can add waivers by spec id. Each entry is `{layer, ac}`: `layer` is required; `ac` is optional — omitted, it waives that layer for the whole spec; given (an acceptance criterion id), it waives just that one criterion's impl or test requirement, or its contract requirement under `contract: for-each-ac`. The entry must match the spec's `contract` setting — whole-spec contract waives work under `single` and `for-each-ac`, per-criterion contract waives only under `for-each-ac`, and under `contract: none` a contract-layer entry is pointless; `spex check` errors on a mismatch:

```yaml
id: place-pizza-order
status: active
coverage_ignore:
  - layer: contract
  - layer: impl
    ac: no-null-typed-string
```

The root `coverage_ignore` of `spex.yaml`, and that of every workspace repository's [`spex.repo.yaml`](../specification.md#repository-config-spexrepoyaml), take the same entries plus the `spec_id` they apply to. A spec's waivers are the union of its own `coverage_ignore` and every entry naming its id, wherever declared, regardless of the repositories in its `scope`. An entry naming a spec that does not exist waives nothing:

```yaml
coverage_ignore:
  - spec_id: place-pizza-order
    layer: contract
  - spec_id: place-pizza-order
    layer: impl
    ac: no-null-typed-string
```

## Report

- Lists specs grouped by status (omitting any status with no specs), each tracked spec showing its progress as `done/total AC` — a criterion counts once its required impl annotation is present, whether or not its required test annotation is; a missing test annotation on an implemented criterion is reported as a gap instead.
- The "Gaps" section lists only the active or in-progress specs that have a gap (`✗` — a required contract annotation missing: spec-level under `contract: single`, per criterion under `contract: for-each-ac`; or an acceptance criterion missing a required impl or test annotation), each with its progress and what's missing, naming the acceptance criterion ids involved (e.g. `missing: contract, impl(no-null-typed-string), test(no-null-typed-string, empty-string-for-unset)`; under `contract: for-each-ac` the missing contract is named per criterion, e.g. `contract(ac-one, ac-two)`). A gapped spec with pending criteria reports them on a separate line labeled `in-progress`.
- A missing requirement is pending, not a gap, only while the acceptance criterion is flagged `is_done: false` and has no impl annotation yet — expected on an in-progress spec still being worked on; the spec-level contract under `contract: single` is pending too while every criterion is. A pending-only `in-progress` spec (`●`) is listed under its `in-progress` status group instead of "Gaps".
- The "Gaps" header counts the gapped specs only, and a spec with no gap is not listed at all. Gapped specs are grouped by `scope`, sorted alphabetically with unscoped specs last under `unscoped`; a spec with several scopes appears under each, and the header counts it once.
- The report ends with a one-line summary of spec counts per status and the gap count, e.g. `planned (5), in-progress (3), active (2), gaps (1)`.

## Flags

- `--scope <repository-short-name>...` — limit the report to specs whose `scope` includes the given repository short name (`repository_short_name`). Defaults to all specs; errors on an unrecognized name.
- `--verbose` (`-v`) — add description detail: each `planned` spec listed under its status also shows its `description`, and each gapped spec in the "Gaps" section also shows its `description` plus, for every acceptance criterion still missing a required layer, that criterion's own text next to its id.

```sh
spex coverage --scope api frontend
spex coverage --verbose
```
