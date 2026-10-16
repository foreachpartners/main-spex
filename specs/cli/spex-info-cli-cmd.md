# `spex info`

Prints project information: the resolved `spex.yaml` and each `spex.package.yaml` configuration merged with additional fields and some fields changed as pretty JSON. It is the general-purpose form scripts pipe into [`spex jq`](spex-jq-cli-cmd.md) for whatever field they need.

## Acceptance criteria

- MUST print the resolved `spex.yaml` configuration as pretty JSON.
- MUST add root field `spex_dir`: the absolute path of the spex directory.
- MUST rewrite `agent_root` and each package's `path` to an absolute path.
- MUST merge every package's `spex.package.yaml` into its package entry, so each package lists its own `check_ignore` and `coverage_ignore`.
- MUST filter the `packages` list to the packages named in `--package <name>...`, each matched by either `package_name` or `package_short_name`.
- MUST error when any name given to `--package` matches no package.

## Examples

```sh
spex info --package api frontend
```
