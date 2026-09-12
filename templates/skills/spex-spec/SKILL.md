---
name: spex-spec
description: Create new specs or extend/change existing ones by interviewing the user. Use when the user wants to extend/change existing specs or create new ones — from a PRD, an RFD, a feature idea, a business rule, or a technical capability the product needs.
---

# Create, extend or change Specs

A spec describes _a capability the system must provide_: one coherent product behavior, or a technical implementation needed to support the product's rules, with its specifics captured as verifiable acceptance criteria. Most are extracted from the PRD; some capture supporting technical capabilities. A single source usually covers several capabilities, so it can produce one or more specs. Your job is to turn a fuzzy idea or PRD into schema-valid specs by asking the user focused questions, then writing one file per capability — or, when the user names an existing spec, to update/extend it in place.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the spec's `id`.

The schema at `<workspace-root>/<project-name>-spex/schemas/spec.schema.json` is authoritative for field rules and contains worked examples. Read it before drafting.

For new specs write the result into the spex repository's `<workspace-root>/<project-name>-spex/specs/<spec-id>.yaml`.

## Spec vs Rule

A specific capability the system must provide (an endpoint, integration, health check, performance target) -> spec. A cross-cutting rule all code must obey, enforceable repo-wide -> rule.

## Scope rule: one capability per spec

A spec captures exactly one coherent capability; its specifics are the acceptance criteria. A PRD typically breaks down into many specs, so a single source can yield several. When the source covers more than one capability, identify each distinct one, propose the split as a list of specs (one `id` and one-line summary each), and confirm it with the user before drafting. Write each confirmed capability as its own spec file.

## Step 0 — Establish create vs edit

First decide whether this is a new spec or a change to one that already exists:

- If the user names an existing spec ID (for example `place-pizza-order`) or asks to update, extend, fix, or change a spec, this is an **edit**. Read `<workspace-root>/<project-name>-spex/specs/<spec-id>.yaml` in full before doing anything else — this is the file you will modify, not replace. Confirm with the user what should change (new or revised acceptance criteria, a scope change, a status change), applying the same field rules as Step 3. Skip straight to Step 4, editing the existing file in place rather than writing a new one; preserve every untouched field.
- Otherwise, this is a **new spec** — continue with Step 1.

## Step 1 — Gather context first

Determine what the specs are based on before doing anything else:

- If the user attached or pasted a source (a PRD, an RFD, or a feature description) — whether as text or as a file path/link — treat it as the primary input. Read the attached file or the pasted text in full and pre-fill every answer you can from it before asking anything.
- If the user provided no source and nothing to spec, ask first what they want to spec. Do not invent a capability or proceed on assumptions.
- Read `<workspace-root>/<project-name>-spex/spex.yaml` for the project overview. Repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role and use that to determine the spec's implementation `scope`.
- Run `spex specs --ai --output <tmp-path>`, then read `<tmp-path>` for related specs.
- Read the interface contracts in the `spex` repository's `contracts/` folder, listed per repository under `contracts` in `spex.yaml`, to learn the existing interfaces the spec relates to.
- Run `spex rules --id writing-style --ai` for the writing conventions the spec's prose must follow.

## Step 2 — Identify the capabilities and clarify missing details

Break the source into distinct capabilities — one spec per capability (see the scope rule above). When there is more than one, confirm the split with the user before drafting.

Pre-fill every field you can from the source and existing context. Then ask the user only about what's missing or ambiguous — unresolved details or acceptance criteria you cannot infer. Confirm anything you inferred rather than assuming silently.

## Step 3 — Draft the fields

Draft each spec following the field rules in `<workspace-root>/<project-name>-spex/schemas/spec.schema.json`, using its `examples` as the model for the YAML you write. Key points while drafting:

- `id` must convey the capability so it reads without opening the spec. It should be unique enough so that global search will find no other matches, prefer two or more words with dash as it's unlikelly to match anything.
- Write `acceptance_criteria` as verifiable statements, each beginning with an implementation checkbox followed by a commitment level — write `[ ]` for every newly drafted criterion, since none is implemented yet — ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY. These are the spec's specifics. Double-quote each criterion in YAML so the leading `[ ]` is read as text, not a flow sequence.
- Be specific. When the user has given a concrete decision — a constant, default, threshold, rate, or rule (e.g. each pizza takes one minute to make, two workers are on shift, the estimate is rounded to whole minutes) — capture it verbatim as its own verifiable criterion. Do not soften a value the user supplied into a vague directional statement, and do not invent a value the user did not supply: if a detail needed to make a criterion verifiable is missing, ask for it (see Step 2) instead of guessing.
- Describe end behavior, not implementation steps. A criterion must be verifiable from the outside — what the system does or exposes — not the internal steps taken to build it (installing a package, adding a dependency, creating a file, writing a migration).
- Set `scope` to the repositories that implement the capability, each matching a `repository_short_name` in `spex.yaml`.
- When `scope` lists more than one repository, make the division of responsibility explicit: every acceptance criterion must name the repository that owns it (after the commitment level, since each criterion must start with one), so the boundary between repositories is unambiguous and no shared behavior is left unassigned. Split a behavior that crosses the boundary into separate per-repository criteria rather than one vague criterion. For example, the API owns producing a value and the frontend owns presenting it — write "MUST compute the delivery-time estimate in the `api` from the order's pizza count, the in-progress orders, and the workers on shift" and "MUST display the estimate in the `frontend` in the cart and on the order confirmation" as distinct criteria.
- Set `blocked_by` only for specs that must be in place before this one can be implemented and their status is not `active`. Do not list a shared foundation that every capability rides on (e.g. the API surface), and never list an already-shipped or established spec. When nothing genuinely blocks the work, leave it absent.
- Keep `description` short, simple, and concrete: a general summary of the capability and who or what is affected. Do not restate specifics already covered by `acceptance_criteria`, and leave the step-by-step implementation to the plan.
- Add `examples` only when a request/response, payload, or usage snippet clarifies the capability better than prose. Key each entry with a short label (e.g. `request`, `response`).

## Step 4 — Write, then offer edits

For each drafted or edited spec:

1. Write it to `<workspace-root>/<project-name>-spex/specs/<spec-id>.yaml` — the filename must equal the `<spec-id>`. Use literal block scalars for multi-paragraph `description`. When editing an existing spec, change only what the user asked for and leave every other field, its ordering, and formatting untouched.
2. Validate the result against `<workspace-root>/<project-name>-spex/schemas/spec.schema.json`.
3. Tell the user the path you wrote (or the diff you applied, for an edit) and ask whether they want to change anything further. Apply any requested edits in place.
4. When `scope` lists more than one repository (for example `frontend` and `api`), name each in its acceptance criteria as shown in the example. For a single-scoped spec, omit the repository name from acceptance criteria — it's already redundant with `scope`.

## Example

Written to `specs/reset-password.yaml`:

```yaml
id: reset-password
title: Let users reset a forgotten password
description: >-
  Lets a user who forgot their password regain access by requesting a reset
  link by email and choosing a new password.
acceptance_criteria:
  - "[ ] MUST send a reset link to the account's email in the `api` when the user requests a reset with a registered email"
  - "[ ] MUST expire the reset link in the `api` 30 minutes after it is sent"
  - "[ ] MUST invalidate the reset link in the `api` once it has been used"
  - "[ ] MUST require the new password in the `api` to meet the account's standard password rules"
  - "[ ] MUST NOT reveal in the `api` whether an email is registered when a reset is requested for an unknown email"
  - "[ ] MUST let the user request a reset link in the `frontend` from the sign-in screen"
  - "[ ] MUST let the user choose a new password in the `frontend` from the reset link"
  - "[ ] SHOULD sign the user out of all other active sessions in the `api` once the password is reset"
scope:
  - api
  - frontend
status: planned
```

## Step 5 — Point to the next skill

After the specs are written, offer to draft each one's contract: run `spex-contract <spec-id>` with the spec's `id`. Note that it is better to start a new chat first to save context.

Follow the `writing-style` rule read in Step 1.
