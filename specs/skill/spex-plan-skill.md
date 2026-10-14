# `spex-plan` skill

Creates an implementation plan markdown file for large-scale, complex features or significant system changes across one or more specs, to be executed by multiple AI agents working in parallel. Loaded for breaking down major initiatives into development tasks, scoping high-effort engineering work, and planning multi-phase architectural rollouts.

## Acceptance criteria

### Purpose

- MUST create a new implementation plan markdown file from the requested specs.
- MUST capture the shape of a change, never its logic: new or changed method signatures, database/schema changes, domain models, handler/endpoint signatures.
- MUST read `references/plan-example.md` and follow its structure.
- MUST read the requested specs, the governing rules, and the contracts, inspect the current code, and draft the plan.
- MUST produce output that is machine and human readable, deterministic, and structured for autonomous execution by other AI systems or humans.

### Gather context

- MUST run `spex specs --ai --id <spec-id-a> <spec-id-b>` for every planned spec, and confirm with the user how to handle any unresolved blocker.
- MUST resolve the `Packages` for the requested changes.
- MUST run `spex rules --package <package-a> <package-b> --ai --output <tmp-path>` for the resolved `Packages`, then read `<tmp-path>` for the governing rules.
- MUST read each affected package's information in `Packages` from `spex.yaml`.
- MUST read the related contracts in `<spex-dir>/contracts/`.
- MUST read all existing code related to the changes: domain models, database schema, repositories, handlers, endpoints, components.
- MUST ground every interface it plans to change, extend, or create.

### Create the plan file

- MUST create the plan at `<spex-dir>/plans/<date>-<slug>.md`, where `<date>` is today's real date (`YYYY-MM-DD`) and `<slug>` is a short lowercase hyphenated summary.
- MUST validate the filename stem against `plan-filename.schema.json`, and keep it within 50 characters.
- MUST pick a leading spec for `<spec-id>` and generate the slug from the main changes when more than one spec is provided.

### Write plan metadata

- MUST write `Specs`, the specs the user provided.
- MUST write `Packages` based on the changes to implement, without spex itself.
- MUST write `Rules` from the governing rules already read in `<tmp-path>` that are applicable to the changes.

### Write the steps and tasks

- MUST break the work into ordered steps.
- MUST split a step into tasks only when there is more than one independent unit of work for AI to do in parallel.
- MUST make every task in one step safe to run in parallel: it does not depend on another task's output in the same step, and does not edit the same files as another task in that step.
- MUST put work that depends on an earlier result in a later step; a schema or contract change comes before the code that consumes it.
- MUST give each task a disjoint write scope (its own files) and state which package it runs in.
- MUST describe a new method as its full signature as code, labeled with its type and file.
- MUST describe a changed method the same way, showing before and after.
- MUST describe a table added or changed as a Markdown table, one row per column, with type and constraints.
- MUST describe a domain model added or changed as a fenced code block with its fields and types, no logic.
- MUST describe a new or changed handler or endpoint by its route and request/response types, not its body.
- MUST include only changes big enough to matter for review, omitting renames, minor internal helpers, and anything a reviewer would not need to see to approve the shape of the change.
- MUST give every step and every task its own `Status:` line, `not-started` or `completed`; a fresh plan is all `not-started`.
- MUST be done only when every step and task has a `Status:` line, every write scope is disjoint within its step, and every dependency sits in a later step than what it depends on.

### Report back

- MUST tell the user the path it wrote (or the change it applied), walk through the step boundaries and which tasks run in parallel, then apply any requested edits in place.
- MUST offer to execute the plan with `spex-implement <spec-id>`, noting that starting a new chat first saves context.
- MUST mention that the plan is a working document, not a permanent artifact: `spex-implement` deletes it once every step is `completed`, since the spec and code then record the outcome more accurately.
- MUST follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else, with no filler, hedging, motivational, or apologetic phrasing.
