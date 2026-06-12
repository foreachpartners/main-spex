---
name: spex-create-biz-spec
description: Create a business spec (BIZ) by interviewing the user. Use when the user wants to author, draft, or extract a BIZ spec from a PRD, an RFD, a feature idea, or a business rule. Asks targeted questions, then writes a schema-valid YAML file to <workspace_root>/spex/biz-specs/.
---

# Create a Business Spec (BIZ)

A business spec (`BIZ`) describes _how the product behaves_: a single business rule with verifiable acceptance criteria, extracted from the PRD. Your job is to turn a fuzzy idea into one schema-valid spec by asking the user focused questions, then writing the file.

Match the prose style of the existing specs and write the result into the spex repository's `<workspace_root>/spex/biz-specs/` directory.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace_root>` with the real folder name and `<spec-id>` with the spec's `id`.

The schema at `<workspace_root>/spex/schemas/biz-spec.schema.json` is authoritative for field rules and contains worked examples. Read it before drafting.

## When to use this skill

Use it when the user wants to create, draft, or extract a BIZ spec — from a PRD excerpt, a feature request, or a single business rule. Do not use it for technical/cross-cutting rules; those are TECH specs.

## Scope rule: one rule per spec

A BIZ spec captures exactly one business rule. If the user describes several behaviors, propose splitting them into multiple specs and confirm the split before writing. A PRD typically breaks down into many business specs.

## Step 0 — Establish the source

Determine what the spec is based on before doing anything else:

- If the user attached or pasted a source (a PRD, an RFD, or a feature description) — whether as text or as a file path/link — treat it as the primary input. Read the attached file or the pasted text in full and pre-fill every answer you can from it before asking anything.
- If the user provided no source and no rule to spec, ask first what they want to spec. Do not invent a behavior or proceed on assumptions.

## Step 1 — Gather context first

Before asking the user anything, read what already exists so you don't ask for known answers or create duplicates:

- Read `<workspace_root>/spex/spex.yml` for the project overview and use that to determine the spec's implementation `scope`.
- Read `<workspace_root>/spex/biz-specs/` for related specs
- Read `<workspace_root>/spex/contract/` for schemas if exists

## Step 2 — Clarify missing details with the user

Pre-fill every field you can from the source and existing context. Then ask the user only about what's missing or ambiguous — unresolved details or acceptance criteria you cannot infer. Confirm anything you inferred rather than assuming silently.

## Step 3 — Draft the fields

Follow the field rules in `<workspace_root>/spex/schemas/biz-spec.schema.json` and use its `examples` as the model for the YAML you write. Key points while drafting:

Draft spec for user

## Step 4 — Confirm, then write

1. Show the drafted spec to the user for confirmation. Apply edits if needed.
2. Write it to `<workspace_root>/spex/biz-specs/<spec-id>.yaml` — the filename must equal the `<spec-id>`.
3. Use literal block scalars for multi-paragraph `description`.
4. Validate the result against `<workspace_root>/spex/schemas/biz-spec.schema.json`.

## Step 5 — Point to the next skill

After the spec is written, prompt the user to plan its implementation: run `spex-create-plan <spec-id>` with the spec's `id`.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
