# CLI

The `spex` command-line tool manages a spex workspace.

```sh
spex --help
```

[Install](install.md) the binary from source, or see [Development](development.md) for building the `spex` checkout itself.

## Command overview

| Command                                | Purpose                                                                |
| -------------------------------------- | ---------------------------------------------------------------------- |
| [`spex init`](init.md)                 | Create the spex workspace repository                                   |
| [`spex update`](update.md)             | Refresh the bundled templates in the spex repository                   |
| [`spex check`](check.md)               | Run the rule checks declared in `spex.yaml`; fails on `error` findings |
| [`spex check-schema`](check-schema.md) | Validate JSON or YAML files against a JSON Schema                      |
| [`spex coverage`](coverage.md)         | Report `@spec` annotation coverage per acceptance criterion            |
| [`spex info`](info.md)                 | Print the resolved `spex.yaml` as JSON, for scripts                    |
| [`spex jq`](jq.md)                     | Run a jq filter on JSON or YAML, for scripts and agents                |
| [`spex rules`](rules.md)               | Print the rules governing the selected repositories                    |
| [`spex specs`](specs.md)               | Print the workspace specs                                              |

Every command except `init` finds the spex repository from the current directory — walking up and checking each directory it passes, plus its immediate subdirectories named `spex` or ending in `-spex`, for a `spex.yaml` — so any of them can run from anywhere inside the workspace.

`rules` and `specs` print a YAML list, to stdout or to `--output`.

Every command exits `0` on success, `1` when it ran and reported failures (`check` errors, `coverage` gaps, `check-schema` violations), and `2` when it could not run (invalid arguments, a missing `spex.yaml`, an unreadable schema). A command that could not run prints one `error: <message>` line to stderr, so a CI script can tell a tool failure from a finding.
