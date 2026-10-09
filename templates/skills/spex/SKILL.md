---
name: spex
description: Information, rules, commands and other spex framework context. MUST be loaded before reading, writing, testing, or running any code in project spex repository (this).
---

# General

Run `spex info --repo spex` for general repository information. This repository holds the specs and config that govern the whole project.

# Paths

- `spex.yaml` - project and each workspace repository configuration, description, their paths, contracts, and applicable rules.
- `specs/` - spec: a capability the system must provide, with acceptance criteria. Specs are single source of truth for project - all behaviour, contracts, code, tests and other artifacts come from it and must adhere to it.
- `rules/` - rule: a cross-cutting convention or rule, with acceptance criteria. Each repository must adhere to a list of rules applicable to it defined in `spex.yaml`. A rule's `script` checks keep their scripts in a folder named after the rule beside its file (`rules/<rule-id>.yaml` and `rules/<rule-id>/`)
- `contracts/` - contract: interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) shared across repositories. A contract file's location is user-owned — most commonly inside the owning repository itself — and recorded in `spex.yaml`; the `spex` repository's `contracts/` folder is only a convention for contracts genuinely shared across repositories.
- `plans/` - plan: temporary implementation plan derived from approved spec and contract to plan and track huge task split in multiple steps. Deleted after completion.

---

# Project

Everything above `Project` is overrided on `spex update`, any project spicific data should be added here
