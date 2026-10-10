# `spex check-schema`

Validates JSON or YAML files against a JSON Schema.

- Cross-file `$ref`s resolve against sibling schema files in the schema's own directory — fully offline, no network access.
- Violations are printed to stderr as `<file>: <message>`; the command exits non-zero when any file fails.
- `spex check` uses the same validation, built in, to keep the spex repository's own files (`spex.yaml`, `specs/<spec-id>.yaml`, `rules/<rule-id>.yaml`) valid against the bundled schemas.

## Flags

- `--schema <path>` — the JSON Schema the files are validated against (required).
- `--files <path>...` — one or more JSON or YAML files to validate against the schema (required).

```sh
spex check-schema --schema schemas/rule.schema.json --files rules/naming-convention.yaml
spex check-schema --schema schemas/spec.schema.json --files specs/list-pizza-menu.yaml specs/place-pizza-order.yaml
```
