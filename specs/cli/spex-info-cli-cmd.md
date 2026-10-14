# `spex info`

Prints project information: the resolved `spex.yaml` configuration as pretty JSON. It is the general-purpose form scripts pipe into [`spex jq`](spex-jq-cli-cmd.md) for whatever field they need.

## Acceptance criteria

- MUST print the resolved `spex.yaml` configuration as pretty JSON, led by `spex_dir`: the absolute path of the spex directory, which the relative `path` of each contract is resolved against.
- MUST print, after `spex_dir`, every declared field, descriptions included.
- MUST print an absent `contracts`, `rules`, `ignored_errors`, `rules_unused`, or `coverage_ignore` as an empty list.
- MUST resolve `agent_root` and each package's `path` to an absolute path, so they open from any working directory.
- MUST report `spex.yaml` as written, without merging in any package's `spex.package.yaml`.
- MUST filter the `packages` list to the packages named in `--package <name>...`, each matched by either `package_name` or `package_short_name`, leaving every other field unchanged and the packages in their `spex.yaml` order.
- MUST error when any name given to `--package` matches no package.

## Examples

```sh
spex info | spex jq -r '.project_name'
spex info | spex jq -r '.spex_dir'
spex info | spex jq -r '.agent_root'
spex info | spex jq -r '.packages[].path'
spex info --package api frontend
```
