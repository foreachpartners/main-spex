# Specification

How the spex framework behaves, written as specs: each section states one behavior with its acceptance criteria. [glossary.md](glossary.md) defines the terms, [cli.md](cli.md) the command usage, and the bundled `schemas/` the field rules.

# Framework

## Artifact chain

Turn business goals into verified code through a chain of artifacts, each traceable by a shared Spec ID:

`BRD` (why, optional) -> `PRD` (what, optional) -> spec (capability) -> contract (interface, if applicable) -> implementation (code) -> test (proof)

### BRD

Business goals and stakeholder needs, the root source when present. The product owner identifies, prioritizes, and translates them into a PRD.

### PRD

Features and user-facing behavior extracted from the BRD by the product owner as user stories and requirements for the development team. Breaks down into many specs.

### Spec

One capability the system must provide, with its specifics captured as acceptance criteria. Extracted from the PRD — or from the BRD or a feature idea directly when there is no PRD — by the product owner and/or developer. The single source of truth for product behavior: implemented, then extended or changed over time.

### Contract

The interface derived from a spec when applicable: OpenAPI, proto, GraphQL SDL, or AsyncAPI for a machine-readable interface (API, cross-service communication); a design reference — a markdown document linking the spec to its visual design (e.g. a Figma frame) — for a UI-facing spec with no machine-readable interface. Small next to the implementation, so it is cheap to create, review, and reason about first.

### Plan

An ordered set of steps turning approved specs and their contracts into implementation instructions an agent executes, with a status on every step and task so progress is trackable.

- Optional for small, self-contained changes.
- Focused on interfaces, not implementation: a plan nobody can review delivers nothing.
- A working document, not a permanent artifact: a step splits a large change into reviewable pieces, and a status line lets work pause and resume across sessions.
- `spex-implement` deletes a plan file once every step is `completed`. A finished plan says nothing the spec, code, and tests don't already say more accurately, and left behind it only becomes a stale document that search keeps surfacing.

## Specs

A spec captures one coherent capability the system must provide — a product behavior, or a technical implementation needed to support the product's rules — with its specifics in the acceptance criteria. Lives in `specs/`.

- The test: a specific capability the system provides -> spec.
- `specs/` may group files in nested subdirectories (e.g. `specs/frontend/checkout-flow.yaml`); the Spec ID is always just the filename, whatever the nesting.

## Rules

A rule captures one coherent cross-cutting convention or rule — style, structure, security, traceability, documentation — with its individual rules in the acceptance criteria. Lives in `rules/`.

- The test: a convention every change must follow, enforceable repo-wide -> rule.
- `rules/` may group files in nested subdirectories (e.g. `rules/frontend/frontend-testing.yaml`); the Rule ID is always just the filename, whatever the nesting.

# Workspace

## Layout

A project spans several workspace repositories under one workspace root folder.

- Each workspace repository is its own git repository, its directory named `<project-name>-<repository-short-name>` (e.g. `pepperoni-pizza-spex`, `pepperoni-pizza-api`, `pepperoni-pizza-frontend`).
- The spex repository (`<project-name>-spex`) holds the specs, rules, schemas, scripts, skills, config, and contracts — the single source of truth.
- spex locates the spex repository by finding the `spex.yaml` it contains, so every command except `init` works from the workspace root or any workspace repository, whatever the repository directory names are.
- Every fact has one authoritative source and is referenced, not copied: an interface only in its contract, a capability only in its spec, a convention only in its rule.
- Specs, contracts, and checks are plain files and deterministic commands, so no step of the workflow depends on a specific editor, terminal, or AI model.
- Each repository's contracts are declared per repository in `spex.yaml`; the contract file's location is user-owned — most commonly inside the owning repository itself — with the spex repository's `contracts/` folder reserved for contracts genuinely shared across repositories.
- An editor workspace file (`<project-name>.code-workspace`) opens every repository together.

## Traceability

Every behavioral change traces to a spec, and one Spec ID traces across the repositories that satisfy it.

- The Spec ID is the unit of traceability: every behavioral change traces to one, with no orphans.
- Traceability is bidirectional: top-down (spec -> contract -> code -> test) and bottom-up (test -> code -> contract -> spec).
- Separate repositories give per-repository access control (least privilege).
- One Spec ID traces across many repositories.
- Checking out a commit shows exactly which specs the code was built against.

# CLI

## init

Creates the spex repository (`<project-name>-spex`) in the workspace root and scaffolds the framework.

- Prompts to confirm the project name, defaulting to the workspace root folder name.
- Creates the spex repository directory named `<project-name>-spex`; run from an existing spex repository directory, it reuses that directory instead.
- Prompts to confirm the discovered workspace repositories, deriving each `repository_short_name` from its directory name minus the `<project-name>-` prefix.
- Writes `spex.yaml`, recording the installed CLI version as `cli_version`, and the editor workspace file with a folder per repository.
- Scaffolds the bundled templates: `schemas/`, `docs/`, `scripts/`, `rules/`, and skills; creates empty `specs/`, `plans/`, and `contracts/` folders.
- Installs skills under `.agents/skills/` and symlinks `.claude` -> `.agents`, so different agent tools detect the same skills.
- Seeds a stub `SKILL.md` for every workspace repository, named after the repository, holding only a frontmatter `description` and no body. The description states the skill must be loaded before reading, writing, testing, or running any code in that repository, so an agent working across the workspace autoloads it from context instead of waiting for an explicit `/name` invocation. This gives every repository a stable place for its own commands and context; `spex-setup` fills in the body.
- Writes a root `AGENTS.md` and symlinks `CLAUDE.md` -> `AGENTS.md`, seeding them only when absent. A `# Project` marker splits `AGENTS.md`: content above it is managed and kept in sync by `spex update`, content from the marker onward is the user's own and is never touched.
- Copies every bundled rule into the spex repository's `rules/`, but leaves every repository's `rules` list in `spex.yaml` empty.
- Directs the user to finish setup by running the `/spex-setup` skill.

## update

Rewrites the managed framework files in the spex repository with the versions shipped in the installed `spex` binary.

- Rewrites the bundled schemas and docs.
- Rewrites a bundled skill already present locally only when its `metadata.sync-with-upstream` is not explicitly `false`; a skill that opts out keeps its local changes, so consumers can override a global skill.
- Rewrites a bundled rule (and its scripts) already present locally only when it has `sync_with_upstream: true`; a local rule without it is left untouched, so consumers can override a global rule.
- Deploys a bundled rule missing locally, unless its id is listed in `rules_unused` in `spex.yaml`; listing an id there records a deliberate deletion, so it stays deleted on future updates.
- Rewrites the managed section of the root `AGENTS.md` (everything above the `# Project` marker), preserving everything from that marker onward; a local `AGENTS.md` predating the marker is fully overwritten, since managed and user content cannot be told apart in it.
- Overwrites managed files only; leaves plans and any files the user added untouched.
- Updates `cli_version` in `spex.yaml`.
- Prints the `CHANGELOG.md` entries newer than the previous `cli_version`, newest first; prints nothing if there are none, and every entry if the previous `cli_version` is not found in the changelog.

## check

Runs the rule checks declared in `spex.yaml` and gates on errors.

- Opens its report with the installed CLI's version (e.g. `spex 0.10.0`, matching `spex --version`), followed by a blank line, so the output identifies the binary that produced it.
- Runs the rule checks each repository lists in its `rules`, reporting findings per repository.
- Skips files ignored by `.gitignore`, even in a repository without a `.git` directory, plus well-known dependency lock files (e.g. `Cargo.lock`, `package-lock.json`).
- Renders every finding with a location as a terminal hyperlink to the absolute file it points at, so it opens correctly whatever the shell's current working directory is; the displayed text stays the short repository-relative path.
- Exits non-zero when any `error`-severity finding is reported; `warn` findings do not affect the exit status.

### Self-validation

When the spex repository is in scope, `spex check` validates it directly, without a rule:

- `spex.yaml`, every `specs/<spec-id>.yaml`, and every `rules/<rule-id>.yaml` must satisfy its JSON Schema.
- Each spec file must be named `<spec-id>.yaml` and each rule file `<rule-id>.yaml`.
- Declared properties must follow their JSON Schema's declared order, at every nesting level the schema describes (e.g. a rule's `checks[]` items, or `spex.yaml`'s `repositories[].*` and `repositories[].contracts[].*`).
- Acceptance criteria must be ordered by RFC 2119 commitment level (`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`), in specs and rules alike, and each criterion's `id` must be unique within its spec.
- No id repeats across `specs/` and `rules/`: Spec IDs and Rule IDs share one namespace.
- Every script a rule check references must exist at `scripts/<rule-id>/<script>`.
- A spec whose acceptance criteria are all done (`is_done` `true` or absent) but whose `status` is neither `active` nor `deprecated` warns; a spec with no acceptance criteria does not warn.
- A spec whose `status` is `active` with any acceptance criterion still `is_done: false` errors, since `active` means fully implemented.
- A `blocked_by` entry must name an existing spec and must be dropped once satisfied: referencing an already-`active` spec, or any rule, warns, since rules always apply.
- A `scope` entry must match a repository's `repository_short_name` in `spex.yaml`.
- A `coverage_ignore` entry must match the spec's `contract` setting: a contract-layer entry errors under `contract: none` (there is no contract coverage to waive) and, when it carries an `ac`, under `contract: single` (contracts are spec-level there, so per-criterion contracts only exist under `contract: for-each-ac`).

This reads only each criterion's `is_done` field — whether that is actually backed by impl and test annotations is `spex coverage`'s job.

### Annotation checks

`spex check` also validates Spec ID annotations across every repository, built in rather than declared as a rule:

- A line that opens an annotation with `@spec` but matches no full annotation errors.
- An annotation referencing a Spec ID with no matching spec or rule — an orphan — errors.
- A `#<ac-id>` suffix naming no acceptance criterion of its spec errors.
- Markdown files are skipped, as are the spex repository's own `schemas/`, `specs/`, `rules/`, and `scripts/` directories, which hold illustrative example annotations rather than real references.

### Baselines

A repository baselines pre-existing `error`-severity findings with `ignored_errors`, so a check can be enforced going forward without first fixing every existing violation. Each entry names a check by `rule_id` + `check`; an `error` finding from it is dropped when its location file is listed in `files`, or always when `files` is omitted. New errors in other files still surface, and `warn` findings are never affected.

- `--fill-preexisting-errors` records the current `error`-severity findings as an `ignored_errors` baseline in `spex.yaml`, then exits `0` without gating. Use it to adopt a check on a repository that already has violations.
- It runs every selected repository's checks, groups the resulting `error` findings by check and file, and writes one `ignored_errors` entry per check into that repository's block: a finding tied to a file is baselined for that file, and a file-less error (e.g. a `per-repo` script inspecting git history) widens to a whole-check entry that omits `files`.
- Each selected repository's baseline is regenerated from the current findings — dropping entries whose errors are gone — while the rest of `spex.yaml` and any unselected repository stay byte-stable.
- Repository-level errors that belong to no named check (e.g. a misconfigured override) are not baselined; fix those directly.

### Flags

- `--repo <name>...` — limit the run to the named workspace repositories, matched by `repository_name` or `repository_short_name`; defaults to all repositories in `spex.yaml`, and errors on a name matching no repository.
- `--files <path>...` — limit the run to the given files, relative to each repository; defaults to all non-ignored files, and errors when a given file exists in no workspace repository.
  - A `per-repo` script then runs only in the repositories that contain a listed file, unioned with any `--repo` named, while a `workspace` script always runs once.
  - Negated `regexp`, `glob`, and `ast-grep` checks are skipped, because a negated check needs the full file set to prove a forbidden pattern is absent; a subset would only produce false positives.

## check-schema

Validates JSON or YAML files against a JSON Schema.

- Resolves cross-file `$ref`s against sibling schema files in the schema's own directory — fully offline, no network access.
- Prints violations to stderr as `<file>: <message>`; exits non-zero when any file fails.
- `spex check` uses the same validation, built in, to keep the spex repository's own files valid against the bundled schemas.
- `--schema <path>` — the JSON Schema to validate against (required).
- `--files <path>...` — the JSON or YAML files to validate (required).

## coverage

Reports how well the active and in-progress specs are covered by `@spec` annotations across the workspace, per acceptance criterion.

- Opens its report with the installed CLI's version, followed by a blank line, so the output identifies the binary that produced it.
- Tracks every spec whose `status` is `active` or `in-progress`; a spec with no `status` defaults to `planned` and is not tracked, though its ID is still listed under its status.
- Requires, per tracked spec:
  - **Contract** — per the spec's `contract` field: `single` requires one spec-level `@spec contract <spec-id>` anywhere in a repository's file text, including as the string value of an OpenAPI vendor extension such as `x-spec`; `for-each-ac` requires a `@spec contract <spec-id>#<ac-id>` annotation per acceptance criterion; `none` requires none.
  - **Impl and test** — `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>` for every acceptance criterion.
- A bare, unqualified `@spec impl|test <spec-id>` never satisfies a criterion's requirement, since a single bare annotation would otherwise trivially satisfy every criterion in the spec.
- Searches every workspace repository, skipping the spex repository's own files except the contract files it declares, and skips files ignored by `.gitignore`. `--scope` narrows only which specs are reported, never where annotations are searched for.
- `coverage_ignore` relaxes the requirements per layer and acceptance criterion (see the `Spec` field list below).
- Shows each tracked spec's progress as `done/total AC`, counting a criterion once its required impl annotation is present, whether or not its required test annotation is; a missing test annotation shows up as a gap or as pending instead.
- **Gap** — a tracked spec missing a required contract annotation (spec-level under `contract: single`, per criterion under `contract: for-each-ac`), or an acceptance criterion missing a required impl or test annotation while its `is_done` is `true` or absent.
- **Pending** — an acceptance criterion missing a required annotation while flagged `is_done: false`, expected on an in-progress spec still being worked on, so it is not a gap.
- Lists specs grouped by status (omitting any status with no specs). The **Gaps** section lists only tracked specs that have a gap (`✗`), grouped by `scope` sorted alphabetically, with unscoped specs last under `unscoped`; a spec listed under several scopes appears under each, and the header count reflects each gapped spec once. Each entry shows its progress and what is missing, naming the acceptance criteria involved (e.g. `missing: contract, impl(no-null-typed-string), test(no-null-typed-string, empty-string-for-unset)`; under `contract: for-each-ac` the missing contract is named per criterion, e.g. `contract(ac-one, ac-two)`). A gapped spec with pending criteria reports them on a separate line labeled `in-progress`.
- A pending-only `in-progress` spec (`●`) is listed under its `in-progress` status group instead, not under **Gaps**, and is not grouped by `scope`.
- Ends with a one-line summary of spec counts per status and the gap count (e.g. `planned (5), in-progress (3), active (2), gaps (1)`).
- Exits non-zero only when an `active` spec has a gap; gaps on `in-progress` specs are reported but do not affect the exit code.
- `--scope <name>...` — limit the report to specs whose `scope` includes the given `repository_short_name`; defaults to all specs, and errors on an unrecognized name.
- `--verbose` (`-v`) — add description detail: each listed `planned` spec shows its `description`, and each gapped spec shows its `description` plus, for every acceptance criterion still missing a required layer, that criterion's text next to its id.

## info

Prints the resolved `spex.yaml` configuration as pretty JSON — the general-purpose form scripts pipe into `jq` for whatever field they need.

- `--print <field>` — print one top-level scalar field bare, without quoting or JSON escaping; errors (non-zero exit, message to stderr) when the field does not exist or is not a scalar (`repositories` is a list, so use plain `spex info` for it).
- `--repo <name>...` — limit output to the general fields (`project_name`, `cli_version`) plus the named repositories, matched by `repository_name` or `repository_short_name`; errors on a name matching no repository.

## rules

Prints the rules governing the selected repositories as YAML.

- Prints each rule that at least one selected repository lists in `rules`, once per rule no matter how many repositories list it, so an agent can read exactly what governs a repository before touching it.
- `--ai` drops fields an agent doesn't need to follow a rule (`sync_with_upstream` and `checks`), and adds a `path` field after `id` with the rule file's location relative to the workspace root (e.g. `rules/secret-handling.yaml`), so an agent can reference or edit the rule directly.
- `--id <id>...` — limit output to the given Rule IDs; defaults to all rules, and errors on an unrecognized ID.
- `--repo <name>...` — limit output to rules assigned to the named repositories, matched by `repository_name` or `repository_short_name`; defaults to all repositories in `spex.yaml`, and errors on an unrecognized name.
- `--output <path>` — write the rendered YAML to a file instead of stdout, since the full rules list is large and gets truncated when printed to a terminal or piped through a shell.

## specs

Prints the workspace specs as YAML.

- Prints every spec in the workspace, so an agent can read exactly what is specified before touching it.
- `--ai` adds a `path` field after `id` with the spec file's location relative to the workspace root (e.g. `specs/place-pizza-order.yaml`), so an agent can reference or edit the spec directly.
- `--id <id>...` — limit output to the given Spec IDs; defaults to all specs, and errors on an unrecognized ID.
- `--scope <name>...` — limit output to specs whose `scope` includes the given `repository_short_name`; defaults to all specs, and errors on an unrecognized name.
- `--output <path>` — write the rendered YAML to a file instead of stdout, since the full specs list is large and gets truncated when printed to a terminal or piped through a shell.

# Skills

Agent skills drive the artifact chain: each packages the instructions an AI coding agent follows to run one workflow step, applying every rule that governs the repositories it touches.

- Skills live in the spex repository, installed under `.agents/skills/`.
- A skill is triggered by name (e.g. `/spex-spec`) or loaded on demand by the agent from its description.
- Skills are portable instructions, not proprietary integrations: plain `SKILL.md` files any editor, terminal, or AI coding agent can load.
- A skill's `SKILL.md` frontmatter follows the [Agent Skills specification](https://agentskills.io/specification); setting `metadata.sync-with-upstream: false` opts a locally customized skill out of being overwritten by `spex update`.

The bundled skills:

- **spex-setup** — completes setup after `spex init`: fills in the workspace config descriptions, writes each repository's real commands into the `SKILL.md` `init` seeded for it, adds each repository's applicable rules, and records each repository's existing contracts. Requires AI, so it cannot run programmatically.
- **spex-spec** — turns a PRD, an RFD, a feature idea, a business rule, or a technical capability into one or more schema-valid specs at `specs/<spec-id>.yaml`.
- **spex-rule** — turns a convention into a schema-valid rule with checks; writes `rules/<rule-id>.yaml` plus any script bodies under `scripts/<rule-id>/`.
- **spex-contract** — derives or updates the interface contract from one or more specs.
- **spex-plan** — splits approved specs into an ordered, parallel-safe plan at `plans/<date>-<slug>.md`, with a status on every step and task.
- **spex-implement** — executes a plan step by step, or a plain user request directly, then finalizes the touched specs' status and checks the result. Deletes the plan file once every step is `completed`. Writes no tests and never sets a spec's status to `active` — that is `spex-test`'s job, since only it knows both impl and test annotations are in place.
- **spex-review-rule** — reviews a diff against the governing cross-cutting rules and coding standards, plus a baseline of Fowler code smells, and runs `spex check` for the affected repositories.
- **spex-review-spec** — reviews a diff against a spec, checking for missing, extra, or wrong implementation of its requirements.
- **spex-review** — runs `spex-review-rule` and `spex-review-spec` as parallel sub-agents against the same diff and reports both axes side by side.
- **spex-test** — adds or updates tests for an already-implemented change, from a plan, a diff, or an ad-hoc request. Never edits plan files, non-test code, or a spec's acceptance criteria; the only spec edits it makes are clearing `is_done: false` on a criterion `spex coverage` now shows impl and test traced, and setting `status: active` once every criterion is done.
- **spex-assembly-line** — runs `spex-implement` once, then a rule review/fix loop followed by a spec review/fix loop, each until a pass finds nothing or a 3-pass cap is hit, then `spex-test` once against the reviewed change.
- **spex-grill-me** — grills the user with one question at a time to stress-test a plan, decision, or idea before acting on it.
- **spex-create-skill** — reference for writing and editing agent skills well, so the user can create a new skill or review an existing one for quality.
- **spex-update** — updates a project's `cli_version` by running `spex update`, then applies `references/migration_guide.md`'s steps for every version between the old and the new `cli_version`.
- **spex-cavemen** — switches the agent to terse, filler-free replies to save tokens.

# Specs

## Spec

One coherent capability the system must provide — a product behavior, or a technical implementation needed to support the product's rules.

- Lives at `specs/<spec-id>.yaml`, nested subdirectories allowed. The `id` follows `schemas/spec-id.schema.json`: lowercase letters and hyphens only, at most 40 characters.
- Has required `id`, `title`, `description`, `acceptance_criteria`, and `contract`; optional `scope`, `blocked_by`, `status`, `coverage_ignore`, and `examples`.
- `contract` sets the contract coverage granularity: `single` (the migration default) requires one spec-level `@spec contract <spec-id>` annotation for the whole spec, `for-each-ac` requires a `@spec contract <spec-id>#<ac-id>` annotation for every acceptance criterion, `none` disables contract coverage.
- `examples` holds named illustrative snippets showing the spec applied in context — placement and usage that prose cannot carry.
- Holds its specifics in the acceptance criteria, ordered by RFC 2119 commitment level: `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`.
- Each acceptance criterion is `{id, must|must_not|should|should_not|may, is_done}`:
  - Exactly one of `must`, `must_not`, `should`, `should_not`, `may` is present, its value the verifiable statement.
  - `id` is a kebab-case slug, unique within the spec, naming the criterion's behavior rather than its position.
  - `is_done` is optional and defaults to `true`; set it to `false` to flag a criterion as not actually done yet. `spex check` reads it — not annotations — to decide whether a spec's status can be `active`. `spex coverage` reads `#<ac-id>`-qualified annotations separately; a criterion missing its annotations while flagged `is_done: false` is reported as pending, not a gap.
- `scope` lists the `repository_short_name`s of the repositories that implement the capability.
- `blocked_by` lists specs or rules that must be in place before this spec can be implemented; it holds only genuine, unsatisfied blockers, so an already shipped or established one is never listed.
- `status` defaults to `planned`; `active` means fully implemented and requires every acceptance criterion's `is_done` to be `true` or absent, `in-progress` is for a spec being implemented but not yet complete.
- `coverage_ignore` waives a coverage requirement, one `{layer, ac}` entry per waiver:
  - `layer` (`contract`, `impl`, or `test`) is required; without `ac`, it waives that layer for the whole spec.
  - With `ac`, it waives just that acceptance criterion's impl or test requirement — or, under `contract: for-each-ac`, its contract requirement.
  - The entry must match the spec's `contract` setting: whole-spec contract waives work under `single` and `for-each-ac`, per-criterion contract waives only under `for-each-ac`, and under `contract: none` a contract-layer entry is pointless. `spex check` errors on a mismatch.

```yaml
coverage_ignore:
  - layer: contract
  - layer: impl
    ac: no-null-typed-string
```

## Spec ID annotations

Annotations link a contract, implementation, or test to its spec, and optionally to one specific acceptance criterion.

- Form is `@spec contract|impl|test <spec-id>[#<ac-id>][, <spec-id>[#<ac-id>]]...`, matched anywhere in a file's text — as a code comment, or as the string value of any OpenAPI vendor extension (`x-*`) property for contracts, whose key name is not itself checked.
- Place one annotation on the smallest unit that actually satisfies the acceptance criterion — a specific field, function, or test case — not on the file, the test suite's top-level describe, or the enclosing type; widen the placement only when the criterion's scope genuinely spans that boundary (e.g. an endpoint, contract). Annotate once at the boundary, never nested: list every acceptance criterion id the boundary satisfies rather than adding a second annotation.
- An impl or test annotation carries the `#<ac-id>` of the acceptance criterion it satisfies: `spex coverage` counts a bare, unqualified annotation only toward a target with no acceptance criteria, such as a rule, never toward a criterion of a spec.
- A contract annotation carries `#<ac-id>` only under `contract: for-each-ac`, where every acceptance criterion needs its own annotated contract; under `contract: single` one bare `@spec contract <spec-id>` suffices, and a criterion-anchored annotation also counts as the spec-level contract.
- Never reference a Spec ID with no spec file.
- Spec traceability is required and enforced by `spex coverage`; rule traceability is optional.
- `spex check` enforces the grammar and the no-orphan rule, built in, in every repository regardless of which rules it opts into.

# Rules

## Rule

One coherent cross-cutting convention or rule: style, structure, security, traceability, or documentation.

- Lives at `rules/<rule-id>.yaml`, nested subdirectories allowed. The `id` follows `schemas/rule-id.schema.json`: lowercase letters and hyphens only, at most 40 characters.
- Has required `id`, `title`, `description`, and `acceptance_criteria`; optional `checks`, `examples`, and `sync_with_upstream`.
- A repository opts into a rule by listing it in `rules` in `spex.yaml`; the rule then applies repo-wide, not per feature. A rule has no lifecycle status, and its acceptance criteria carry no implementation flag.
- Holds its individual rules in the acceptance criteria, ordered `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`, each criterion a string starting with that commitment.
- `sync_with_upstream: true` marks a global framework rule that `spex update` overwrites (with its scripts) from the bundled template on every run; unsetting it customizes the rule locally and keeps those changes across updates.
- A deleted bundled rule is recognized as deliberate only once its id is added to `rules_unused` in `spex.yaml`; otherwise `update` treats its absence as new upstream and redeploys it.

## Checks

A check proves a rule's acceptance criterion.

- Has type `regexp` (file contents), `glob` (file paths), `ast-grep` (structural code patterns), or `script` (external program).
- `include` / `exclude` globs narrow the file set of every check except `per-repo` and `workspace` scripts, which discover their own files.
- `negate` inverts a `regexp`, `glob`, or `ast-grep` check: a forbidden pattern versus a required pattern. Negated checks are skipped when `check --files` narrows the file set, since they need every file to prove a forbidden pattern absent.
- `severity` is `error` (default) or `warn`.
- A repository inherits a rule's checks through `rules`.
- `ignored_errors` baselines existing errors so a check can be enforced going forward, skipping listed errors.
- A script check resolves at `scripts/<rule-id>/<script>`; `run` is `per-file` (default), `per-repo`, or `workspace`.
- `per-file` runs the script once per in-scope file with the file paths as arguments (batched); `per-repo` runs it once per repository in that repository's directory with no arguments; `workspace` runs it exactly once for the whole workspace with no arguments.
- Choose `per-repo` when a check must run once per governed repository, `workspace` when the script must run once for the whole workspace.
- A failing script reports one finding per line as `<location><TAB><message>`: `<location>` is a repository-relative file, optionally `<file>:<line>`. A line with no location (empty before the tab, or no tab at all) becomes a file-less finding, and a script that exits non-zero without printing anything reports a single file-less finding built from its stderr or exit status. A non-zero exit code still means the check failed.
- A script that needs data from `spex.yaml` (the project name, the repository list) runs `spex info` rather than parsing `spex.yaml` by hand.
- Every criterion a script can verify is proven by a check; AI inspection covers only the criteria that require judgment.

# Config (spex.yaml)

## Structure

Declares the project and its repositories.

- Root fields: required `project_name` and `cli_version`, plus optional `description`, `rules_unused` (Rule IDs `spex update` must not redeploy), and `repositories`.
- Each repository entry has `repository_name` (the full `<project-name>-<repository-short-name>`, matching its directory), `repository_short_name`, and `path` (relative to the spex directory) required, plus an optional `description`, `contracts` list, `rules` list, and `ignored_errors` baseline.
- Each `contracts` entry has a `path` and a `description`, both required; the path is user-owned, typically inside the owning repository, and relative to the spex directory.
- `--repo` accepts either a repository's full name or its short name.
- Validated against `schemas/spex.schema.json`.

## Versioning

Records and reconciles the CLI version that wrote the config.

- `spex.yaml` records the CLI version that wrote it as `cli_version`.
- The commands that load the workspace (`check`, `coverage`, `info`, `rules`, `specs`) warn when `cli_version` differs from the installed binary.
- A mismatch is resolved by running `spex update`.
