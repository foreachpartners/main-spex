# `spex-plan` skill

Creates an implementation plan markdown file for large-scale, complex features or significant system changes across one or more specs, to be executed by multiple AI agents in parallel. Loaded for breaking down major initiatives into development tasks, scoping high-effort engineering work, and planning multi-phase architectural rollouts.

## Acceptance criteria

- MUST create a plan at `<spex-dir>/plans/<date>-<slug>.md`, where `<date>` is today's real date (`YYYY-MM-DD`) and `<slug>` a short lowercase hyphenated summary within 50 characters, following the structure of `references/plan-example.md`.
- MUST capture the shape of a change, never its logic: new or changed method signatures, database and schema changes, domain models, and handler and endpoint signatures.
- MUST read the requested specs, the governing rules (`spex rules --package ... --ai`), the contracts, and the current code, grounding every interface it plans to change, extend, or create.
- MUST write the plan's metadata: `Specs`, `Packages`, and the applicable `Rules`.
- MUST break the work into ordered steps, splitting a step into tasks only for independent parallel work, so every task in a step has a disjoint write scope and no dependency on a sibling, and any dependency sits in a later step.
- MUST give every step and every task a `Status:` line, `not-started` or `completed` (a fresh plan is all `not-started`).
- MUST produce machine- and human-readable, deterministic output, describing only changes big enough to matter for review.
- MUST tell the user the path it wrote, offer to execute it with `spex-implement <spec-id>`, and note that `spex-implement` deletes the plan once every step is `completed`.
