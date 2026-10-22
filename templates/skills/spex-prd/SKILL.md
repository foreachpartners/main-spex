---
name: spex-prd
description: Write or change the project's PRD — its overall goal and main capabilities — by interviewing the user. Use when the user wants to create, extend, or change the PRD, or to describe the product before writing specs.
---

# Write or change the PRD

A PRD says _why_ the product is built and _what_ it does, at the level of the overall goal and its main capabilities. Details live downstream: acceptance criteria in specs, interfaces in contracts, conventions in rules, steps and dates in plans. A PRD is optional, one file per project. Your job is to interview the user about the product, then write the PRD — or, when one exists, change it in place.

## Paths

Replace `<spex-dir>` with the spex directory, printed by `spex info | spex jq -r .spex_dir`, and `<prd-file>` with the PRD's absolute path, printed by `spex info | spex jq -r '.prd // empty'`: the `prd` field of `spex.yaml`, `<spex-dir>/docs/prd.md` when it is unset. When the user keeps the PRD elsewhere, use the path they give. `<prd-path>` is `<prd-file>` relative to `<spex-dir>`, the value `prd` holds in `spex.yaml`.

The PRD is `<prd-file>`.

## Sections

The PRD starts with `# Product requirements`, then these sections in this order:

- `## Problem` — who hurts and why, in a few sentences.
- `## Goal` — the outcome the product delivers, one paragraph.
- `## Users` — one line per user or actor: who they are and what they come for.
- `## Success` — observable outcomes that show the product works, with numbers where the user has them.
- `## Capabilities` — the main blocks, one line each: a bold name and one sentence of user-visible outcome. Each breaks down into one or more specs later. Keep to about ten; more means the PRD has descended to the level of specs.
- `## Non-goals` — what the product deliberately does not do, one line each, with the reason when it is not obvious.

## Step 0 — Establish create vs edit

If `<prd-file>` exists, or the user asks to change the PRD, this is an **edit**: read the file in full, and interview only about what should change. Otherwise this is a **new PRD**.

## Step 1 — Gather context first

- If the user attached or pasted a source (a brief, a pitch, an RFD, a feature list) — as text, a file path, or a link — read it in full and pre-fill every answer you can from it.
- If there is no source and nothing to describe, ask first what the product is. Do not invent one.
- Read `<spex-dir>/spex.yaml` for the project and package descriptions, and `<spex-dir>/docs/glossary.md` for the terms to use.
- Edit only: run `spex specs --ai --output <tmp-path>`, then read `<tmp-path>` to learn which capabilities are already specced.

## Step 2 — Interview

Invoke the `spex-grill-me` skill with the Skill tool. Its subject is the product: walk the sections above in order, putting each answer you pre-filled in Step 1 to the user as your recommendation.

Keep the interview at the PRD's level. When the user volunteers a detail that belongs downstream, name its home (spec, contract, rule, or plan) and move on.

Resolve every open question here. A capability the user defers goes to `Non-goals`.

The step is done when the user confirms a shared understanding and every required section has an answer.

## Step 3 — Write, then offer edits

1. Write `<prd-file>`, following the structure of [`references/prd-example.md`](references/prd-example.md). When editing, change only what the user asked and leave every other section, its heading, and its order untouched.
2. When `spex.yaml` sets no `prd`, set `prd: <prd-path>` in it, after `description` (after `agent_root` when there is none), then run `spex check-schema --schema schemas/spex.schema.json --files spex.yaml` from `<spex-dir>` and fix any violation.
3. Tell the user the path you wrote and ask whether they want to change anything further. Apply requested edits in place.
4. When an edit removes, renames, or narrows a capability, list the specs it affects from Step 1 and ask whether to update them with `spex-spec`. Never change a spec here.

## Step 4 — Point to the next skill

After the PRD is written, offer to break its capabilities into specs: run `spex-spec from prd`. Note that it is better to start a new chat first to save context.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
