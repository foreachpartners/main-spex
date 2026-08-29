# General

Run `spex info --repo spex` for general repository information. This repository holds the specs and config that govern the whole `<project-name>` workspace; it has no `rules` of its own.

# Paths

- `spex.yaml` - project description, workspace repositories, their paths, contracts, and which rules apply to each
- `specs/` - specs: a capability the system must provide, with acceptance criteria
- `rules/` - rules: a cross-cutting convention or rule that applies repo-wide, with acceptance criteria
- `contracts/` - machine-readable interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) shared across repositories
- `plans/` - implementation plans derived from approved specs; an unchangeable history artifact once implementation is done
- `scripts/` - helper scripts a rule's `script` check references, one folder per spec id

Each fact lives in exactly one of these files — reference it, never copy it elsewhere.

# Notes

- `rules` lives per repository in `spex.yaml`, each id maps to its spec file lowercased, e.g. `NAMING-CONVENTION` → `rules/naming-convention.yaml`:
