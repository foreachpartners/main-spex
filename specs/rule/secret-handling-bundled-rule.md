# Bundled `secret-handling` rule

The bundled [`secret-handling`](../../templates/rules/secret-handling.yaml) rule keeps secrets out of logs and out of committed files. The rule file masters its criteria; this spec covers what spex ships and checks for it.

## Acceptance criteria

- MUST ship the rule at `templates/rules/secret-handling.yaml`, deployed by `spex init` and `spex update`.
- MUST report a `warn` finding for each file named `*.bak`, `*.backup`, or `*.orig`, at any depth.
- MUST report an `error` finding for a `console` method call in a `.ts`, `.tsx`, `.js`, or `.jsx` file when an argument contains `token`, `secret`, `password`, `credential`, `apikey`, `api_key`, or `api-key`, case-insensitively.
- MUST report an `error` finding for an `info!`, `warn!`, `error!`, `debug!`, or `trace!` macro call in a `.rs` file when an argument contains one of the same words.
