# Bundled `dependencies-rules` rule

The bundled [`dependencies-rules`](../../templates/rules/dependencies-rules.yaml) rule keeps dependency versions exact and reproducible. The rule file masters its criteria; this spec covers what spex ships and checks for it.

## Acceptance criteria

- MUST ship the rule at `templates/rules/dependencies-rules.yaml`, deployed by `spex init` and `spex update`.
- MUST report an `error` finding for a `Cargo.toml` version string that starts with `^`, `~`, `*`, `<`, or `>`.
- MUST report an `error` finding for a `package.json` string value that starts with `^`, `~`, `*`, `<`, `>`, or `latest`, or that is an x-range such as `1.x` or `1.2.x`.
