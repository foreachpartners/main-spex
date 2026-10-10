# `spex specs`

Prints the workspace specs:

```yaml
- id: place-pizza-order
  title: Place a pizza order
  # ...
```

## Flags

- `--ai` — add a `path` field after `id` with the spec file's absolute path (e.g. `/work/shop/shop-spex/specs/place-pizza-order.yaml`).
- `--id <id>...` — limit output to the given spec IDs. Defaults to all specs; errors on an unrecognized ID.
- `--scope <repository-short-name>...` — limit output to specs whose `scope` includes the given repository short name (`repository_short_name`). Defaults to all specs; errors on an unrecognized name.
- `--output <path>` — write the output to this file instead of stdout, since the full specs list is large and can get truncated when printed to a terminal or piped through a shell.

```sh
spex specs --ai --scope api
spex specs --id place-pizza-order list-pizza-menu
```
