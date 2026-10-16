# `spex.package.yaml`

Lets a package own the ignores that concern it. Validated against `schemas/spex.package.schema.json`.

## Acceptance criteria

- MUST treat a package without a `spex.package.yaml`, or with an empty one, as having nothing to merge.
- MUST merge it by union into the entry of its own package in `spex.yaml`, as if declared there, so its ignores apply to that package only; MUST NOT let one source override another.
- MUST validate `spex.package.yaml` against `schemas/spex.package.schema.json`, an empty file being valid.
- MAY place `spex.package.yaml` at the root of a package.
- MAY declare `check_ignore`, with the entries of a package's `check_ignore` in [spex.yaml](spex-yaml-config.md), its `files` relative to this package's root.
- MAY declare `coverage_ignore`, with the entries of a package's `coverage_ignore` in `spex.yaml`.

## Examples

```yaml
check_ignore:
  - rule_id: writing-style
    check: no-cyrillic
    files: [README.md]
coverage_ignore:
  - spec_id: place-pizza-order
    layer: impl
    ac: no-null-typed-string
```
