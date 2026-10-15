# Spec file

A spec, defined in [glossary.md](../../docs/glossary.md), lives at `specs/<spec-id>.yaml`, nested subdirectories allowed; the Spec ID is the filename, whatever the nesting.

## Acceptance criteria

- MUST name the file `<spec-id>.yaml`, whatever its nesting under `specs/`.
- MUST follow `schemas/spec-id.schema.json` for the `id`: lowercase letters, digits and hyphens only, at most 40 characters.
- MUST have the required fields `id`, `title`, `description`, `acceptance_criteria`, and `contract`.
- MUST have at least one acceptance criterion; a spec with an empty `acceptance_criteria` is a schema error.
- MUST hold the spec's specifics in `acceptance_criteria`, ordered by RFC 2119 commitment level: `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`.
- MUST write each acceptance criterion as `{id, must|must_not|should|should_not|may, is_done}`, with exactly one of `must`, `must_not`, `should`, `should_not`, `may` present, its value the verifiable statement.
- MUST follow `schemas/ac-id.schema.json` for a criterion's `id`: lowercase letters, digits and hyphens only, at most 40 characters.
- MUST keep a criterion's `id` unique within the spec, naming the criterion's behavior rather than its position.
- MUST set `contract` to `single`, `for-each-ac`, or `none`, which sets the contract coverage granularity.
- MUST require, under `contract: single` (the migration default), one spec-level `@spec contract <spec-id>` annotation for the whole spec.
- MUST require, under `contract: for-each-ac`, a `@spec contract <spec-id>#<ac-id>` annotation for every acceptance criterion.
- MUST disable contract coverage under `contract: none`.
- MUST default `is_done` to `true` when absent; `false` flags a criterion as not actually done yet.
- MUST have `spex check` read `is_done`, not annotations, to decide whether a spec's status can be `active`.
- MUST have `spex coverage` read `is_done` only as a marker that a criterion with no impl annotation yet is pending rather than a gap; see [coverage-states-spec.md](coverage-states-spec.md).
- MUST default `status` to `planned`.
- MUST require, for `status: active`, every acceptance criterion's `is_done` to be `true` or absent; `active` means fully implemented.
- MUST list in `blocked_by` only genuine, unsatisfied blockers: specs that must be in place before this spec can be implemented.
- MUST give each `coverage_ignore` entry a required `layer` (`contract`, `impl`, or `test`); without `ac`, it waives that layer for the whole spec.
- MUST have a `coverage_ignore` entry with `ac` waive just that acceptance criterion's impl or test requirement, or, under `contract: for-each-ac`, its contract requirement.
- MUST match each `coverage_ignore` entry to the spec's `contract` setting: a whole-spec contract waiver works under `single` and `for-each-ac`, a per-criterion contract waiver only under `for-each-ac`, and under `contract: none` a contract-layer entry is pointless; `spex check` errors on a mismatch.
- MUST NOT list an already shipped or established spec in `blocked_by`.
- MAY set `packages`, the `package_short_name`s of the packages that implement the capability.
- MAY set `status` to `in-progress` for a spec being implemented but not yet complete, and to `blocked` or `deprecated` where they apply.
- MAY set `examples`: named illustrative snippets showing the spec applied in context, for placement and usage that prose cannot carry.
- MAY declare a waiver outside the spec file, in [`spex.yaml`](../config/spex-yaml-config.md) or a package's [`spex.package.yaml`](../config/spex-package-yaml-config.md), so the package that cannot satisfy a layer owns the waiver without editing the spec.

## Examples

```yaml
coverage_ignore:
  - layer: contract
  - layer: impl
    ac: no-null-typed-string
```
