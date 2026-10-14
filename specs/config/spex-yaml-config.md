# `spex.yaml`

Declares the project and its packages. Validated against `schemas/spex.schema.json`.

## Acceptance criteria

- MUST declare the project and its packages in `spex.yaml`, in the spex directory.
- MUST require the root fields `project_name`, `cli_version`, `layout` (`multi-repo`, `single-repo`, or `monorepo`), and `agent_root` (the [agent root](../../docs/glossary.md), relative to the spex directory).
- MUST have `spex init` write `layout` and `agent_root`.
- MUST require, in each package entry, `package_name`, `package_short_name`, and `path` (relative to the spex directory).
- MUST make `package_name` the full `<project-name>-<package-short-name>`, matching the package directory in `multi-repo`; in `single-repo` it is the project name, which is then also the `package_short_name`.
- MUST require, in each `contracts` entry, a `path` and a `description`; the path is user-owned, typically inside the owning package, and relative to the spex directory.
- MUST declare `coverage_ignore` as one `{spec_id, layer, ac}` entry per waiver, with `spec_id` and `layer` required and `ac` optional; `layer` and `ac` mean what they do in a [spec's `coverage_ignore`](../spec/spec-file-spec.md).
- MUST validate `spex.yaml` against `schemas/spex.schema.json`.
- MAY declare the root fields `description`, `rules_unused` (Rule IDs `spex update` must not redeploy), `packages`, and `coverage_ignore`.
- MAY declare, in each package entry, a `description`, a `contracts` list, a `rules` list, and an `ignored_errors` baseline.

## Examples

```yaml
coverage_ignore:
  - spec_id: place-pizza-order
    layer: contract
  - spec_id: place-pizza-order
    layer: impl
    ac: no-null-typed-string
```
