---
name: spex-capability-spec
description: Create new capability specs (CS) or edit existing ones by interviewing the user. Use when the user wants to author, draft, extract, update, or amend CS specs — from a PRD, an RFD, a feature idea, a business rule, a technical capability the product needs, or a named existing spec. Asks targeted questions, then writes schema-valid YAML files to <workspace-root>/<project-name>-spex/cs-specs/.
---

# Create or Edit Capability Specs (CS)

A capability spec (`CS`) describes _a capability the system must provide_: one coherent product behavior, or a technical implementation needed to support the product's rules, with its individual rules captured as verifiable acceptance criteria. Most are extracted from the PRD; some capture supporting technical capabilities. A single source usually covers several capabilities, so it can produce one or more specs. Your job is to turn a fuzzy idea or PRD into schema-valid specs by asking the user focused questions, then writing one file per capability — or, when the user names an existing spec, to update it in place.

Match the prose style of the existing specs and write the result into the spex repository's `<workspace-root>/<project-name>-spex/cs-specs/` directory.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the spec's `id`.

The schema at `<workspace-root>/<project-name>-spex/schemas/cs-spec.schema.json` is authoritative for field rules and contains worked examples. Read it before drafting.

## When to use this skill

Use it when the user wants to create, draft, extract, update, or amend CS specs — from a PRD, an RFD, a feature request, a business rule, a technical capability the product needs (an endpoint, integration, health check, performance target), or a named existing spec. Do not use it for cross-cutting conventions or rules every change must follow; those are rule specs (RS). The test: a specific capability the system must provide → CS; a rule all code must obey, enforceable repo-wide → RS.

## Scope rule: one capability per spec

A CS spec captures exactly one coherent capability; its individual rules are the acceptance criteria. A PRD typically breaks down into many capability specs, so a single source can yield several. When the source covers more than one capability, identify each distinct one, propose the split as a list of specs (one `id` and one-line summary each), and confirm it with the user before drafting. Write each confirmed capability as its own spec file.

## Step 0 — Establish create vs. edit

First decide whether this is a new spec or a change to one that already exists:

- If the user names an existing spec ID (for example `CS-PLACE-PIZZA-ORDER`) or asks to update, amend, fix, or change a spec, this is an **edit**. Read `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml` in full before doing anything else — this is the file you will modify, not replace. Confirm with the user what should change (new or revised acceptance criteria, a scope change, a status change), applying the same field rules as Step 3. Skip straight to Step 4, editing the existing file in place rather than writing a new one; preserve every untouched field, its ordering, and formatting.
- Otherwise, this is a **new spec** — continue with Step 0's source-gathering below, then Steps 1–4.

### Establish the source (new specs only)

Determine what the specs are based on before doing anything else:

- If the user attached or pasted a source (a PRD, an RFD, or a feature description) — whether as text or as a file path/link — treat it as the primary input. Read the attached file or the pasted text in full and pre-fill every answer you can from it before asking anything.
- If the user provided no source and nothing to spec, ask first what they want to spec. Do not invent a capability or proceed on assumptions.

## Step 1 — Gather context first

Before asking the user anything, read what already exists so you don't ask for known answers or create duplicates:

- Read `<workspace-root>/<project-name>-spex/spex.yaml` for the project overview. Repository names are arbitrary — only `spex` is fixed — so read each repository's `description` to learn its role and use that to determine the spec's implementation `scope`.
- Read `<workspace-root>/<project-name>-spex/cs-specs/` for related specs
- Read the interface contracts in the `spex` repository's `contracts/` folder, listed per repository under `contracts` in `spex.yaml`, to learn the existing interfaces the spec relates to.

## Step 2 — Identify the capabilities and clarify missing details

Break the source into distinct capabilities — one spec per capability (see the scope rule above). When there is more than one, confirm the split with the user before drafting.

Pre-fill every field you can from the source and existing context. Then ask the user only about what's missing or ambiguous — unresolved details or acceptance criteria you cannot infer. Confirm anything you inferred rather than assuming silently.

## Step 3 — Draft the fields

Draft each spec following the field rules in `<workspace-root>/<project-name>-spex/schemas/cs-spec.schema.json`, using its `examples` as the model for the YAML you write. Key points while drafting:

- `id` must start with `CS-` and convey the capability so it reads without opening the spec.
- Write `acceptance_criteria` as verifiable statements, each beginning with an implementation checkbox followed by a commitment level — write `[ ]` for every newly drafted criterion, since none is implemented yet — ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY. These are the spec's individual rules. Double-quote each criterion in YAML so the leading `[ ]` is read as text, not a flow sequence.
- Be specific. When the user has given a concrete decision — a constant, default, threshold, rate, or rule (e.g. each pizza takes one minute to make, two workers are on shift, the estimate is rounded to whole minutes) — capture it verbatim as its own verifiable criterion. Do not soften a value the user supplied into a vague directional statement, and do not invent a value the user did not supply: if a detail needed to make a criterion verifiable is missing, ask for it (see Step 2) instead of guessing.
- Set `scope` to the repositories that implement the capability, each matching a `repository_short_name` in `spex.yaml`.
- When `scope` lists more than one repository, make the division of responsibility explicit: every acceptance criterion must name the repository that owns it (after the commitment level, since each criterion must start with one), so the boundary between repositories is unambiguous and no shared behavior is left unassigned. Split a behavior that crosses the boundary into separate per-repository criteria rather than one vague criterion. For example, the API owns producing a value and the frontend owns presenting it — write "MUST compute the delivery-time estimate in the `api` from the order's pizza count, the in-progress orders, and the workers on shift" and "MUST display the estimate in the `frontend` in the cart and on the order confirmation" as distinct criteria.
- Set `blocked_by` only for specs (CS or RS) that must be in place before this one can be implemented and are not yet satisfied. Do not list a shared foundation that every capability rides on (e.g. the API surface), and never list an already-shipped or established spec. When nothing genuinely blocks the work, leave it absent.
- Keep `description` short, simple, and concrete: a general summary of the capability and who or what is affected. Do not restate individual rules already covered by `acceptance_criteria`, and leave the step-by-step implementation to the plan.

## Step 4 — Write, then offer edits

For each drafted or edited spec:

1. Write it to `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml` — the filename must equal the `<spec-id>`. Use literal block scalars for multi-paragraph `description`. When editing an existing spec, change only what the user asked for and leave every other field, its ordering, and formatting untouched.
2. Validate the result against `<workspace-root>/<project-name>-spex/schemas/cs-spec.schema.json`.
3. Tell the user the path you wrote (or the diff you applied, for an edit) and ask whether they want to change anything further. Apply any requested edits in place.

## Step 5 — Point to the next skill

After the specs are written, offer to draft each one's contract: run `spex-contract <spec-id>` with the spec's `id`. Note that it is better to start a new chat first to save context.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
