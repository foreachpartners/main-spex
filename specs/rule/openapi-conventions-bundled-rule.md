# Bundled `openapi-conventions` rule

The bundled [`openapi-conventions`](../../templates/rules/openapi-conventions.yaml) rule keeps OpenAPI documents consistent as the contract between clients and servers. The rule file masters its criteria; this spec covers what spex ships and checks for it.

## Acceptance criteria

- MUST ship the rule at `templates/rules/openapi-conventions.yaml` with its script folder, which holds the vacuum ruleset, deployed by `spex init` and `spex update`.
- MUST lint each file named `*.openapi.yaml`, `*.openapi.yml`, `*.openapi.json`, `openapi.yaml`, `openapi.yml`, or `openapi.json` with `vacuum` against the bundled ruleset.
- MUST report each ruleset violation as an `error` finding at the violation's line.
- MUST report an `error` finding with the vacuum installation link when `vacuum` is not installed.
- MUST report an `error` finding for a file when `vacuum` fails or its report cannot be read.
- MUST require `openapi` to be `3.1.1` in the ruleset.
- MUST require every operation to declare `operationId`, `summary`, at least one response, and `x-spec`.
- MUST require every `operationId` to be unique across the document.
- MUST require every request and response schema to be a `$ref` to `components/schemas`.
- MUST require every `components/securitySchemes` key to match `^[a-zA-Z0-9._-]+$`.
- MUST require every secret-bearing field to set `x-sensitive: true`, and every secret string field to set `format: password`.
- MUST require property names, query parameter names, and enum values to be snake_case, and URL path segments to be kebab-case.
- MUST NOT allow `anyOf` or `oneOf` without a discriminator in the ruleset.
- MUST NOT allow `info.description` to duplicate the spec mapping carried by `x-spec`.
- MUST NOT allow a description to leak internal implementation or persistence details.
