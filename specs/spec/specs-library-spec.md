# Specs library

The specs library holds template specs for capabilities many projects need. It lives in the `templates/specs/` folder of the spex framework repository, included in the `spex` binary and read with [`spex lib specs`](../cli/spex-lib-cli-cmd.md), and `spex init` and `spex update` never scaffold it, so a project holds an entry only once a user takes it. The [`spex-spec` skill](../../templates/skills/spex-spec/SKILL.md) is encouraged to read the specs library when a user creates a spec. These specs are an opinionated starting point for the general case, which a project adapts to its needs.

## Acceptance criteria

- MUST keep the library specs in `templates/specs/`, which the binary includes and `spex init` and `spex update` never scaffold.
- MUST name each entry `<spec-id>.yaml`, a partial spec file that validates against `templates/schemas/spec.schema.json` without its `packages`, `contract` and `status` fields.
- MUST give each entry an `id` no other entry has.
- MUST fail `cargo test` when an entry is invalid, misnamed or shares its `id` with another entry.
- MUST offer entries only through `spex lib specs`, which the `spex-spec` skill is encouraged to read when a user creates a spec that is not specific to a project.
- MUST write each entry as an opinionated starting point for the general case, choosing defaults and trade-offs most projects can adopt as they are, which a project is free to change for its needs.
- MUST NOT hold `packages`, `contract` or `status` in an entry, which the project sets when it takes the entry.

## Examples

```sh
spex lib specs --id password-policy
```
