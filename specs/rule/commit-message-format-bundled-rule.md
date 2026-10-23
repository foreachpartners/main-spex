# Bundled `commit-message-format` rule

The bundled [`commit-message-format`](../../templates/rules/commit-message-format.yaml) rule keeps commits in Conventional Commits form and traceable to specs. The rule file masters its criteria; this spec covers what spex ships and checks for it.

## Acceptance criteria

- MUST ship the rule at `templates/rules/commit-message-format.yaml` with its script folder, deployed by `spex init` and `spex update`.
- MUST report a `warn` finding, once per package, for each of the last 2 non-merge commit subjects that is not `<type>[(<scope>)][!]: <description>`, where `<type>` is `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, or `revert`, and `<scope>` is lowercase letters, digits, and hyphens.
- MUST report a `warn` finding for each `feat` commit among the last 3 commits whose message has no `@spec <spec-id>[, <spec-id>]` tag.
- MUST NOT report a commit whose subject is `initial commit`, in any case, as not in Conventional Commits format.
- MUST NOT require an `@spec` tag on a commit whose type is not `feat`.
- MUST NOT report any finding when the package directory is not the root of a git repository.
