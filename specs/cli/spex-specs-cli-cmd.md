# `spex specs`

Prints the project specs as a YAML list, to stdout or to `--output`.

## Acceptance criteria

- MUST print the project specs as a YAML list (`- id: place-pizza-order`, `title: Place a pizza order`, ...).
- MUST add a `path` field after `id` with the spec file's absolute path with `--ai` (e.g. `/work/shop/shop-spex/specs/place-pizza-order.yaml`).
- MUST limit the output to the spec IDs given in `--id <id>...`; it defaults to all specs.
- MUST error on an unrecognized ID given to `--id`.
- MUST limit the output to the specs whose `scope` includes a package short name given in `--scope <package-short-name>...` (`package_short_name`); it defaults to all specs.
- MUST error on an unrecognized name given to `--scope`.
- MUST write the output to the file given in `--output <path>` instead of stdout, since the full specs list is large and can get truncated when printed to a terminal or piped through a shell.

## Examples

```sh
spex specs --ai --scope api
spex specs --id place-pizza-order list-pizza-menu
```
