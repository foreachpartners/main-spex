# `spex.package.yaml`

Lets a package own the waivers and the baseline that concern it. Validated against `schemas/spex.package.schema.json`.

## Acceptance criteria

- MUST treat a package without a `spex.package.yaml` as having nothing to merge.
- MUST merge everything by union; MUST NOT let one source override another.
- MUST compute a spec's coverage waivers as the union of its own `coverage_ignore`, the `coverage_ignore` entries of `spex.yaml` naming its id, and those of every package's `spex.package.yaml` naming its id, whichever packages the spec is `scope`d to.
- MUST compute a package's `ignored_errors` baseline as its entry's in `spex.yaml` plus its own `spex.package.yaml`'s.
- MUST validate `spex.package.yaml` against `schemas/spex.package.schema.json`.
- MAY place `spex.package.yaml` at the root of a package.
- MAY declare `coverage_ignore`, with the entries of the root `coverage_ignore` of [spex.yaml](spex-yaml-config.md).
- MAY declare `ignored_errors`, with the entries of a package's `ignored_errors` in `spex.yaml`, its `files` relative to this package's root.

## Examples

```yaml
coverage_ignore:
  - spec_id: place-pizza-order
    layer: impl
    ac: no-null-typed-string
ignored_errors:
  - rule_id: writing-style
    check: no-cyrillic
    files: [README.md]
```
