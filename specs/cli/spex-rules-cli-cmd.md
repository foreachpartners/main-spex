# `spex rules`

Prints rules project uses, whole or narrowed to the ones an AI agent needs, so a skill reads exactly the rules that govern its work.

## Acceptance criteria

- MUST print each rule once, however many packages list it.
- MUST print the full content of each rule as a valid YAML list and nothing else on stdout (warnings go to stderr), so [`spex jq`](spex-jq-cli-cmd.md) and any other YAML tool read the output.
- MUST add a `path` field after the `id` of each rule with the rule file's absolute path (e.g. `/work/shop/shop-spex/rules/secret-handling.yaml`), so an AI agent edits the file without searching for it.
- MUST limit the output to the rule IDs given in `--id <id>...`; it defaults to all rules project uses.
- MUST error on an unrecognized ID given to `--id`.
- MUST limit the output to the packages named in `--package <name>...`, matched by either `package_name` or `package_short_name`; it defaults to all packages in `spex.yaml`.
- MUST error on an unrecognized name given to `--package`.
- MUST print only the top-level fields named in `--fields <field>...` of each rule, in the rule's own field order (`path` is a field like any other), instead of the rule in full, so an AI agent reads what it needs to follow a rule (e.g. `id`, `path`, `title`, `description`, `acceptance_criteria` and `examples`) without spending its context on the `checks`; it defaults to all fields.
- MUST error on a name given to `--fields` that is neither `path` nor a field of the rule schema, listing the valid fields, so a typo does not silently print nothing. A field no rule sets is accepted and printed for no rule.
- MUST print only the rules that contain any keyword of `--search <keyword>...`, matched case-insensitively against the text of every field of the rule file (field names and `path` are not searched), so an AI agent finds the rules about a convention without printing them all. It combines with `--id`, `--package` and `--fields`.
- MUST NOT fail when no rule is selected, for example when none matches `--search` or the `--package` uses none: it prints an empty list and prints `nothing found` to stderr, because no match is a valid answer.
- MUST write the output to the file given in `--output <path>` instead of stdout, since the full rules list is large and can get truncated when printed to a terminal or piped through a shell.

## Examples

```sh
spex rules --package api
spex rules --id secret-handling naming-convention
spex rules --package api --fields id path title description acceptance_criteria examples
spex rules --search secret token --fields id description
```
