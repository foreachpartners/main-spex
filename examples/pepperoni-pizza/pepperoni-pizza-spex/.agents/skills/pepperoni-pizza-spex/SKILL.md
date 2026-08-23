---
name: pepperoni-pizza-spex
description: Information, rules, commands and other pepperoni-pizza-spex repository context. MUST be loaded before reading, writing, testing, or running any code in that repository.
---

# General

Run `spex info --repo spex` for general repository information. This repository holds the specs and config that govern the whole `pepperoni-pizza` workspace; it has no `rules` of its own.

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

# Commands

Run from anywhere inside the workspace — `spex` locates `spex.yaml` automatically.

- `spex check` - runs the rule spec checks declared in `spex.yaml` against the workspace (`--repo <name>` to limit scope, `--files <path>...` to target specific files, `--fill-preexisting-errors` to baseline current errors)
- `spex coverage` - reports how well the active specs are covered by spec-id annotations (`@spec impl`, `@spec test`, `@spec contract`) across the workspace
- `spex check-schema` - validates JSON or YAML files against a JSON Schema
- `spex info` - prints the resolved spex configuration (`--print <name>` for a single field, `--repo <name>` to limit to one repository)
- `spex update` - rewrites bundled skills, schemas, scripts and rules with the latest versions
