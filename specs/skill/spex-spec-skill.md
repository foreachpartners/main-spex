# `spex-spec` skill

Creates new specs, or extends and changes existing ones, by interviewing the user. Loaded when the user wants to extend or change existing specs or create new ones, from a PRD, an RFD, a feature idea, a business rule, or a technical capability the product needs. It turns a fuzzy idea or PRD into schema-valid specs, one file per capability, or updates the named existing spec in place.

## Acceptance criteria

- MUST read `<spex-dir>/schemas/spec.schema.json`, the authority for field rules with worked examples, before drafting, and write a new spec to `<spex-dir>/specs/<spec-id>.yaml`.
- MUST treat a specific capability the system must provide as a spec, and a cross-cutting rule all code must obey, enforceable package-wide, as a rule.
- MUST capture exactly one coherent capability per spec, its specifics being the acceptance criteria, and split a source that covers several capabilities into one spec each, confirming the split with the user.
- MUST treat the request as an edit when the user names an existing spec ID or asks to change a spec, editing that file in place and preserving every untouched field; otherwise as a new spec.
- MUST gather context before drafting: the user's source, `spex.yaml`, related specs (`spex specs --ai`), each package's contracts, and the `writing-style` rule.
- MUST pre-fill every field it can and ask the user only about what is missing or ambiguous, confirming anything it inferred.
- MUST draft `acceptance_criteria` as `{id, must|must_not|should|should_not|may, is_done}` objects ordered by commitment level, each `id` a kebab-case slug naming the behavior, and set `is_done: false` on every criterion it writes or adds, never on one the user did not ask it to change.
- MUST capture a concrete user decision verbatim, describe end behavior rather than implementation steps, and set `packages`, `blocked_by` (genuine blockers only), and `contract` (`single`, `for-each-ac`, or `none`).
- MUST name the owning package at the start of each criterion when `packages` lists more than one, and omit the package name for a single-package spec.
- MUST validate the result against `schemas/spec.schema.json` and tell the user the path it wrote, applying requested edits in place.
- MUST offer to draft each spec's contract with `spex-contract <spec-id>`.
