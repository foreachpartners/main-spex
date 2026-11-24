# `spex doctor`

Reports the project and Git inputs the installed CLI would inspect.

## Acceptance criteria

- MUST report the running CLI's version and executable path, the resolved project config and configured CLI version, and each declared package path.
- MUST report whether a package is present, its Git revision and dirty status when available, and why a Git revision or status could not be obtained.
- MUST retain the project and package diagnostics for an older config with readable identity even when the installed project schema rejects it. List the incompatibilities and point to the migration guide. Report a syntax error in the requested report format when the config cannot be decoded.
- MUST report a version mismatch separately from config compatibility.
- MUST return failure for incompatible project config or missing package directories. A present non-Git package may be inspected normally.
- MUST support `--format json` for a versioned structured diagnostic.
- MUST print a portable JSON revision manifest with `--revisions`. Include the schema version, project name, the config repository's full commit ID and the package commit IDs by package name. Require the config and every package to belong to a clean Git worktree and verify the Git root against the project layout.
- MUST, with `--verify-revisions <path>`, require the manifest to describe this project and exactly its packages. Report missing, different, unavailable and dirty package revisions and fail on any mismatch.
- MUST let `check` and `coverage` verify the same manifest with `--require-revisions <path>` before evaluating any package.
- MUST leave project files untouched.

## Examples

```sh
spex --project ./shopping-notes-spex doctor
spex doctor --format json
spex doctor --revisions > /tmp/shopping-notes.revisions.json
spex doctor --verify-revisions /tmp/shopping-notes.revisions.json
```
