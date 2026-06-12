---
name: spex-create-tech-spec
description: Create a technical spec (TECH) by interviewing the user. Use when the user wants to author, draft, or extract a TECH spec from a coding standard, a style guide, a security policy, an engineering decision, or an existing code pattern. Asks targeted questions, proposes automated checks, then writes a schema-valid YAML file to <workspace_root>/spex/tech-specs/.
---

# Create a Technical Spec (TECH)

A technical spec (`TECH`) describes _how the code is built_: a single cross-cutting technical rule on code style, structure, testing, or security. It is not tied to one feature — it applies repository-wide. Your job is to turn a fuzzy convention into one schema-valid spec by asking the user focused questions, proposing automated checks that verify it, then writing the file.

Match the prose style of the existing specs and write the result into the spex repository's `<workspace_root>/spex/tech-specs/` directory.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace_root>` with the real folder name and `<spec-id>` with the spec's `id`. |

The schema at `<workspace_root>/spex/schemas/tech-spec.schema.json` is authoritative for field rules and contains worked examples. Read it before drafting.

## When to use this skill

Use it when the user wants to create, draft, or extract a TECH spec — from a coding standard, a style or naming guide, a security policy, a testing rule, or a pattern already present in the code. Do not use it for product behavior or business rules; those are BIZ specs.

## Scope rule: one rule per spec

A TECH spec captures exactly one cross-cutting rule. If the user describes several unrelated conventions, propose splitting them into multiple specs and confirm the split before writing.

## Step 0 — Establish the source

Determine what the spec is based on before doing anything else:

- If the user attached or pasted a source (a style guide, a security policy, an engineering decision, or a code excerpt) — whether as text or as a file path/link — treat it as the primary input. Read the attached file or the pasted text in full and pre-fill every answer you can from it before asking anything.
- If the user provided no source and no rule to spec, ask first what convention they want to spec. Do not invent a rule or proceed on assumptions.

## Step 1 — Gather context first

Before asking the user anything, read what already exists so you don't ask for known answers or create duplicates:

- Read `<workspace_root>/spex/spex.yml` for the project overview, the workspace repositories, and which tech-specs they already list.
- Read `<workspace_root>/spex/tech-specs/` for related specs and to match their style. Reuse an existing spec rather than duplicating its rule.

## Step 2 — Clarify missing details with the user

Pre-fill every field you can from the source and existing context. Then ask the user only about what's missing or ambiguous — the rationale (what the rule protects against), the acceptance criteria, and which repositories it applies to. Confirm anything you inferred rather than assuming silently.

Do not set `readonly` or `include_for_spex` on a consumer spec unless the user explicitly asks; those govern global framework specs. Ask whether the rule should apply to every repository (`include_for_all_repos: true`) or only to specific ones.

## Step 3 — Draft the fields

Follow the field rules in `<workspace_root>/spex/schemas/tech-spec.schema.json` and use its `examples` as the model for the YAML you write. Key points while drafting:

- `id` must start with `TECH-` and convey the rule so it reads without opening the spec.
- Write `acceptance_criteria` as verifiable statements, each starting with a commitment level, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY.
- Keep each criterion provable by an automated check or by AI inspection.

## Step 4 — Propose checks

For each acceptance criterion, propose how it can be verified automatically, then confirm the set with the user before writing. Pick the simplest check `type` that proves the criterion:

- `regexp` — match file contents. Use `negate: false` (default) when a match is a violation (forbidden pattern); use `negate: true` when a file with no match is a violation (required pattern). Narrow with `include`/`exclude` globs.
- `glob` — match file paths. Use it to forbid or require the presence of files (for example `**/*.bak`).
- `script` — run `scripts/<name>.sh` once per matched file. Use it when a rule needs logic a regex can't express.
- `command` — run `scripts/<name>.sh` once per repository. Use it for repository-wide checks (for example inspecting git history).

Guidance:

- Name each check in kebab-case, unique within the spec; for `script`/`command` the name is the script filename (`scripts/<name>.sh`).
- Set `severity: warn` for advisory checks and `error` (default) for enforced ones.
- Not every criterion needs a check. When a criterion can only be judged by AI or human review, say so and leave it without a check rather than inventing a brittle one.
- When you propose a `script` or `command` check, also draft the `scripts/<name>.sh` file: a POSIX `sh` script, `set -eu`, that exits non-zero with a clear message on violation. Mirror the header style of existing scripts (`# <SPEC-ID> / <name>`).

Show the proposed checks (and any script bodies) to the user and confirm them before writing.

## Step 5 — Confirm, then write

1. Show the drafted spec and proposed checks to the user for confirmation. Apply edits if needed.
2. Write the spec to `<workspace_root>/spex/tech-specs/<spec-id>.yaml` — the filename must equal the `<spec-id>`.
3. Write any `script`/`command` check bodies to `<workspace_root>/spex/scripts/<name>.sh` and make them executable.
4. Use a literal block scalar (`>-`) for a multi-paragraph `description`.
5. Validate the spec against `<workspace_root>/spex/schemas/tech-spec.schema.json`.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
