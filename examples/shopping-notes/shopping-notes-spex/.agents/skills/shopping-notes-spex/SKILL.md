---
name: shopping-notes-spex
description: Information, rules, commands and other shopping-notes-spex repository context. MUST be loaded before reading, writing, testing, or running any code in that repository.
---

# General

Run `spex info --repo spex` for general repository information. This repository holds the specs and config that govern the whole `shopping-notes` workspace; it has no `rules` of its own.

# Paths

- `spex.yaml` - project description, workspace repositories, their paths, contracts, and which rules apply to each
- `specs/` - spec: a capability the system must provide, with acceptance criteria
- `rules/` - rule: a cross-cutting convention or rule that applies repo-wide, with acceptance criteria
- `contracts/` - contract: machine-readable interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) shared across repositories. Contracts can be located in different repositories, see `spex.yaml` for paths.
- `plans/` - plan: temporary implementation plan derived from approved spec and contract to plan and track huge task. Deleted after completion.
- `scripts/` - helper scripts a rule's `script` check references, one folder per spec id

Each fact lives in exactly one of these files — reference it, never copy it elsewhere.

# Notes

- `rules` applicable to each repositoy defined in `spex.yaml`, each rule id maps to its file, e.g. `naming-convention` -> `rules/**/naming-convention.yaml`:
