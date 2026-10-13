# `spex rules`

Prints, for the selected packages, the full content of each rule listed in their `rules` — each rule once, however many packages list it:

```yaml
- id: secret-handling
  title: Secret protection
  # ...
```

## Flags

- `--ai` — drop fields an agent doesn't need to follow a rule (`sync_with_upstream` and `checks`), and add a `path` field after `id` with the rule file's absolute path (e.g. `/work/shop/shop-spex/rules/secret-handling.yaml`).
- `--id <id>...` — limit output to the given rule IDs. Defaults to all rules; errors on an unrecognized ID.
- `--package <name>...` — limit output to the named packages, matched by either `package_name` or `package_short_name`. Defaults to all packages in `spex.yaml`; errors on an unrecognized name.
- `--output <path>` — write the output to this file instead of stdout, since the full rules list is large and can get truncated when printed to a terminal or piped through a shell.

```sh
spex rules --ai --package api
spex rules --id secret-handling naming-convention
```
