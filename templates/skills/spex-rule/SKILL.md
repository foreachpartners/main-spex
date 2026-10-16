---
name: spex-rule
description: Create a new rule or edit an existing one by interviewing the user. Use when the user wants to create, extend, or update a rule — from a coding standard, a style guide, a security policy, an engineering decision, an existing code pattern, or a named existing spec.
---

# Create or Edit a Rule

A rule describes _how the code and artifacts are built_: one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules captured as acceptance criteria. It is not tied to one feature — it applies across whole packages. Your job is to turn a fuzzy convention into one schema-valid spec by asking the user focused questions, proposing automated checks that verify it, then writing the file — or, when the user names an existing spec, to update it in place.

## Paths

Replace `<spex-dir>` with the spex directory, printed by `spex info | spex jq -r .spex_dir`, and `<rule-id>` with the rule's `id`.

The schema at `<spex-dir>/schemas/rule.schema.json` is authoritative for field rules and contains worked examples. Read it before drafting.

Writes a schema-valid YAML file to `<spex-dir>/rules/<rule-id>.yaml`

## Spec vs Rule

A cross-cutting rule all code must obey, enforceable package-wide -> rule. A specific capability the system must provide (an endpoint, integration, health check, performance target) -> spec.

## Scope rule: one convention per spec

A rule captures exactly one coherent cross-cutting convention; its individual rules are the acceptance criteria. If the user describes several unrelated conventions, propose splitting them into multiple specs and confirm the split before writing.

### Small conventions: use general-rules instead of a new file

Not every convention earns its own file. If the requested convention is small — a single terse acceptance criterion (for example "no console.log", "always run gofmt before commit") — and doesn't share a theme with any existing rule, don't create a dedicated spec for it. Instead propose adding it as one more acceptance criterion (with its check) to the catch-all bucket for the packages it applies to:

- `rules/general-rules.yaml` (`id: general-rules`) when it applies project-wide or to every package the user names.
- `rules/general-rules-<package-short-name>.yaml` (`id: general-rules-<package-short-name>`) when it applies to one specific package only.

If the matching bucket file already exists, treat this as an edit to it (see Step 0): read it in full and append the new acceptance criterion and check, preserving everything else untouched. If it doesn't exist yet, create it fresh with this single criterion — it will grow as more small, unrelated conventions come in.

A convention still gets its own dedicated file when it has multiple related acceptance criteria, a clear theme of its own, or the user asks for a separate file. Confirm which path you're taking with the user before writing.

## Step 0 — Establish create vs. edit

First decide whether this is a new spec or a change to one that already exists:

- If the user names an existing rule ID (for example `secret-handling`) or asks to update, amend, fix, or change a spec, this is an **edit**. Read `<spex-dir>/rules/<rule-id>.yaml` in full before doing anything else — this is the file you will modify, not replace. Also read its checks and any `script` bodies at `<spex-dir>/rules/<rule-id>/`. Confirm with the user what should change (revised or new acceptance criteria, new or adjusted checks, a scope change), applying the same field rules as Steps 3–4. Skip straight to Step 5, editing the existing file (and any check scripts) in place rather than starting over; preserve every untouched field, its ordering, and formatting.
- Otherwise, this is a **new spec** — continue with Step 0's source-gathering below, then Steps 1–4.

### Establish the source (new specs only)

Determine what the spec is based on before doing anything else:

- If the user attached or pasted a source (a style guide, a security policy, an engineering decision, or a code excerpt) — whether as text or as a file path/link — treat it as the primary input. Read the attached file or the pasted text in full and pre-fill every answer you can from it before asking anything.
- If the user provided no source and nothing to spec, ask first what convention they want to spec. Do not invent a convention or proceed on assumptions.

## Step 1 — Gather context first

Before asking the user anything, read what already exists so you don't ask for known answers or create duplicates:

- Read `<spex-dir>/spex.yaml` for the project overview, the packages, and which rules they already list. Package names are arbitrary, so read each package's `description` to learn its role.
- Read `<spex-dir>/rules/` for related specs and to match their style. Reuse an existing spec rather than duplicating its convention. Note whether `general-rules.yaml` or a per-package `general-rules-<package-short-name>.yaml` already exists — see "Small conventions" below.

## Step 2 — Clarify missing details with the user

Pre-fill every field you can from the source and existing context. Then ask the user only about what's missing or ambiguous — the rationale (what the convention protects against), the acceptance criteria, and which packages it applies to. Confirm anything you inferred rather than assuming silently.

Do not set `sync_with_upstream` on a consumer spec unless the user explicitly asks; it governs global framework specs. Ask which packages the convention applies to, then list the spec's `id` under each of those packages' `rules` in `spex.yaml`.

## Step 3 — Draft the fields

Follow the field rules in `<spex-dir>/schemas/rule.schema.json` and use its `examples` as the model for the YAML you write. Key points while drafting:

- `id` must convey the convention so it reads without opening the spec. It should be unique enough so that global search will find no other matches, prefer two or more words with dash as it's unlikely to match anything.
- Keep `description` short, simple, and concrete: a general summary of the rule and what it protects against. Do not restate individual rules already covered by `acceptance_criteria`.
- Write `acceptance_criteria` as verifiable statements, each beginning with a commitment level, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY. A rule is a global rule that always applies, so its criteria carry no implementation checkbox.
- Keep each criterion provable by an automated check or by AI inspection.

## Step 4 — Propose checks

For each acceptance criterion, propose how it can be verified automatically, then confirm the set with the user before writing. Pick the simplest check `type` that proves the criterion:

- `regexp` — match file contents. Use `negate: false` (default) when a match is a violation (forbidden pattern); use `negate: true` when a file with no match is a violation (required pattern). Narrow with `include`/`exclude` globs.
- `glob` — match file paths. Use it to forbid or require the presence of files (for example `**/*.bak`).
- `ast-grep` — match a code structure by syntax tree, ignoring comments/strings, tolerating reformatting, and expressing relational constraints (a node inside or not inside another, argument text matching a sub-pattern) that `regexp` cannot. Give it a structural `pattern` (e.g. `console.log($$$ARGS)`; `$VAR` binds one node, `$$$VAR` a list) or a full ast-grep `rule` object (`pattern`, `kind`, `regex`, `inside`, `has`, `not`, `all`, `any`, with metavariable `constraints`), mutually exclusive. Set `language` (e.g. `typescript`, `rust`) or omit it to infer per file from the extension, skipping files with no matching language. Use `negate: false` (default) when a match is a violation, `negate: true` when a file with no match is a violation.
- `script` — run an external script named by `script:` (a bare filename, resolved in the rule's script folder `rules/<rule-id>/<script>`, beside the rule file). Use it when a rule needs logic a regex can't express. Set `run: per-file` (default) to invoke it with the matched files as arguments. Set `run: per-package` to invoke it once per package, in that package's directory, with no arguments. Set `run: project` to invoke it exactly once, with no arguments.

Guidance:

- Name each check in kebab-case, unique within the spec. A `script` check names its program with `script:` (just the filename), resolved in the rule's script folder `rules/<rule-id>/` so checks from different rules never share a path.
- Set `severity: warn` for advisory checks and `error` (default) for enforced ones.
- Not every criterion needs a check. When a criterion can only be judged by AI or human review, say so and leave it without a check rather than inventing a brittle one.
- When you propose a `script` check, also draft the file it references: a POSIX `sh` script that exits non-zero on violation. Store it at `rules/<rule-id>/<check-name>.sh` and mirror the header style of existing scripts (`# <rule-id> / <name>`). A `per-file` script receives the candidate files as arguments; a `per-package` script receives none.
- A failing script MUST emit one finding per line on stdout following the script location contract: `<location><TAB><message>`, where `<location>` is a package-relative file (optionally `<file>:<line>`) and `<message>` describes the violation. This gives script errors the same per-file precision as `regexp` and `glob` checks, so an `ignored_errors` baseline can suppress them per file. Emit a file-less finding (no location, leaving the part before the tab empty, or printing a line with no tab) only when the violation has no natural path, for example a rule about git history. Do not print a multi-line dump for one violation — emit one tab-separated line per location:
  ```sh
  printf '%s:%s\t%s\n' "$file" "$lineno" "matches forbidden pattern"
  ```
- A script receives the absolute path of the spex directory in the `SPEX_DIR` environment variable. When it needs data from `spex.yaml` (the project name, the package list), read it with `spex jq` rather than parsing `spex.yaml` by hand — hand-parsing is brittle and duplicated across scripts, and `jq` itself may not be installed. `spex jq` runs a jq filter on JSON or YAML (e.g. `spex jq -r .project_name "$SPEX_DIR/spex.yaml"`); pipe `spex info` into it for the fully-resolved configuration, with absolute package paths (e.g. `spex info | spex jq -r '.packages[].path'`).

Draft the checks (and any script bodies) into the spec; you write them and offer edits in Step 5.

## Step 5 — Write, then offer edits

1. Write the spec to `<spex-dir>/rules/<rule-id>.yaml` — the filename must equal the `<rule-id>` (`general-rules.yaml` or `general-rules-<package-short-name>.yaml` when folding into the catch-all bucket). Use a literal block scalar (`>-`) for a multi-paragraph `description`. When editing an existing spec — including appending to an existing general-rules bucket — change only what the user asked for and leave every other field, its ordering, and formatting untouched.
2. Write any `script` check bodies to the path referenced by `script:` (`<spex-dir>/rules/<rule-id>/<name>.sh`) and make them executable.
3. For a new spec (including a newly created general-rules bucket), add its `id` under `rules` in `spex.yaml` for each package the user said it applies to.
4. Run `spex check` to validate compliance.
5. Tell the user the spec and checks you wrote (or the diff you applied, for an edit) and where, then ask whether they want to change anything further. Apply any requested edits in place.

## Example

A themed, multi-criterion convention earns its own file — written to `rules/writing-style.yaml`:

```yaml
id: writing-style
title: Writing style
description: >-
  Specs, code, and docs are read by engineers and AI agents alike. Direct,
  imperative prose with explicit commitment levels keeps rules unambiguous.
acceptance_criteria:
  - "MUST use imperative, directive language: 'Load config at startup.' not 'We'll load config...'"
  - "MUST use RFC 2119 commitment levels — MUST, SHOULD, MAY, MUST NOT, SHOULD NOT"
  - "MUST NOT use motivational or hedging phrases ('Great job!', 'maybe', 'I think')"
checks:
  - name: no-hedging-language
    description: Flag hedging and motivational phrasing
    type: regexp
    pattern: "(?i)\\b(maybe|perhaps|great job)\\b"
    severity: warn
```

A small, single-criterion convention with no theme of its own joins the shared bucket instead — appended as one more entry in `rules/general-rules.yaml`:

```yaml
id: general-rules
title: General rules
description: >-
  Miscellaneous cross-cutting conventions too small to warrant their own spec.
acceptance_criteria:
  - "MUST NOT use `console.log` in application code — use the project logger"
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

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
