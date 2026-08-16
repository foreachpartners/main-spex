# General

Run `spex info --repo spex` for general repository information. This repository holds the specs and config that govern the whole `<project-name>` workspace; it has no `rule_spec_ids` of its own.

# Paths

- `spex.yaml` - project description, workspace repositories, their paths, contracts, and which rs-specs apply to each
- `fs-specs/` - functional specs (`FS-*`): a capability the system must provide, with acceptance criteria
- `rs-specs/` - rule specs (`RS-*`): a cross-cutting convention or rule that applies repo-wide, with acceptance criteria
- `contracts/` - machine-readable interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) shared across repositories
- `plans/` - implementation plans derived from approved functional specs; an unchangeable history artifact once implementation is done
- `scripts/` - helper scripts a rs-spec's `script` check references, one folder per spec id

Each fact lives in exactly one of these files — reference it, never copy it elsewhere.

# Commands

Run from anywhere inside the workspace — `spex` locates `spex.yaml` automatically.

- `spex check` - runs the rule spec checks declared in `spex.yaml` against the workspace (`--repo <name>` to limit scope, `--files <path>...` to target specific files, `--fill-preexisting-errors` to baseline current errors)
- `spex coverage` - reports how well the active functional specs are covered by spec-id annotations (`@spec impl`, `@spec test`, `@spec contract`) across the workspace
- `spex check-schema` - validates JSON or YAML files against a JSON Schema
- `spex info` - prints the resolved spex configuration (`--print <name>` for a single field, `--repo <name>` to limit to one repository)
- `spex update` - rewrites bundled skills, schemas, scripts and rs-specs with the latest versions

# Rules

After all changes across the workspace, run `spex check` (optionally scoped with `--repo`) and `spex coverage`.
