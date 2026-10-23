# Bundled `naming-convention` rule

The bundled [`naming-convention`](../../templates/rules/naming-convention.yaml) rule derives every cross-cutting identifier from the project name and the package short name. The rule file masters its criteria; this spec covers what spex ships and checks for it.

## Acceptance criteria

- MUST ship the rule at `templates/rules/naming-convention.yaml` with its script folder, deployed by `spex init` and `spex update`.
- MUST report an `error` finding, in the `multi-repo` layout, for a package directory name that is not kebab-case.
- MUST report an `error` finding, in the `multi-repo` layout, for a package directory name that does not start with `<project-name>-` followed by at least one character.
- MUST report an `error` finding when the layout or the project name cannot be read from `spex.yaml`.
- MUST NOT check the package directory name in the `single-repo` and `monorepo` layouts.
