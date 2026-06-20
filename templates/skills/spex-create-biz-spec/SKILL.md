---
name: spex-create-biz-spec
description: Create one or more business specs (BIZ) by interviewing the user. Use when the user wants to author, draft, or extract BIZ specs from a PRD, an RFD, a feature idea, or a business rule. Asks targeted questions, then writes schema-valid YAML files to <workspace-root>/spex/biz-specs/.
---

# Create Business Specs (BIZ)

A business spec (`BIZ`) describes _how the product behaves_: one coherent product capability or behavior, with its individual rules captured as verifiable acceptance criteria, extracted from the PRD. A PRD usually covers several capabilities, so a single source can produce one or more specs. Your job is to turn a fuzzy idea or PRD into schema-valid specs by asking the user focused questions, then writing one file per capability.

Match the prose style of the existing specs and write the result into the spex repository's `<workspace-root>/spex/biz-specs/` directory.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the spec's `id`.

The schema at `<workspace-root>/spex/schemas/biz-spec.schema.json` is authoritative for field rules and contains worked examples. Read it before drafting.

## When to use this skill

Use it when the user wants to create, draft, or extract BIZ specs — from a PRD, an RFD, a feature request, or a business rule. Do not use it for technical/cross-cutting conventions; those are TECH specs.

## Scope rule: one capability per spec

A BIZ spec captures exactly one coherent capability or behavior; its individual rules are the acceptance criteria. A PRD typically breaks down into many business specs, so a single source can yield several. When the source covers more than one capability, identify each distinct one, propose the split as a list of specs (one `id` and one-line summary each), and confirm it with the user before drafting. Write each confirmed capability as its own spec file.

## Step 0 — Establish the source

Determine what the specs are based on before doing anything else:

- If the user attached or pasted a source (a PRD, an RFD, or a feature description) — whether as text or as a file path/link — treat it as the primary input. Read the attached file or the pasted text in full and pre-fill every answer you can from it before asking anything.
- If the user provided no source and nothing to spec, ask first what they want to spec. Do not invent a behavior or proceed on assumptions.

## Step 1 — Gather context first

Before asking the user anything, read what already exists so you don't ask for known answers or create duplicates:

- Read `<workspace-root>/spex/spex.yaml` for the project overview. Repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role and use that to determine the spec's implementation `scope`.
- Read `<workspace-root>/spex/biz-specs/` for related specs
- Use the repository `description`s in `spex.yaml` to find the repository that holds the contract/interface definitions (it may be named anything — for example `contracts`, `proto`, or `api`), resolve its `path`, and read any related schemas there.

## Step 2 — Identify the capabilities and clarify missing details

Break the source into distinct capabilities — one spec per capability (see the scope rule above). When there is more than one, confirm the split with the user before drafting.

Pre-fill every field you can from the source and existing context. Then ask the user only about what's missing or ambiguous — unresolved details or acceptance criteria you cannot infer. Confirm anything you inferred rather than assuming silently.

## Step 3 — Draft the fields

Draft each spec following the field rules in `<workspace-root>/spex/schemas/biz-spec.schema.json`, using its `examples` as the model for the YAML you write. Key points while drafting:

- `id` must start with `BIZ-` and convey the capability so it reads without opening the spec.
- Write `acceptance_criteria` as verifiable statements, each starting with a commitment level, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY. These are the spec's individual rules.
- Set `scope` to the repositories that implement the capability, each matching a `repository_name` in `spex.yaml`.
- Keep the `description` focused on user-visible behavior; leave technical implementation to the plan.

## Step 4 — Confirm, then write

For each drafted spec:

1. Show the drafted spec to the user for confirmation. Apply edits if needed.
2. Write it to `<workspace-root>/spex/biz-specs/<spec-id>.yaml` — the filename must equal the `<spec-id>`.
3. Use literal block scalars for multi-paragraph `description`.
4. Validate the result against `<workspace-root>/spex/schemas/biz-spec.schema.json`.

## Step 5 — Point to the next skill

After the specs are written, prompt the user to plan each one's implementation in a new chat window to save context: run `spex-create-plan <spec-id>` with the spec's `id`. Do not offer to run it in the current chat.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
