# `spex lib`

Prints entries of a library bundled in the `spex` binary, so a skill or an AI agent reads ready-made template specs offline, from anywhere. The only kind today is `specs`, the [specs library](../spec/specs-library-spec.md): template specs to draft from, not the project's own specs, which [`spex specs`](spex-specs-cli-cmd.md) prints.

```sh
spex lib <KIND> [OPTIONS]
```

## Acceptance criteria

- MUST take `<KIND>`, the library to read, and accept only `specs` for now.
- MUST print every entry of the kind in full as a YAML list ordered by `id`, with a blank line between entries, in the same shape as [`spex specs`](spex-specs-cli-cmd.md) and [`spex rules`](spex-rules-cli-cmd.md), so one `spex jq` filter reads all three.
- MUST print only the entries named in `--id <id>...`, still ordered by `id` whatever the order given, each in the same format, when `--id` is given.
- MUST error on an unrecognized ID given to `--id`.
- MUST print only the top-level fields named in `--fields <field>...` of each entry, in the entry's own field order, instead of the entry in full, so an AI agent scans a catalog (e.g. `id` and `description`) without spending its context on every acceptance criterion. It defaults to all fields.
- MUST error on a name given to `--fields` that is not a field of an entry (`id`, `title`, `description` or `acceptance_criteria`), listing them, so a typo does not silently print nothing. A field no entry sets is accepted and printed for no entry.
- MUST print only the entries that contain any keyword of `--search <keyword>...`, matched case-insensitively against the text of every field of the entry (field names are not searched), still ordered by `id` and in the same format, so an AI agent finds the entries about a capability without printing the whole library. It combines with `--id` (only entries named there) and with `--fields`.
- MUST NOT fail when no entry is selected, for example when none matches `--search`: it prints an empty list (written to the file with `--output`) and prints `nothing found` to stderr, because no match is a valid answer.
- MUST write the output to the file given in `--output <path>` instead of stdout.
- MUST NOT look for `spex.yaml`: it reads only the bundled library.

## Examples

```sh
spex lib specs
spex lib specs --fields id description
spex lib specs --search password login --fields id description
spex lib specs --id password-policy
```
