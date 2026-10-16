# `spex.yaml`

Declares the project and its packages. Validated against `schemas/spex.schema.json`.

## Acceptance criteria

- MUST declare the project and its packages in `spex.yaml`, in the spex directory.
- MUST require the root fields `project_name`, `cli_version`, `layout` (`multi-repo`, `single-repo`, or `monorepo`), and `agent_root` (the [agent root](../../docs/glossary.md), relative to the spex directory).
- MUST have `spex init` write `layout` and `agent_root`.
- MUST require, in each package entry, `package_name`, `package_short_name`, and `path` (relative to the spex directory).
- MUST make `package_name` the full `<project-name>-<package-short-name>`, matching the package directory in `multi-repo`; in `single-repo` it is the project name, which is then also the `package_short_name`.
- MUST require, in each `contracts` entry, a `path` and a `description`; the path is user-owned, typically inside the owning package, and relative to the spex directory.
- MUST declare a package entry's `coverage_ignore` as one `{spec_id, layer, ac}` entry per waiver, with `spec_id` and `layer` required and `ac` optional; `layer` and `ac` mean what they do in a [spec's `coverage_ignore`](../spec/spec-file-spec.md).
- MUST declare a package entry's `exclude` as a list of glob patterns, the globs a [check's](../rule/rule-checks-rule.md) `include` / `exclude` use, matched against paths relative to the package's root, each naming files or folders that `spex check` and `spex coverage` leave out of that package, as [spex check](../cli/spex-check-cli-cmd.md) and [spex coverage](../cli/spex-coverage-cli-cmd.md) describe.
- MUST apply the `exclude`, `check_ignore`, and `coverage_ignore` declared in a package entry to that package only, so the excludes and ignores of one package never affect another.
- MUST validate `spex.yaml` against `schemas/spex.schema.json`.
- MAY declare the root fields `description`, `rules_unused` (Rule IDs `spex update` must not redeploy), and `packages`.
- MAY declare, in each package entry, a `description`, a `contracts` list, a `rules` list, and its own `exclude`, `check_ignore` and `coverage_ignore`.

## Examples

The full config of the `shopping-notes` example project:

```yaml
project_name: shopping-notes
cli_version: 0.25.0
layout: multi-repo
agent_root: .
description: >-
  Web application for managing shopping lists, letting users note items to
  buy so they don't forget them at the store.
rules_unused:
  - graphql-conventions
packages:
  - package_name: shopping-notes-api
    package_short_name: api
    description: Fastify REST API backend for the shopping notes application.
    path: ../shopping-notes-api
    contracts:
      - path: contracts/api.openapi.yaml
        description: OpenAPI contract for the shopping notes API.
    rules:
      - commit-message-format
      - dependencies-rules
      - naming-convention
      - openapi-conventions
      - secret-handling
      - testing-rules
      - writing-style
  - package_name: shopping-notes-frontend
    package_short_name: frontend
    description: React SPA that lets users create and manage shopping notes against the API.
    path: ../shopping-notes-frontend
    contracts:
      - path: ../shopping-notes-frontend/contracts/figma.md
        description: Figma design references for frontend UI specs.
    rules:
      - commit-message-format
      - dependencies-rules
      - naming-convention
      - secret-handling
      - testing-rules
      - writing-style
  - package_name: shopping-notes-spex
    package_short_name: spex
    description: Spex framework. Holds the specs, rules, skills and config that govern the project.
    path: .
    rules:
      - commit-message-format
      - naming-convention
      - secret-handling
      - writing-style
```
