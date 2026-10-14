# CLI version

Records and reconciles the CLI version that wrote the config.

## Acceptance criteria

- MUST record the CLI version that wrote `spex.yaml` as `cli_version`.
- MUST report a mismatch between `cli_version` and the installed binary from the commands that load the project: `check`, `coverage`, `info`, `rules`, and `specs`.
- MUST resolve a mismatch by running [`spex update`](../cli/spex-update-cli-cmd.md), which updates `cli_version` to the installed binary's version.
