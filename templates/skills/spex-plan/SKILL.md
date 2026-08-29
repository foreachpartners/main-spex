---
name: spex-plan
description: Creates a comprehensive implementation plan md file for large-scale, complex features or significant system changes across one or more specs. Use for breaking down major initiatives into development tasks, scoping high-effort engineering work, and planning multi-phase architectural rollouts.
---

# Primary Directive

Your goal is to create a new implementation plan markdown file from requested specs to be executed by multiple AI agents working in parallel.

A plan captures the shape of a change — new or changed method signatures, database/schema changes, domain models, handler/endpoint signatures — never its logic. You must read `references/plan-example.md` and follow its structure.

You must read requested specs, governing rules, contracts, inspect the current code and draft a plan.

Your output must be machine and human-readable, deterministic, and structured for autonomous execution by other AI systems or humans.

# Gather context

Run `spex specs --ai --id <spec-id-a> <spec-id-b>` for every planned spec. Confirm with the user how to handle any unresolved blocker.

Resolve a `Repositories scope` for requested changes.

Run `spex rules --repo <repo-a> <repo-b> --ai --output <tmp-path>` for the resolved `Repositories scope`, then read `<tmp-path>` for the governing rules.

Read each affected repository's information in `Repositories scope` from `spex.yaml`.

Read related contracts in `<workspace-root>/<project-name>-spex/contracts/`.

Read all existing code related to changes: domain models, database schema, repositories, handlers, endpoints, components.

Ground every interface you plan to change, extend or create.

# Create the plan file

Create a spec file in `<workspace-root>/<project-name>-spex/plans/<date>-<slug>.md`. `<date>` is today's real date (`YYYY-MM-DD`). `<slug>` is a short lowercase hyphenated summary. The filename stem must validate against `plan-filename.schema.json` and stay within 50 characters.

If more than one spec is provided, pick a leading one for `<spec-id>` and generate the slug based on the main changes.

# Write plan metadata

Write `Specs` that the user provided.

Write `Repositories scope` based on changes we need to implement (don't include spex itself).

Write `Rules` in the plan from the governing rules already read in `<tmp-path>` that are applicable to changes.

# Write the steps and tasks

Break the work into ordered steps. Within a step, split into tasks only when there is more than one independent unit of work for AI to do in parallel:

- Every task in one step MUST be safe to run in parallel: it must not depend on another task's output in the same step, and must not edit the same files as another task in that step.
- Put work that depends on an earlier result in a later step — a schema or contract change comes before the code that consumes it.
- Give each task a disjoint write scope (its own files) and state which repository it runs in.

Describe only the shape of each change, never its logic:

- A new method: its full signature as code, labeled with its type and file.
- A changed method: the same, showing before and after.
- A table added or changed: a Markdown table, one row per column, with type and constraints.
- A domain model added or changed: a fenced code block with its fields and types, no logic.
- A new or changed handler/endpoint: its route and request/response types, not its body.

Include only changes big enough to matter for review. Omit renames, minor internal helpers, and anything a reviewer would not need to see to approve the shape of the change.

Give every step and every task its own `Status:` line, `not-started` or `completed`; a fresh plan is all `not-started`.

Done when: every step and task has a `Status:` line, every write scope is disjoint within its step, and every dependency sits in a later step than what it depends on.

# Report back

Tell the user the path you wrote (or the change you applied), walk through the step boundaries and which tasks run in parallel, then apply any requested edits in place. Offer to execute it: run `spex-implement <spec-id>`, noting it's better to start a new chat first to save context.

Mention that the plan is a working document, not a permanent artifact: `spex-implement` deletes it once every step is `completed`, since the spec and code then record the outcome more accurately.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
