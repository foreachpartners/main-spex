# Project discovery

Every command that works on a project finds its spex directory itself, so it runs from anywhere inside the project.

## Acceptance criteria

- MUST find the spex directory, in commands that load a project, by walking up from the current directory and checking for a direct `spex.yaml`, then the immediate subdirectories named `spex` or ending in `-spex`.
- MUST load an explicit `--project <path>` directly. Accept the path to a `spex.yaml` file or a directory containing one, resolving relative paths from the current directory.
- MUST report the candidate paths and require `--project` when a discovery step finds several sibling configs and cannot identify exactly one that declares the current directory's package.
- MUST let a command that deliberately does not load a project, such as `update --migration-guide`, run independently of discovery.
