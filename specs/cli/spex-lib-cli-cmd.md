# `spex lib`

Prints entries of a library bundled in the `spex` binary, so a skill or an AI agent reads ready-made templates offline, from anywhere. A kind is either `specs`, the [specs library](../spec/specs-library-spec.md) of template specs to draft from, or `rules`, the [rules library](../rule/rules-library-rule.md) of rules a project takes as they are. Neither is the project's own, which [`spex specs`](spex-specs-cli-cmd.md) and [`spex rules`](spex-rules-cli-cmd.md) print.

```sh
spex lib <KIND> [OPTIONS]
```

## Acceptance criteria

- MUST take `<KIND>`, the library to read, and accept only `specs` and `rules`.
- MUST print every entry of the kind in full as a valid YAML list ordered by `id` and nothing else on stdout, with a blank line between entries, in the same shape as [`spex specs`](spex-specs-cli-cmd.md) and [`spex rules`](spex-rules-cli-cmd.md) but without their `path`, so one `spex jq` filter reads all of them.
- MUST print only the entries named in `--id <id>...`, still ordered by `id` whatever the order given, each in the same format, when `--id` is given.
- MUST error on an unrecognized ID given to `--id`.
- MUST print only the top-level fields named in `--fields <field>...` of each entry, in the entry's own field order, instead of the entry in full, so an AI agent scans a catalog (e.g. `id` and `description`) without spending its context on every acceptance criterion. It defaults to all fields.
- MUST error on a name given to `--fields` that is not a field of an entry, listing them, so a typo does not silently print nothing: for `specs` `id`, `title`, `description` or `acceptance_criteria`, for `rules` a field of the rule schema. A field no entry sets is accepted and printed for no entry.
- MUST print only the entries that contain any keyword of `--search <keyword>...`, matched case-insensitively against the text of every field of the entry (field names are not searched), still ordered by `id` and in the same format, so an AI agent finds the entries about a capability or convention without printing the whole library. It combines with `--id` (only entries named there) and with `--fields`.
- MUST NOT fail when no entry is selected, for example when none matches `--search`: it prints an empty list (written to the file with `--output`) and prints `nothing found` to stderr, because no match is a valid answer.
- MUST write the output to the file given in `--output <path>` instead of stdout.
- MUST, given `--copy <dir>` for `rules`, write each rule named in `--id`, as bundled, to `<dir>/<rule-id>.yaml`, with its script folder `<dir>/<rule-id>/` and the scripts executable when it has one, and print the path of each written rule file, one per line, instead of the entries, so an AI agent takes a rule with its scripts and moves the files where they belong. The written rule keeps `sync_with_upstream: true`, so `spex update` keeps it in sync.
- MUST require `--id` with `--copy`, and refuse `--copy` together with `--search`, `--fields` or `--output`, so a project takes only the rules it picks, whole.
- MUST error on `--copy` for `specs`, which an agent drafts from rather than copies.
- MUST error, writing nothing, when `<dir>` already holds a rule file or script folder of a rule `--copy` would write, so a copy never overwrites a rule.
- MUST NOT look for `spex.yaml`: it reads only the bundled library.

## Examples

```sh
spex lib specs
spex lib specs --fields id description
spex lib specs --search password login --fields id description
spex lib specs --id password-policy
spex lib rules --fields id description
spex lib rules --search secret --fields id description
spex lib rules --id secret-handling openapi-conventions --copy my-project-spex/rules
```
