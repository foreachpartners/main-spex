# `spex info`

Prints the resolved `spex.yaml` configuration as pretty JSON, led by `spex_dir` — the absolute path of the spex repository, which the relative `path` of each contract is resolved against — followed by every declared field, descriptions included, with an absent `contracts`, `rules`, `ignored_errors`, `rules_unused`, or `coverage_ignore` printed as an empty list and each repository's `path` resolved to an absolute path, so it opens from any working directory. It reports `spex.yaml` as written, without merging in any repository's `spex.repo.yaml`. It is the general-purpose form scripts pipe into [`spex jq`](jq.md) for whatever field they need:

```sh
spex info | spex jq -r '.project_name'
spex info | spex jq -r '.spex_dir'
spex info | spex jq -r '.repositories[].path'
```

## Flags

- `--repo <name>...` — filter the `repositories` list to the named repositories, each matched by either `repository_name` or `repository_short_name`; every other field is printed unchanged. Repositories keep their `spex.yaml` order. Errors when any given name matches no repository.

```sh
spex info --repo api frontend
```
