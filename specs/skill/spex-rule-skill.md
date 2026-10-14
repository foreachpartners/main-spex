# `spex-rule` skill

Creates a new rule or edits an existing one by interviewing the user. Loaded when the user wants to create, extend, or update a rule, from a coding standard, a style guide, a security policy, an engineering decision, an existing code pattern, or a named existing rule. It turns a fuzzy convention into one schema-valid rule by asking the user focused questions, proposing automated checks that verify it, then writing the file, or updates the named existing rule in place.

## Acceptance criteria

### Paths and scope

- MUST read `<spex-dir>/schemas/rule.schema.json`, the authority for field rules with worked examples, before drafting.
- MUST write a schema-valid YAML file to `<spex-dir>/rules/<rule-id>.yaml`.
- MUST treat a cross-cutting rule all code must obey, enforceable package-wide, as a rule, and a specific capability the system must provide (an endpoint, integration, health check, performance target) as a spec.
- MUST capture exactly one coherent cross-cutting convention per rule, its individual rules being the acceptance criteria.
- MUST propose, when the user describes several unrelated conventions, splitting them into multiple rules, and confirm the split before writing.

### Small conventions

- MUST propose adding a small convention, a single terse acceptance criterion (e.g. "no console.log", "always run gofmt before commit") that does not share a theme with any existing rule, as one more acceptance criterion (with its check) to a catch-all bucket for the packages it applies to, instead of creating a dedicated rule file.
- MUST use `rules/general-rules.yaml` (`id: general-rules`) as the bucket when the convention applies project-wide or to every package the user names, and `rules/general-rules-<package-short-name>.yaml` (`id: general-rules-<package-short-name>`) when it applies to one specific package only.
- MUST treat adding to an existing bucket file as an edit: read it in full and append the new acceptance criterion and check, preserving everything else untouched.
- MUST create the bucket fresh with the single criterion when it does not exist yet.
- MUST still give a convention its own dedicated file when it has multiple related acceptance criteria, a clear theme of its own, or the user asks for a separate file.
- MUST confirm with the user which path it is taking before writing.

### Step 0: establish create vs edit

- MUST treat the request as an edit when the user names an existing rule ID (e.g. `secret-handling`) or asks to update, amend, fix, or change a rule.
- MUST read `<spex-dir>/rules/<rule-id>.yaml` in full before doing anything else, for an edit, together with its checks and any `script` bodies at `<spex-dir>/rules/<rule-id>/`, confirm with the user what should change (revised or new acceptance criteria, new or adjusted checks, a scope change), and skip straight to Step 5, editing the existing file (and any check scripts) in place and preserving every untouched field, its ordering, and formatting.
- MUST treat the request otherwise as a new rule: gather the source below, then follow Steps 1-4.
- MUST treat a source the user attached or pasted (a style guide, a security policy, an engineering decision, or a code excerpt), as text or as a file path or link, as the primary input for a new rule: read it in full and pre-fill every answer it supports before asking anything.
- MUST ask first what convention the user wants to specify when they provided no source and nothing to specify.
- MUST NOT invent a convention or proceed on assumptions.

### Step 1: gather context

- MUST read, before asking the user anything, `<spex-dir>/spex.yaml` for the project overview, the packages, and which rules they already list, and each package's `description` to learn its role.
- MUST read `<spex-dir>/rules/` for related rules and to match their style, reusing an existing rule rather than duplicating its convention.
- MUST note whether `general-rules.yaml` or a package-scoped `general-rules-<package-short-name>.yaml` already exists.

### Step 2: clarify missing details with the user

- MUST pre-fill every field it can from the source and existing context, then ask the user only about what is missing or ambiguous: the rationale (what the convention protects against), the acceptance criteria, and which packages it applies to.
- MUST confirm anything it inferred rather than assuming silently.
- MUST ask which packages the convention applies to, then list the rule's `id` under each of those packages' `rules` in `spex.yaml`.
- MUST NOT set `sync_with_upstream` on a consumer rule unless the user explicitly asks; it governs global framework rules.

### Step 3: draft the fields

- MUST follow the field rules in `<spex-dir>/schemas/rule.schema.json` and use its `examples` as the model for the YAML.
- MUST give the `id` a name that conveys the convention so it reads without opening the rule, unique enough that a global search finds no other matches, preferring two or more words with a dash.
- MUST keep `description` short, simple, and concrete: a general summary of the rule and what it protects against.
- MUST write `acceptance_criteria` as verifiable statements, each beginning with a commitment level, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY, with no implementation checkbox, since a rule is global and always applies.
- MUST keep each criterion provable by an automated check or by AI inspection.
- MUST NOT restate in `description` the individual rules already covered by `acceptance_criteria`.

### Step 4: propose checks

- MUST propose, for each acceptance criterion, how it can be verified automatically, and confirm the set with the user before writing.
- MUST pick the simplest check `type` that proves the criterion: `regexp` (match file contents), `glob` (match file paths), or `script` (an external script, when a rule needs logic a regex cannot express).
- MUST use `negate: false` (default) on a `regexp` check when a match is a violation (a forbidden pattern), and `negate: true` when a file with no match is a violation (a required pattern), narrowing with `include`/`exclude` globs.
- MUST use a `glob` check to forbid or require the presence of files (e.g. `**/*.bak`).
- MUST name a `script` check's program with `script:`, a bare filename resolved in the rule's script folder `rules/<rule-id>/<script>` beside the rule file, so checks from different rules never share a path, and set `run: per-file` (default; matched files as arguments), `run: per-package` (once per package, in that package's directory, no arguments), or `run: project` (exactly once, no arguments).
- MUST name each check in kebab-case, unique within the rule.
- MUST set `severity: warn` for advisory checks and `error` (default) for enforced ones.
- MUST leave a criterion that can only be judged by AI or human review without a check, and say so, rather than invent a brittle one.
- MUST draft, for a `script` check, the file it references: a POSIX `sh` script that exits non-zero on violation, stored at `rules/<rule-id>/<check-name>.sh`, mirroring the header style of existing scripts (`# <rule-id> / <name>`).
- MUST have a failing script emit one finding per line on stdout as `<location><TAB><message>`, where `<location>` is a package-relative file (optionally `<file>:<line>`) and `<message>` describes the violation, e.g. `printf '%s:%s\t%s\n' "$file" "$lineno" "matches forbidden pattern"`, so script errors get the same per-file precision as `regexp` and `glob` checks and an `ignored_errors` baseline can suppress them per file.
- MUST emit a file-less finding (no location, leaving the part before the tab empty, or a line with no tab) only when the violation has no natural path, for example a rule about git history.
- MUST read data a script needs from `spex.yaml` (the project name, the package list) with `spex jq` rather than parsing `spex.yaml` by hand, since hand-parsing is brittle and duplicated across scripts and `jq` itself may not be installed; the script receives the absolute path of the spex directory in `SPEX_DIR`, and `spex info` piped into `spex jq` gives the fully-resolved configuration with absolute package paths.
- MUST draft the checks, and any script bodies, into the rule, and write them and offer edits in Step 5.
- MUST NOT print a multi-line dump for one violation: one tab-separated line per location.

### Step 5: write, then offer edits

- MUST write the rule to `<spex-dir>/rules/<rule-id>.yaml`, the filename equal to the `<rule-id>` (`general-rules.yaml` or `general-rules-<package-short-name>.yaml` when folding into the catch-all bucket), using a literal block scalar (`>-`) for a multi-paragraph `description`.
- MUST change, when editing an existing rule, including appending to an existing general-rules bucket, only what the user asked for, leaving every other field, its ordering, and formatting untouched.
- MUST write any `script` check bodies to the path referenced by `script:` (`<spex-dir>/rules/<rule-id>/<name>.sh`) and make them executable.
- MUST add, for a new rule (including a newly created general-rules bucket), its `id` under `rules` in `spex.yaml` for each package the user said it applies to.
- MUST run `spex check` to validate compliance.
- MUST tell the user the rule and checks it wrote (or the diff it applied, for an edit) and where, ask whether they want to change anything further, and apply any requested edits in place.

### Step 6: point to the next skill

- MUST do nothing when there are no `spex check` errors related to the new rule.
- MUST offer to fix the errors when they can be fixed easily.
- MUST offer to plan the implementation with `spex-plan <rule-id>` when there are too many errors, noting that starting a new chat first saves context.
- MUST follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else, with no filler, hedging, motivational, or apologetic phrasing.
