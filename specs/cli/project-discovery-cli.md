# Project discovery

Every command that works on a project finds its spex directory itself, so it runs from anywhere inside the project.

## Acceptance criteria

- MUST find the spex directory, in every command except `init`, `check-schema`, `jq`, and `lib`, by looking for a `spex.yaml` walking up from the current directory and checking each directory it passes, plus its immediate subdirectories named `spex` or ending in `-spex`.
