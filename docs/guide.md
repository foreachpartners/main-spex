# Guide

In-depth guide to working with spex, picking up where the [README](../README.md) quickstart leaves off.

## Updating spex

After upgrading the `spex` binary, run `spex update` to pull the latest framework files into your spex repository:

```sh
spex update
```

See the [CLI reference](cli.md) for the full command set.

## Overriding a check script per repository

`script` and `command` checks run `scripts/<name>.sh`, which spex resolves from the workspace repository first and from the spex repository only as a fallback. A tech spec can therefore ship a shared default script while letting an individual repository override it.

To override the shared script for one repository, add a `scripts/<name>.sh` with the same name in that repository:

```text
pepperoni-pizza/                 <- workspace root folder
  spex/
    scripts/
      commit-id-format.sh        <- shared default, used by every repository
  api/
    scripts/
      commit-id-format.sh        <- overrides the shared script for `api` only
```

When `spex check` runs the `commit-id-format` check against `api`, it uses `api/scripts/commit-id-format.sh`; every other repository falls back to `spex/scripts/commit-id-format.sh`.

## See also

- [Conventions](conventions.md)
- [FAQ](faq.md)
