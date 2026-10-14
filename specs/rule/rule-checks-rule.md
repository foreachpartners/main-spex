# Rule checks

A check proves a rule's acceptance criterion. How `spex check` invokes a script check, and what a script must report, is specified in [spex-check-cli-cmd.md](../cli/spex-check-cli-cmd.md).

## Acceptance criteria

- MUST give each check a type: `regexp` (file contents), `glob` (file paths), `ast-grep` (structural code patterns), or `script` (external program).
- MUST narrow the file set of every check with `include` / `exclude` globs, except `per-package` and `workspace` scripts, which discover their own files.
- MUST invert a `regexp`, `glob`, or `ast-grep` check with `negate`: a forbidden pattern versus a required pattern.
- MUST set `severity` to `error` (default) or `warn`.
- MUST have a package inherit a rule's checks through `rules`.
- MUST resolve a script check in the rule's script folder, named after the rule file and beside it (`rules/<rule-id>.yaml` -> `rules/<rule-id>/<script>`), with `run` set to `per-file` (default), `per-package`, or `workspace`.
- SHOULD read data from `spex.yaml` (the project name, the package list) in a script with `spex jq` rather than parsing `spex.yaml` by hand.
