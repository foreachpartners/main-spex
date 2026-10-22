# `spex-prd` skill

Creates or changes the project's PRD, its overall goal and main capabilities, by interviewing the user with [`spex-grill-me`](spex-grill-me-skill.md). Loaded when the user wants to create, extend, or change the PRD, or to describe the product before writing specs. It turns a product idea into a short PRD that [`spex-spec`](spex-spec-skill.md) later breaks down into specs.

## Acceptance criteria

- MUST locate the PRD, one file per project, at the absolute path `spex info` prints as `prd`, defaulting to `docs/prd.md` in the spex directory when `prd` is unset, or at the path the user gives.
- MUST set `prd` in `spex.yaml` to the PRD's path relative to the spex directory, after `description` (after `agent_root` when there is none), when it writes a PRD and `prd` is unset, and validate `spex.yaml` against `schemas/spex.schema.json`.
- MUST treat the request as an edit when the PRD file exists or the user asks to change the PRD, reading it in full, interviewing only about what should change, and preserving every untouched section, heading, and order; otherwise as a new PRD.
- MUST gather context before interviewing: the user's source read in full, `spex.yaml`, the glossary, and, for an edit, the existing specs (`spex specs --ai`).
- MUST ask the user what the product is when there is no source and no product to describe, never inventing one.
- MUST interview the user by invoking the `spex-grill-me` skill, walking the PRD's sections in order and pre-filling every answer it can from the gathered context.
- MUST write the PRD only after the user confirms a shared understanding and every required section has an answer.
- MUST write the PRD under the title `Product requirements`, then the headings `Problem`, `Goal`, `Users`, `Success`, `Capabilities`, and `Non-goals` in this order, following the structure of `references/prd-example.md`.
- MUST write `Problem` in a few sentences, `Goal` as one paragraph, `Users` as one line per user or actor, `Success` as observable outcomes with numbers where the user has them, and `Non-goals` as one line each with the reason when it is not obvious.
- MUST write each capability as a bold name and one sentence of user-visible outcome.
- MUST name where a detail the user volunteers belongs (spec, contract, rule, or plan) and keep the interview at the PRD's level.
- MUST resolve every open question during the interview, moving a deferred capability to `Non-goals`.
- MUST list the existing specs a change affects when an edit removes, renames, or narrows a capability, and ask whether to update them with `spex-spec`, never changing them itself.
- MUST tell the user the path it wrote and ask whether to change anything further, applying requested edits in place.
- MUST offer to break the PRD into specs with `spex-spec from prd`, noting that starting a new chat first saves context.
- SHOULD NOT list more than about ten capabilities, a sign the PRD has descended to the level of specs.
