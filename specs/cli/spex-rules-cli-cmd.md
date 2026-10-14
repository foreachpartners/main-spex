# `spex rules`

Prints, for the selected packages, the full content of each rule listed in their `rules`, as a YAML list, to stdout or to `--output`.

## Acceptance criteria

- MUST print each rule once, however many packages list it.
- MUST print the full content of each rule, as a YAML list (`- id: secret-handling`, `title: Secret protection`, ...).
- MUST drop the fields an agent does not need to follow a rule (`sync_with_upstream` and `checks`) with `--ai`, and add a `path` field after `id` with the rule file's absolute path (e.g. `/work/shop/shop-spex/rules/secret-handling.yaml`).
- MUST limit the output to the rule IDs given in `--id <id>...`; it defaults to all rules.
- MUST error on an unrecognized ID given to `--id`.
- MUST limit the output to the packages named in `--package <name>...`, matched by either `package_name` or `package_short_name`; it defaults to all packages in `spex.yaml`.
- MUST error on an unrecognized name given to `--package`.
- MUST write the output to the file given in `--output <path>` instead of stdout, since the full rules list is large and can get truncated when printed to a terminal or piped through a shell.

## Examples

```sh
spex rules --ai --package api
spex rules --id secret-handling naming-convention
```
