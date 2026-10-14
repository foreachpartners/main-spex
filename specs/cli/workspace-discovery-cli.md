# Workspace discovery

Every command that works on a workspace finds its spex directory itself, so it runs from anywhere inside the workspace.

## Acceptance criteria

- MUST find the spex directory from the current directory, in every command except `init`, `check-schema`, and `jq`.
- MUST look for a `spex.yaml` by walking up from the current directory and checking each directory it passes, plus its immediate subdirectories named `spex` or ending in `-spex`.
- MUST NOT look for a `spex.yaml` in `spex init` ([spex-init-cli-cmd.md](spex-init-cli-cmd.md)), which creates one, in `spex check-schema` ([spex-check-schema-cli-cmd.md](spex-check-schema-cli-cmd.md)), which reads only the given schema and files, or in `spex jq` ([spex-jq-cli-cmd.md](spex-jq-cli-cmd.md)), which reads only the given file or standard input.
