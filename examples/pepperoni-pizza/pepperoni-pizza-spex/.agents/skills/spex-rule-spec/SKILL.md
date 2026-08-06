---
name: spex-rule-spec
description: Create a new rule spec (RS) or edit an existing one by interviewing the user. Use when the user wants to create, extend, or update an RS spec — from a coding standard, a style guide, a security policy, an engineering decision, an existing code pattern, or a named existing spec.
---

# Create or Edit a Rule Spec (RS)

A rule spec (`RS`) describes _how the code and artifacts are built_: one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules captured as acceptance criteria. It is not tied to one feature — it applies repository-wide. Your job is to turn a fuzzy convention into one schema-valid spec by asking the user focused questions, proposing automated checks that verify it, then writing the file — or, when the user names an existing spec, to update it in place.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the spec's `id`.

The schema at `<workspace-root>/<project-name>-spex/schemas/rs-spec.schema.json` is authoritative for field rules and contains worked examples. Read it before drafting.

Writes a schema-valid YAML file to `<workspace-root>/<project-name>-spex/rs-specs/<spec-id>.yaml`

## FS vs RS

A cross-cutting rule all code must obey, enforceable repo-wide → RS. A specific capability the system must provide (an endpoint, integration, health check, performance target) → FS.

## Scope rule: one convention per spec

An RS spec captures exactly one coherent cross-cutting convention; its individual rules are the acceptance criteria. If the user describes several unrelated conventions, propose splitting them into multiple specs and confirm the split before writing.

## Step 0 — Establish create vs. edit

First decide whether this is a new spec or a change to one that already exists:

- If the user names an existing spec ID (for example `RS-SECRET-HANDLING`) or asks to update, amend, fix, or change a spec, this is an **edit**. Read `<workspace-root>/<project-name>-spex/rs-specs/<spec-id>.yaml` in full before doing anything else — this is the file you will modify, not replace. Also read its checks and any `script` bodies at `<workspace-root>/<project-name>-spex/scripts/<spec-id>/`. Confirm with the user what should change (revised or new acceptance criteria, new or adjusted checks, a scope change), applying the same field rules as Steps 3–4. Skip straight to Step 5, editing the existing file (and any check scripts) in place rather than starting over; preserve every untouched field, its ordering, and formatting.
- Otherwise, this is a **new spec** — continue with Step 0's source-gathering below, then Steps 1–4.

### Establish the source (new specs only)

Determine what the spec is based on before doing anything else:

- If the user attached or pasted a source (a style guide, a security policy, an engineering decision, or a code excerpt) — whether as text or as a file path/link — treat it as the primary input. Read the attached file or the pasted text in full and pre-fill every answer you can from it before asking anything.
- If the user provided no source and nothing to spec, ask first what convention they want to spec. Do not invent a convention or proceed on assumptions.

## Step 1 — Gather context first

Before asking the user anything, read what already exists so you don't ask for known answers or create duplicates:

- Read `<workspace-root>/<project-name>-spex/spex.yaml` for the project overview, the workspace repositories, and which rs-specs they already list. Repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role.
- Read `<workspace-root>/<project-name>-spex/rs-specs/` for related specs and to match their style. Reuse an existing spec rather than duplicating its convention.

## Step 2 — Clarify missing details with the user

Pre-fill every field you can from the source and existing context. Then ask the user only about what's missing or ambiguous — the rationale (what the convention protects against), the acceptance criteria, and which repositories it applies to. Confirm anything you inferred rather than assuming silently.

Do not set `sync_with_upstream` on a consumer spec unless the user explicitly asks; it governs global framework specs. Ask which repositories the convention applies to, then list the spec's `id` under each of those repositories' `rs_specs` in `spex.yaml`.

## Step 3 — Draft the fields

Follow the field rules in `<workspace-root>/<project-name>-spex/schemas/rs-spec.schema.json` and use its `examples` as the model for the YAML you write. Key points while drafting:

- `id` must start with `RS-` and convey the convention so it reads without opening the spec.
- Keep `description` short, simple, and concrete: a general summary of the rule and what it protects against. Do not restate individual rules already covered by `acceptance_criteria`.
- Write `acceptance_criteria` as verifiable statements, each beginning with a commitment level, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY. A rule spec is a global rule that always applies, so its criteria carry no implementation checkbox.
- Keep each criterion provable by an automated check or by AI inspection.

## Step 4 — Propose checks

For each acceptance criterion, propose how it can be verified automatically, then confirm the set with the user before writing. Pick the simplest check `type` that proves the criterion:

- `regexp` — match file contents. Use `negate: false` (default) when a match is a violation (forbidden pattern); use `negate: true` when a file with no match is a violation (required pattern). Narrow with `include`/`exclude` globs.
- `glob` — match file paths. Use it to forbid or require the presence of files (for example `**/*.bak`).
- `script` — run an external script named by `script:` (a bare filename, resolved at `scripts/<spec-id>/<script>`). Use it when a rule needs logic a regex can't express. Set `run: per-file` (default) to invoke it with the matched files as arguments. Set `run: per-repo` to invoke it once per repository, in that repository's directory, with no arguments. Set `run: workspace` to invoke it exactly once, with no arguments.

Guidance:

- Name each check in kebab-case, unique within the spec. A `script` check names its program with `script:` (just the filename), resolved at `scripts/<spec-id>/<script>` so checks from different specs never share a path. A repository can adjust an inherited check or add its own in `spex.yaml` via `checks_override` and `checks_additional`.
- Set `severity: warn` for advisory checks and `error` (default) for enforced ones.
- Not every criterion needs a check. When a criterion can only be judged by AI or human review, say so and leave it without a check rather than inventing a brittle one.
- When you propose a `script` check, also draft the file it references: a POSIX `sh` script that exits non-zero on violation. Store it at `scripts/<spec-id>/<check-name>.sh` and mirror the header style of existing scripts (`# <SPEC-ID> / <name>`). A `per-file` script receives the candidate files as arguments; a `per-repo` script receives none.
- A failing script MUST emit one finding per line on stdout following the script location contract: `<location><TAB><message>`, where `<location>` is a repository-relative file (optionally `<file>:<line>`) and `<message>` describes the violation. This gives script errors the same per-file precision as `regexp` and `glob` checks, so a `checks_preexisting_errors` baseline can suppress them per file. Emit a file-less finding (no location, leaving the part before the tab empty, or printing a line with no tab) only when the violation has no natural path, for example a rule about git history. Do not print a multi-line dump for one violation — emit one tab-separated line per location:
  ```sh
  printf '%s:%s\t%s\n' "$file" "$lineno" "matches forbidden pattern"
  ```
- When a script needs data from `spex.yaml` (the project name, the repository list), run `spex info` rather than parsing `spex.yaml` by hand — hand-parsing is brittle and duplicated across scripts. Use `spex info --print <field>` for a single top-level scalar (e.g. `spex info --print project_name`), or `spex info --json | jq ...` for anything else (e.g. `spex info --json | jq -r '.repositories[].path'`).

Draft the checks (and any script bodies) into the spec; you write them and offer edits in Step 5.

## Step 5 — Write, then offer edits

1. Write the spec to `<workspace-root>/<project-name>-spex/rs-specs/<spec-id>.yaml` — the filename must equal the `<spec-id>`. Use a literal block scalar (`>-`) for a multi-paragraph `description`. When editing an existing spec, change only what the user asked for and leave every other field, its ordering, and formatting untouched.
2. Write any `script` check bodies to the path referenced by `script:` (`<workspace-root>/<project-name>-spex/scripts/<spec-id>/<name>.sh`) and make them executable.
3. For a new spec, add its `id` under `rs_specs` in `spex.yaml` for each repository the user said it applies to.
4. Run `spex check` to validate compliance.
5. Tell the user the spec and checks you wrote (or the diff you applied, for an edit) and where, then ask whether they want to change anything further. Apply any requested edits in place.

## Example

```yaml
id: RS-NO-CONSOLE-LOG
title: No stray console.log statements
description: >-
  Stray console.log calls left over from debugging clutter production output
  and can leak internal state. Use the project logger instead.
acceptance_criteria:
  - "MUST NOT use `console.log` in application code — use the project logger"
  - "MUST log through the shared logger so output includes level and context"
checks:
  - name: no-console-log
    description: Stray console.log calls should use the project logger instead
    type: regexp
    pattern: "console\\.log\\("
    include:
      - "**/*.ts"
      - "**/*.tsx"
    exclude:
      - "**/*.test.ts"
    severity: error
```

## Step 6 — Point to the next skill

If there are no `spex check` errors related to the new spec, do nothing.
If there are errors:

- Errors can be fixed easily: offer to fix them
- Too many errors: offer to plan its implementation: run `spex-plan <spec-id>` with the spec's `id`. Note that it is better to start a new chat first to save context.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
