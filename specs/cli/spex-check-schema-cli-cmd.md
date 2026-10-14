# `spex check-schema`

Validates JSON or YAML files against a JSON Schema.

## Acceptance criteria

- MUST validate every file given in `--files <path>...`, one or more JSON or YAML files, against the schema given in `--schema <path>`; both are required.
- MUST resolve cross-file `$ref`s against sibling schema files in the schema's own directory, fully offline.
- MUST print violations to stderr as `<file>: <message>`.
- MUST exit non-zero when any file fails.
- MUST use the same validation as `spex check`, built in, to keep the spex repository's own files (`spex.yaml`, `specs/<spec-id>.yaml`, `rules/<rule-id>.yaml`) valid against the bundled schemas.
- MUST NOT use the network.

## Examples

```sh
spex check-schema --schema schemas/rule.schema.json --files rules/naming-convention.yaml
spex check-schema --schema schemas/spec.schema.json --files specs/list-pizza-menu.yaml specs/place-pizza-order.yaml
```
