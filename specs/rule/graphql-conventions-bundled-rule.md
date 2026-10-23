# Bundled `graphql-conventions` rule

The bundled [`graphql-conventions`](../../templates/rules/graphql-conventions.yaml) rule keeps the GraphQL SDL consistent as the contract between clients and servers. The rule file masters its criteria; this spec covers what spex ships for it.

## Acceptance criteria

- MUST ship the rule at `templates/rules/graphql-conventions.yaml`, deployed by `spex init` and `spex update`.
- MUST NOT define a check in the rule; its criteria are verified by review with `spex-review-rule`.
