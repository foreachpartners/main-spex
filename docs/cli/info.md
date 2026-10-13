# `spex info`

Prints the resolved `spex.yaml` configuration as pretty JSON, led by `spex_dir` — the absolute path of the spex directory, which the relative `path` of each contract is resolved against — followed by every declared field, descriptions included, with an absent `contracts`, `rules`, `ignored_errors`, `rules_unused`, or `coverage_ignore` printed as an empty list, and `agent_root` and each package's `path` resolved to an absolute path, so they open from any working directory. It reports `spex.yaml` as written, without merging in any package's `spex.package.yaml`. It is the general-purpose form scripts pipe into [`spex jq`](jq.md) for whatever field they need:

```sh
spex info | spex jq -r '.project_name'
spex info | spex jq -r '.spex_dir'
spex info | spex jq -r '.agent_root'
spex info | spex jq -r '.packages[].path'
```

## Flags

- `--package <name>...` — filter the `packages` list to the named packages, each matched by either `package_name` or `package_short_name`; every other field is printed unchanged. Packages keep their `spex.yaml` order. Errors when any given name matches no package.

```sh
spex info --package api frontend
```
