# Command set

The `spex` command-line tool manages a spex project. `spex --help` lists its commands.

## Acceptance criteria

- MUST provide `spex init`: initialize spex directory for a project ([spex-init-cli-cmd.md](spex-init-cli-cmd.md)).
- MUST provide `spex update`: sync project spex files with latest updates ([spex-update-cli-cmd.md](spex-update-cli-cmd.md)).
- MUST provide `spex check`: run the rule and framework checks ([spex-check-cli-cmd.md](spex-check-cli-cmd.md)).
- MUST provide `spex check-schema`: validate JSON or YAML files against a JSON Schema ([spex-check-schema-cli-cmd.md](spex-check-schema-cli-cmd.md)).
- MUST provide `spex coverage`: report specs coverage for a project ([spex-coverage-cli-cmd.md](spex-coverage-cli-cmd.md)).
- MUST provide `spex info`: print project information ([spex-info-cli-cmd.md](spex-info-cli-cmd.md)).
- MUST provide `spex jq`: run a jq filter on JSON or YAML ([spex-jq-cli-cmd.md](spex-jq-cli-cmd.md)).
- MUST provide `spex rules`: print the rules governing the selected packages ([spex-rules-cli-cmd.md](spex-rules-cli-cmd.md)).
- MUST provide `spex specs`: print the project specs ([spex-specs-cli-cmd.md](spex-specs-cli-cmd.md)).
- MUST list every command in `spex --help`.
