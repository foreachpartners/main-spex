# `spex-rule` skill

Creates a new rule or edits an existing one by interviewing the user. Loaded when the user wants to create, extend, or update a rule, from a coding standard, a style guide, a security policy, an engineering decision, an existing code pattern, or a named existing rule. It turns a fuzzy convention into one schema-valid rule with automated checks that verify it, or updates the named existing rule in place.

## Acceptance criteria

- MUST read `<spex-dir>/schemas/rule.schema.json`, the authority for field rules with worked examples, before drafting, and write to `<spex-dir>/rules/<rule-id>.yaml`.
- MUST treat a cross-cutting rule all code must obey, enforceable package-wide, as a rule, and a specific capability the system must provide as a spec.
- MUST capture exactly one coherent convention per rule, its individual rules being the acceptance criteria, and split several unrelated conventions into one rule each, confirming the split with the user.
- MUST propose a small, single-criterion convention with no theme of its own as one more acceptance criterion in the catch-all bucket — `rules/general-rules.yaml` (`id: general-rules`) project-wide, or `rules/general-rules-<package-short-name>.yaml` (`id: general-rules-<package-short-name>`) for one package — instead of a dedicated file, and confirm the path with the user.
- MUST treat the request as an edit when the user names an existing rule ID or asks to change a rule, reading the rule and its scripts in full and preserving every untouched field; otherwise as a new rule.
- MUST gather context before drafting: the user's source, `spex.yaml`, and existing rules, reusing a rule rather than duplicating its convention.
- MUST pre-fill every field it can and ask the user only about what is missing or ambiguous (rationale, acceptance criteria, applicable packages), confirming anything it inferred, and list the rule's `id` under each applicable package's `rules` in `spex.yaml`.
- MUST write `acceptance_criteria` as verifiable statements ordered by commitment level, each provable by an automated check or AI inspection, with no implementation checkbox.
- MUST propose a check per acceptance criterion, picking the simplest `type` (`regexp`, `glob`, `ast-grep`, or `script`) with `negate`, `include`/`exclude`, `severity`, and for `script` a `run` of `per-file`, `per-package`, or `project`, confirming the set with the user.
- MUST store a `script` check's POSIX `sh` body beside the rule in `rules/<rule-id>/`, make it executable, have it print one `<location><TAB><message>` finding per line, and read `spex.yaml` data with `spex jq` rather than parsing it by hand.
- MUST leave a criterion only AI or human review can judge without a check and say so.
- MUST run `spex check` and offer to fix easy errors, or to plan a large fix with `spex-plan <rule-id>`.
