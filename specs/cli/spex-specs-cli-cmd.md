# `spex specs`

Prints the project specs, whole or narrowed to the ones an AI agent needs, so a skill reads exactly the specs it works on.

## Acceptance criteria

- MUST print the project specs as a valid YAML list and nothing else on stdout (warnings go to stderr), so [`spex jq`](spex-jq-cli-cmd.md) and any other YAML tool read the output.
- MUST add a `path` field after the `id` of each spec with the spec file's absolute path (e.g. `/work/shop/shop-spex/specs/place-pizza-order.yaml`), so an AI agent edits the file without searching for it.
- MUST limit the output to the spec IDs given in `--id <id>...`; it defaults to all specs.
- MUST error on an unrecognized ID given to `--id`.
- MUST limit the output to the specs whose `packages` include a package named in `--package <name>...`, matched by either full `package_name` or `package_short_name`; it defaults to all specs.
- MUST error on a name given to `--package` that matches no package.
- MUST print only the top-level fields named in `--fields <field>...` of each spec, in the spec's own field order (`path` is a field like any other), instead of the spec in full, so an AI agent scans a catalog (e.g. `id` and `description`) without spending its context on every acceptance criterion; it defaults to all fields.
- MUST error on a name given to `--fields` that is neither `path` nor a field of the spec schema, listing the valid fields, so a typo does not silently print nothing. A field no spec sets is accepted and printed for no spec.
- MUST print only the specs that contain any keyword of `--search <keyword>...`, matched case-insensitively against the text of every field of the spec file (field names and `path` are not searched), so an AI agent finds the specs about a capability without printing them all. It combines with `--id`, `--package` and `--fields`.
- MUST NOT fail when no spec is selected, for example when none matches `--search` or the `--package` has none: it prints an empty list and prints `nothing found` to stderr, because no match is a valid answer.
- MUST write the output to the file given in `--output <path>` instead of stdout, since the full specs list is large and can get truncated when printed to a terminal or piped through a shell.

## Examples

```sh
spex specs --package api
spex specs --id place-pizza-order list-pizza-menu
spex specs --fields id title description
spex specs --search password login --fields id path description
```
