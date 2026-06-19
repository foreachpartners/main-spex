---
name: spex-create-tech-spec
description: Create a technical spec (TECH) by interviewing the user. Use when the user wants to author, draft, or extract a TECH spec from a coding standard, a style guide, a security policy, an engineering decision, or an existing code pattern. Asks targeted questions, proposes automated checks, then writes a schema-valid YAML file to <workspace-root>/spex/tech-specs/.
---

# Create a Technical Spec (TECH)

A technical spec (`TECH`) describes _how the code is built_: a single cross-cutting technical rule on code style, structure, testing, or security. It is not tied to one feature — it applies repository-wide. Your job is to turn a fuzzy convention into one schema-valid spec by asking the user focused questions, proposing automated checks that verify it, then writing the file.

Match the prose style of the existing specs and write the result into the spex repository's `<workspace-root>/spex/tech-specs/` directory.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the spec's `id`.

The schema at `<workspace-root>/spex/schemas/tech-spec.schema.json` is authoritative for field rules and contains worked examples. Read it before drafting.

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

- Read `<workspace-root>/spex/spex.yaml` for the project overview, the workspace repositories, and which tech-specs they already list.
- Read `<workspace-root>/spex/tech-specs/` for related specs and to match their style. Reuse an existing spec rather than duplicating its rule.

## Step 2 — Clarify missing details with the user

Pre-fill every field you can from the source and existing context. Then ask the user only about what's missing or ambiguous — the rationale (what the rule protects against), the acceptance criteria, and which repositories it applies to. Confirm anything you inferred rather than assuming silently.

Do not set `readonly` or `include_for_spex` on a consumer spec unless the user explicitly asks; those govern global framework specs. Ask whether the rule should apply to every repository (`include_for_all_repos: true`) or only to specific ones.

## Step 3 — Draft the fields

Follow the field rules in `<workspace-root>/spex/schemas/tech-spec.schema.json` and use its `examples` as the model for the YAML you write. Key points while drafting:

- `id` must start with `TECH-` and convey the rule so it reads without opening the spec.
- Write `acceptance_criteria` as verifiable statements, each starting with a commitment level, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY.
- Keep each criterion provable by an automated check or by AI inspection.

## Step 4 — Propose checks

For each acceptance criterion, propose how it can be verified automatically, then confirm the set with the user before writing. Pick the simplest check `type` that proves the criterion:

- `regexp` — match file contents. Use `negate: false` (default) when a match is a violation (forbidden pattern); use `negate: true` when a file with no match is a violation (required pattern). Narrow with `include`/`exclude` globs.
- `glob` — match file paths. Use it to forbid or require the presence of files (for example `**/*.bak`).
- `script` — run an external script named by `script:` (a bare filename, resolved at `scripts/<spec-id>/<script>`). Use it when a rule needs logic a regex can't express. Set `run: per-file` (default) to invoke it with the matched files as arguments, `run: per-repo` to invoke it once in the repository directory with no arguments (for example inspecting git history), or `run: workspace` to invoke it once across the whole workspace with every repository's matched files as arguments (for example reasoning about cross-repository state).

Guidance:

- Name each check in kebab-case, unique within the spec. A `script` check names its program with `script:` (just the filename), resolved at `scripts/<spec-id>/<script>` so checks from different specs never share a path. A repository can adjust an inherited check or add its own in `spex.yaml` via `checks_override` and `checks_additional`.
- Set `severity: warn` for advisory checks and `error` (default) for enforced ones.
- Not every criterion needs a check. When a criterion can only be judged by AI or human review, say so and leave it without a check rather than inventing a brittle one.
- When you propose a `script` check, also draft the file it references: a POSIX `sh` script that exits non-zero with a clear message on violation. Store it at `scripts/<spec-id>/<check-name>.sh` and mirror the header style of existing scripts (`# <SPEC-ID> / <name>`). A `per-file` script receives the candidate files as arguments; a `per-repo` script receives none.

Show the proposed checks (and any script bodies) to the user and confirm them before writing.

## Step 5 — Confirm, then write

1. Show the drafted spec and proposed checks to the user for confirmation. Apply edits if needed.
2. Write the spec to `<workspace-root>/spex/tech-specs/<spec-id>.yaml` — the filename must equal the `<spec-id>`.
3. Write any `script` check bodies to the path referenced by `script:` (`<workspace-root>/spex/scripts/<spec-id>/<name>.sh`) and make them executable.
4. Use a literal block scalar (`>-`) for a multi-paragraph `description`.
5. Validate the spec against `<workspace-root>/spex/schemas/tech-spec.schema.json`.

## Step 6 — Point to the next skill

After the spec is written, prompt the user to plan its implementation: run `spex-create-plan <spec-id>` with the spec's `id`.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
