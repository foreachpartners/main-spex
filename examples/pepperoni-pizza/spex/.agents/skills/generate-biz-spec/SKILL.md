---
name: generate-biz-spec
description: Generate a business spec (BIZ) by interviewing the user. Use when the user wants to author, draft, or extract a BIZ spec from a PRD, a feature idea, or a business rule. Asks targeted questions, then writes a schema-valid YAML file to <workspace_root>/spex/biz-specs/.
---

# Generate a Business Spec (BIZ)

A business spec (`BIZ`) describes _how the product behaves_: a single business rule with verifiable acceptance criteria, extracted from the PRD. Your job is to turn a fuzzy idea into one schema-valid spec by asking the user focused questions, then writing the file.

Match the prose style of the existing specs and write the result into the spex repository's `biz-specs/` directory.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace_root>` with the real folder name and `<spec-id>` with the spec's `id`.

| What | Path |
|------|------|
| Spex repository | `<workspace_root>/spex/` |
| BIZ specs directory | `<workspace_root>/spex/biz-specs/` |
| This spec's file | `<workspace_root>/spex/biz-specs/<spec-id>.yaml` |
| TECH specs (style reference) | `<workspace_root>/spex/tech-specs/` |
| Workspace config | `<workspace_root>/spex/spex.yml` |

The field rules in this skill are authoritative; you do not need a local schema file. `<workspace_root>/spex/biz-specs/` may not exist yet — create it when you write the first spec.

## When to use this skill

Use it when the user wants to create, draft, or extract a BIZ spec — from a PRD excerpt, a feature request, or a single business rule. Do not use it for technical/cross-cutting rules; those are TECH specs.

## Scope rule: one rule per spec

A BIZ spec captures exactly one business rule. If the user describes several behaviors, propose splitting them into multiple specs and confirm the split before writing. A PRD typically breaks down into many business specs.

## Step 1 — Gather context first

Before asking the user anything, read what already exists so you don't ask for known answers or create duplicates:

- List `<workspace_root>/spex/biz-specs/` to learn existing Spec IDs, naming style, and prose tone (skip if the directory does not exist yet).
- Skim `<workspace_root>/spex/tech-specs/` for the project's writing style.
- If the user pasted or referenced a PRD, read it and pre-fill answers from it.

## Step 2 — Interview the user

Ask only for what you cannot infer. Group related questions and ask in a short numbered list so the user can answer in one pass. Confirm anything you inferred rather than assuming silently.

Cover these areas:

1. **Behavior** — What single business rule should the product enforce? What user-visible behavior is expected?
2. **Actor & trigger** — Who is affected (which user/role), and what action or event triggers this behavior?
3. **Problem & value** — What problem does this solve and why does it matter? (Feeds the description.)
4. **Acceptance criteria** — What must be true for this to be correct? Probe for edge cases, error handling, and prohibitions (what must _not_ happen).
5. **Dependencies** — Does this require other BIZ specs to ship first? Collect their Spec IDs.
6. **Status** — Is this `active`, `planned`, `blocked`, or `deprecated`? Default to `planned` for new, not-yet-built behavior.

Keep `description` business-level. Technical implementation details belong in a plan, not in the spec.

## Step 3 — Draft the fields

### `id`

- Fixed prefix `BIZ-`, then uppercase English letters and hyphens only: `^BIZ-[A-Z]+(-[A-Z]+)*$`.
- Max 40 characters total.
- The name must convey the spec's contents so it's understandable without opening the file. Example: `BIZ-PLACE-PIZZA-ORDER`.
- Ensure it does not collide with an existing file in `<workspace_root>/spex/biz-specs/`.

### `title`

- One imperative, directive line. No filler.
- Example: `Allow customers to place an order from their cart`.

### `description`

- Multi-paragraph narrative: the problem, who is affected, and the expected user-visible behavior.
- Plain business language. No implementation details.

### `acceptance_criteria`

- Each item starts with a commitment level: `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, or `MAY` (regex `^(MUST NOT|MUST|SHOULD NOT|SHOULD|MAY) .+`).
- Order strictly by commitment level: MUST, then MUST NOT, then SHOULD, then SHOULD NOT, then MAY.
- Each criterion must be independently verifiable. Avoid vague qualifiers like "fast" or "user-friendly".
- At least one criterion is required.

### `depends_on` (optional)

- Array of other BIZ Spec IDs that must ship before this one. Omit if none.

### `status` (optional)

- One of `active`, `planned`, `blocked`, `deprecated`. Omitting means `planned`.

## Step 4 — Confirm, then write

1. Show the drafted spec to the user and confirm the `id`, criteria, and split (if any).
2. Write it to `<workspace_root>/spex/biz-specs/<spec-id>.yaml` — the filename must equal the `id`.
3. Use literal block scalars for multi-paragraph `description`.

## Template

```yaml
id: BIZ-PLACE-PIZZA-ORDER
title: Allow customers to place an order from their cart
description: >-
  Customers who have added items to their cart need a way to complete a
  purchase. When a customer confirms checkout, the system creates an order from
  the current cart contents, captures payment, and empties the cart.

  This affects all authenticated shoppers. After a successful order, the
  customer sees an order confirmation with a unique order number.
acceptance_criteria:
  - "MUST create an order only when the cart contains at least one item"
  - "MUST capture payment before the order is marked as confirmed"
  - "MUST empty the customer's cart after the order is successfully placed"
  - "MUST NOT place an order when payment capture fails"
  - "SHOULD display an order confirmation with a unique order number"
depends_on:
  - BIZ-CART-ADD-ITEM
status: planned
```

## Quality bar

- One rule per spec.
- `id` is unique, prefixed `BIZ-`, and ≤ 40 characters.
- Every acceptance criterion starts with a commitment level and is verifiable.
- Criteria are ordered by commitment level.
- Description stays business-level, no implementation detail.
- File is written to `<workspace_root>/spex/biz-specs/<spec-id>.yaml` and the filename matches the `id`.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
