# Rules library

The rules library holds rules for conventions many projects share. It lives in the `templates/rules/` folder of the spex framework repository, included in the `spex` binary and read with [`spex lib rules`](../cli/spex-lib-cli-cmd.md). `spex init` and `spex update` never add a rule to a project, so a project holds a rule only once it takes it with `spex lib rules --copy`, which the [`spex-setup` skill](../../templates/skills/spex-setup/SKILL.md) does for the rules that fit each package. A taken rule is the project's own rule file that `spex update` keeps in sync with the library until the project sets `sync_with_upstream: false`. These rules are an opinionated starting point for the general case, which a project adapts to its needs.

## Acceptance criteria

- MUST keep the library rules in `templates/rules/`, which the binary includes and `spex init` and `spex update` never add to a project.
- MUST name each entry `<rule-id>.yaml`, a rule file that validates against `templates/schemas/rule.schema.json`, with its script folder (same name, beside the file) when its checks run scripts.
- MUST allow entries in any number of subfolders of `templates/rules/` (`db/`, `db/postgres/`), telling such a subfolder from a script folder by having no rule file of the same name beside it, and list, copy and sync those entries the same as entries at the top of the library.
- MUST give each entry an `id` no other entry has.
- MUST set `sync_with_upstream: true` right after the `title` of each entry, so a project's rule that sets `true` is known to come from the library.
- MUST fail `cargo test` when an entry is invalid, misnamed, unsynced, fails a conformance check of `spex check` (such as acceptance criteria out of commitment order) or shares its `id` with another entry.
- MUST offer entries only through `spex lib rules`, which the `spex-setup` skill reads to pick the rules of each package and the `spex-rule` skill reads when a user creates a rule that is not specific to a project.
- MUST write each entry as an opinionated starting point for the general case, choosing defaults and trade-offs most projects can adopt as they are, which a project is free to change for its needs.

## Examples

```sh
spex lib rules --fields id description
spex lib rules --id secret-handling --copy my-project-spex/rules
```
