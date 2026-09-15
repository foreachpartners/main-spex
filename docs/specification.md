# Specification

How the spex framework behaves, written as specs. Each spec has a short
description and a list of acceptance criteria (`ac`). See [glossary.md](glossary.md)
for terms, [cli.md](cli.md) for command flags, and the bundled `schemas/` for
field rules.

# Framework

## Artifact chain

Framework helps turn business goals into verified code through a chain of artifacts, each
traceable by a shared Spec ID.

Flow runs BRD (why) -> PRD (what) -> spec (what it provides) -> contract (interface, if applicable) -> plan (how to build it, optional) -> implementation (code) -> test (proof).

### BRD

Business goals and stakeholder needs that product owner must identify, prioritize, and translate into PRD.

### PRD

Features and user-facing behavior extracted from the BRD by product owner as actionable user stories and specifications for the development team.

### Spec

Extracted from the PRD by product owner and/or developer. It describes a capability the system must provide, with its specifics captured as acceptance criteria. A spec serves as single source of truth of product behavior. It's implemented and can be extended or changed over time.

### Contract

The interface derived from a spec if applicable. For a machine-readable interface (API, cross-service communication) it's OpenAPI, proto, GraphQL SDL, or AsyncAPI. For a UI-facing spec with no machine-readable interface, it's a design reference: a markdown document linking the spec to its visual design (e.g. a Figma frame). It's small, easy to create, review and reason before bigger implementation and testing artifacts.

### Plan

Plan turns specs (and their contracts) into step by step implementation instructions for agent and human for review, with a status on every step and task so progress is trackable.

Plan should be simple, small and focused on interfaces instead of implementation or people will not review it.

A plan helps with large changes and is optional for small, self-contained ones.

A plan is a working document, not a permanent artifact: useful while a change is in flight, so a step splits a big task into reviewable pieces and a status line lets work pause and resume across sessions unchanged. Once every step is `completed` and the change has shipped, the plan has nothing left to say that the code, spec, and git history don't already say better and more accurately — the spec records what must be true, the code and tests record what is true, and the plan's blow-by-blow steps only drift out of date as the code evolves further. Left in the repository, a finished plan is pure noise: a stale, unmaintained document that AI and human search alike keep surfacing, and that outsiders (or agents) can mistake for current instructions. `spex-implement` therefore deletes a plan file once it finishes executing it — completed plans do not accumulate.

## Specs

A spec captures one coherent capability the system must provide — a product behavior, or a technical implementation needed to support the product's rules — with its specifics in the acceptance criteria. Lives in `specs/`.

- The test: a specific capability the system provides -> spec.
- `specs/` may group files in nested subdirectories (e.g. `specs/frontend/checkout-flow.yaml`). The Spec ID is always just the filename, regardless of nesting.

## Rules

A rule captures one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules in the acceptance criteria. Lives in `rules/`.

- A rule applies repo-wide, not per feature. The test: a rule every change must follow, enforceable repo-wide -> rule.
- `rules/` may group files in nested subdirectories (e.g. `rules/frontend/frontend-testing.yaml`). The Rule ID is always just the filename, regardless of nesting.

## Naming

All names are chosen to match clear and precise meaning for human and AI.

- **Spec** signals a system capability — a function the product performs — mirroring how product and engineering already talk about "functional requirements"; a spec answers what the system does.
- **Rule** signals an enforceable convention every change must follow, matching how AI coding tools already use "rule" for repo-wide instructions (e.g. editor and agent rules files) and how linters name their checks (ESLint rules, Clippy lints); a rule answers how the system is built, and reads as a directive both a human and an AI agent treat as binding rather than optional.
- "Rule" is reserved for rules. Spec acceptance criteria are called specifics or criteria, never rules, so the word "rule" always means "repo-wide, enforceable convention" and never collides with a single capability's acceptance criteria.
- The spec type is conveyed only by which directory a spec lives in (`specs/` or `rules/`) — IDs carry no type prefix.
- A spec's identifier is a Spec ID (`schemas/spec-id.schema.json`); a rule's identifier is a Rule ID (`schemas/rule-id.schema.json`). Both follow the same lowercase-letters-and-hyphens format, limited to 40 characters to avoid long filenames/refs.
- Every id must be unique across both `specs/` and `rules/`, since Spec IDs and Rule IDs share the same naming convention and could otherwise collide.

# Workspace

## Layout

A project spans several workspace repositories under one workspace root folder.

- Each workspace repository is its own git repository, with its directory named `<project-name>-<repository-short-name>` per naming-convention (for example `pepperoni-pizza-spex`, `pepperoni-pizza-api`, `pepperoni-pizza-frontend`).
- The spex repository (`<project-name>-spex`) holds the specs, schemas, scripts, skills, config, and contracts — the single source of truth.
- spex locates the spex repository by finding the `spex.yaml` it contains, so commands work from the workspace root or any workspace repository regardless of the repository directory names.
- Every fact has one authoritative source and is referenced, not copied: the interface lives only in its native contract, a capability only in its spec, a convention only in its rule.
- Specs, contracts, and checks are plain files and deterministic commands, so no step of the workflow depends on a specific editor, terminal, or AI model.
- Each repository's interface contracts are declared per repository in `spex.yaml`; the contract file's location is user-owned, most commonly inside the owning repository itself, with the `spex` repository's `contracts/` folder reserved for contracts genuinely shared across repositories.
- An editor workspace file (`<project-name>.code-workspace`) opens every repository together.

## Traceability

Every behavioral change traces to a spec, and one Spec ID traces across the repositories that satisfy it.

- The Spec ID is the unit of traceability; every behavioral change traces to one, with no orphans.
- Traceability is bidirectional: top-down (spec -> contract -> plan -> code -> test) and bottom-up (test -> code -> plan -> contract -> spec).
- Separate repositories give per-repository access control (least privilege).
- One Spec ID can trace across many repositories.
- Checking out a commit shows exactly which specs the code was built against.

# CLI

## Global flags

Apply to every subcommand.

- `--verbose` (`-v`) prints progress to stderr as work executes, leaving stdout for the command's actual output.
- Currently only `check` emits progress: each repository and each check it runs, with the time each took.

## init

Creates the spex repository (`<project-name>-spex`) in the workspace root and scaffolds the framework.

- Prompts to confirm the project name, defaulting to the workspace root folder name.
- Creates the spex repository directory named `<project-name>-spex`.
- Prompts to confirm the discovered workspace repositories, deriving each `repository_short_name` from its directory name minus the `<project-name>-` prefix.
- Writes `spex.yaml`, recording the installed CLI version as `cli_version`.
- Writes the editor workspace file with a folder per repository.
- Scaffolds the bundled templates: `schemas/`, `docs/`, `scripts/`, `rules/`, and skills.
- Creates empty `specs/`, `plans/`, and `contracts/` folders.
- Installs skills under `.agents/skills/` and symlinks `.claude` -> `.agents` so different agent tools detect the same skills.
- Seeds a dummy `SKILL.md` for each workspace repository other than the spex repository itself, named after the repository with only a frontmatter `description` (no body). The description states the skill must be loaded before reading, writing, testing, or running any code in that repository, so an agent working across the workspace can autoload it from context — not just on an explicit `/name` invocation. This gives every repository a stable place for its own commands and context, without waiting on a human to write it first; `spex-setup` fills in the body.
- Writes a root `AGENTS.md` and symlinks `CLAUDE.md` -> `AGENTS.md`; both are user-owned, so `init` seeds them only when absent and `update` never touches them.
- Copies every bundled rule into the spex repository's `rules/`, but leaves every repository's `rules` in `spex.yaml` empty.
- Directs the user to finish setup by running the `/spex-setup` skill, which adds each rule that actually applies to a given repository.

## update

Refreshes the managed framework files from the installed binary.

- Rewrites the bundled schemas and docs with the versions in the installed binary.
- Rewrites a bundled skill already present locally only when its `metadata.sync-with-upstream` is not explicitly `false`; a local skill that opts out is left untouched, letting consumers override a global skill.
- Rewrites a bundled rule (and its scripts) already present locally only when it has `sync_with_upstream: true`; a local rule without it is left untouched, letting consumers override a global rule.
- Deploys a bundled rule missing locally, unless its id is listed in `rules_unused` in `spex.yaml`; listing an id there is how a user records a deliberate deletion so it stays deleted on future updates.
- Overwrites managed files only; leaves plans and any files the user added untouched.
- Updates `cli_version` in `spex.yaml`.
- Prints the `CHANGELOG.md` entries newer than the previous `cli_version`, newest first; prints nothing if there are none, and prints every entry if the previous `cli_version` isn't found in the changelog.

## check

Runs each repository's rule checks and gates on errors.

- Runs the rule checks each repository lists in its `rules`, reporting findings per repository.
- When the spex repository is in scope, also validates the spec set itself and attributes the findings to it: a spec whose acceptance criteria are all done (`is_done` `true` or absent) but whose `status` is not `active` or `deprecated` warns; a spec whose `status` is `active` with any acceptance criterion still `is_done: false` errors, since `active` means fully implemented; a `blocked_by` entry that references a non-existing spec errors; a `blocked_by` entry that references an already-`active` spec (or any rule, since those always apply) warns, since a satisfied blocker should be removed; a `scope` entry that does not match any repository's `repository_short_name` in `spex.yaml` errors. This reads only `is_done`, never annotations — proving `is_done` is actually true is `spex coverage`'s job, not `spex check`'s.
- Also errors when a spec file, rule file, or `spex.yaml` declares its properties out of order relative to their JSON Schema's declared property order, at every nesting level the schema describes (for example a rule's `checks[].*` items, or `spex.yaml`'s `repositories[].*` and `repositories[].contracts[].*`), and when an acceptance criterion id is duplicated within the same spec.
- Exits non-zero when any `error`-severity finding is reported.
- `warn` findings do not affect the exit status.
- `--repo <name>...` limits the run to named repositories.
- `--files <path>...` limits the run to given files; a `per-repo` script then runs only in the repositories that own a listed file, unioned with any `--repo` named, while a `workspace` script always runs once.
- Negated `regexp` and `glob` checks are skipped when `--files` is passed, because a negated check needs the full file set to prove a forbidden pattern is absent; running it on a subset would produce false positives.
- Skips files ignored by `.gitignore`, plus well-known generated files (e.g. `Cargo.lock`, `package-lock.json`).
- `--fill-preexisting-errors` baselines the current errors into `spex.yaml`, then exits without gating.
- `--verbose` reports each repository and each check once it finishes, with timings, on stderr.
- Every finding with a location renders it as a terminal hyperlink to the absolute file it actually points at, so it opens correctly no matter the shell's current working directory; the displayed text stays the short repository-relative path.

## coverage

Reports annotation coverage for specs, per acceptance criterion.

- Reports annotation coverage for every spec whose `status` is `active` or `in-progress`.
- `--scope <name>...` limits the report to specs whose `scope` includes one of the given repository short names; the annotation search itself still spans every workspace repository. Defaults to all specs; errors on an unrecognized name.
- Requires a contract annotation per tracked spec (spec-level; a single one anywhere in any repository counts), and requires an impl and a test annotation for every acceptance criterion, each qualified with that criterion's id (`@spec impl|test <spec-id>#<ac-id>`). A bare, unqualified `@spec impl|test <spec-id>` never satisfies a criterion's requirement — only a `#<ac-id>`-qualified annotation does, since a single bare annotation would otherwise trivially satisfy every criterion.
- `coverage_ignore` can waive the contract layer for the whole spec, the impl or test layer for the whole spec, or the impl or test layer for one specific acceptance criterion (see below).
- Reports how many of each tracked spec's acceptance criteria are fully traced (both a required impl and a required test annotation present).
- Specs that are neither `active` nor `in-progress` are listed by ID under their status (e.g. `planned`, `blocked`), not just counted.
- Its "Gaps" section only lists tracked specs with a gap (✗: the contract missing, or any acceptance criterion missing a required impl or test annotation); specs with no gap (fully covered, or only ignored requirements missing) are counted in the section header but not listed individually. A gapped spec's missing detail names which acceptance criterion ids are missing which layer.
- Gaps are grouped by `scope`, sorted alphabetically, with specs that have no scope grouped last under `unscoped`. A spec listed under more than one `scope` is duplicated under each of its scope groups; the "Gaps" header count still reflects each gapped spec once.
- Ends with a one-line summary: the spec count per status (`in-progress`, `planned`, `active`) and the gap count.
- Exits non-zero only when an **active** spec has a missing required layer or acceptance criterion; gaps on `in-progress` specs are reported but don't affect the exit code.

### coverage_ignore

Waives a coverage requirement. Each entry is `{layer, ac}`: `layer` (`contract`, `impl`, or `test`) is required; `ac` is optional.

- Without `ac`, waives that layer for the whole spec (every acceptance criterion, for `impl`/`test`).
- With `ac`, an acceptance criterion id, waives just that one criterion's impl or test requirement. Not valid with `layer: contract`, since `x-spec` has no acceptance-criterion granularity.

```yaml
coverage_ignore:
  - layer: contract
  - layer: impl
    ac: no-null-typed-string
```

## rules

Prints each repository's active rules as YAML.

- For every repository in `spex.yaml`, prints every rule listed in its `rules`, so an agent can read exactly what currently governs a repository before touching it.
- `--ai` drops fields an agent doesn't need to follow a rule: `sync_with_upstream` and `checks`, and adds a `path` field after `id` with the rule file's location relative to the workspace root (e.g. `rules/secret-handling.yaml`), so an agent can reference or edit the rule directly.
- `--id <id>...` filters to one or more given Rule IDs.
- `--repo <name>...` limits output to the named repositories, matched by either `repository_name` or `repository_short_name`.
- `--output <path>` writes the rendered YAML to a file instead of stdout, since the full rules list is large and can get truncated when printed to a terminal or piped through a shell.

## specs

Prints the workspace specs as YAML.

- Prints every spec in the workspace, so an agent can read exactly what is specified before touching it.
- `--ai` adds a `path` field after `id` with the spec file's location relative to the workspace root (e.g. `specs/place-pizza-order.yaml`), so an agent can reference or edit the spec directly.
- `--id <id>...` filters to one or more given Spec IDs.
- `--scope <name>...` limits output to specs whose `scope` includes the given repository short name (`repository_short_name`).
- `--output <path>` writes the rendered YAML to a file instead of stdout, since the full specs list is large and can get truncated when printed to a terminal or piped through a shell.

## check-schema

Validates JSON or YAML files against a JSON Schema.

- Validates JSON or YAML files against a JSON Schema, resolving cross-file `$ref`s offline.
- Requires `--schema` and `--files`.
- Exits non-zero when any file fails.
- `spex check` uses it, built in, to keep spex's own spec files valid.

# Skills

Skills are agent workflows that drive the spec -> plan -> execute -> check loop and other use cases.

- A skill is triggered by name (for example `/spex-spec`) or loaded on demand by the AI agent.
- Skills live in the `spex` repository.
- Skills are portable instructions that work with any editor, terminal, or AI model, with no dependency on a specific vendor.

See skill body for more details. A skill's `SKILL.md` frontmatter follows the [Agent Skills specification](https://agentskills.io/specification); setting `metadata.sync-with-upstream: false` opts a locally customized skill out of being overwritten by `spex update`.

## spex-setup

Complete setup step which requires AI and can't be done programmatically. Fills in the descriptions and icons left blank by `spex init`, collects each repository's contracts, and writes each repository's real commands into the per-repository `SKILL.md` `spex init` seeded. Is run right after `spex init`.

## spex-spec

Turns a PRD into one or more schema-valid specs `specs/<spec-id-lowercase>.yaml`.

## spex-rule

Turns a convention into a schema-valid rule with checks. Writes `rules/<rule-id-lowercase>.yaml` plus any script bodies under `scripts/<rule-id-lowercase>/`.

## spex-contract

Derives the machine-readable interface contract from approved specs.

## spex-plan

Splits approved specs into an ordered, parallel-safe implementation plan. Writes `plans/<date>-<slug>.md` with a status on every step and task.

## spex-implement

Executes a plan step by step, or a plain user request directly, then finalizes the touched specs' status and checks the result. Deletes the plan file once every step is `completed` — a finished plan has nothing left to say that the spec, code, and git history don't already say more accurately. Does not write tests, and never sets a spec's status to `active` — that's `spex-test`'s job, since only it knows both impl and test annotations are in place.

## spex-review-rule

Reviews a diff against governing cross-cutting rules and coding standards, plus a baseline of Fowler code smells, and runs `spex check` for the affected repositories.

## spex-review-spec

Reviews a diff against a spec, checking for missing, extra, or wrong implementation of its requirements.

## spex-review

Runs `spex-review-rule` and `spex-review-spec` as parallel sub-agents against the same diff and reports both axes side by side.

## spex-test

Adds or updates tests for an already-implemented change — from a plan, a diff, or an ad-hoc request. Never edits plan files, non-test code, or a spec's acceptance criteria — the only spec edits it makes are clearing `is_done: false` on a criterion `spex coverage` now shows fully impl+test traced, and setting `status: active` once every criterion is done.

## spex-assembly-line

Runs `spex-implement` once, then a rule review/fix loop followed by a spec review/fix loop, each until a pass finds nothing or a 3-pass cap is hit, then `spex-test` once against the reviewed change.

## spex-grill-me

Grills the user with one question at a time to stress-test a plan, decision, or idea before acting on it.

## spex-create-skill

Reference for writing and editing agent skills well, so the user can create a new skill or review an existing one for quality.

## spex-cavemen

Switches the agent to terse, filler-free replies to save tokens.

# Specs

## Spec

One coherent capability the system must provide — a product behavior, or a technical implementation needed to support the product's rules.

- Has `id`, `title`, `description`, `acceptance_criteria`, `scope`, and optional `blocked_by`, `status`, `coverage_ignore`, `examples`.
- `examples` holds named illustrative snippets showing the spec applied in context — placement and usage that prose cannot carry.
- Holds its specifics in the acceptance criteria as verifiable, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY - RFC 2119.
- Each acceptance criterion is `{id, must|must_not|should|should_not|may, is_done}`: exactly one of `must`, `must_not`, `should`, `should_not`, `may` is present, its value the verifiable statement; `id` is a kebab-case slug, unique within the spec, naming the criterion's behavior rather than its position, and immutable once an annotation references it — a change in meaning takes a new id, never a rename of the old one. `is_done` is optional and defaults to `true` when absent — set it to `false` to flag a criterion as not actually done yet; `spex check` reads it (not annotations) to decide whether a spec's status can be `active`. `spex coverage` separately reads `#<ac-id>`-qualified annotations to report how many of an active or in-progress spec's criteria are impl+test traced, regardless of `is_done`.
- `scope` lists the `repository_short_name`s of the repositories that implement the capability.
- `blocked_by` lists specs or rules that must be in place before this spec can be implemented; it holds only genuine, unsatisfied blockers, so a spec already shipped or established is never listed.
- `status` defaults to `planned`; `active` means fully implemented and requires every acceptance criterion's `is_done` to be `true` or absent, `in-progress` is for a spec being implemented but not yet complete.

## Spec ID annotations

Annotations link a contract, implementation, or test to its spec, and optionally to one specific acceptance criterion.

- Link a contract, implementation, or test to its spec at the layer boundary.
- Spec traceability is required and enforced by `spex coverage`; rule traceability is optional.
- Comment form is `@spec contract|impl|test <spec-id>[#<ac-id>][, <spec-id>[#<ac-id>]]...`; OpenAPI form is `x-spec: <spec-id>` (no `#<ac-id>` form — contracts have no per-criterion granularity).
- An impl or test annotation should carry the `#<ac-id>` of the acceptance criterion it satisfies, since `spex coverage` only counts a bare, unqualified annotation toward a target with no acceptance-criteria ids (e.g. a rule); a bare annotation on a spec never satisfies any of its criteria.
- Annotate once at the boundary, never nested; list every acceptance criterion id the boundary satisfies rather than adding a second nested annotation.
- Never reference a Spec ID with no spec file.

# Rules

## Rule

One coherent cross-cutting convention or rule.

- Has `id`, `title`, `description`, `acceptance_criteria`, and optional `checks`, `examples`.
- Is a global rule that always applies; it has no lifecycle status and its acceptance criteria carry no implementation checkbox.
- Holds its individual rules in the acceptance criteria, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY - RFC 2119.
- `sync_with_upstream` marks a global framework rule that `spex update` overwrites (with its scripts) from the bundled template on every run. Consumers can unset it to customize the rule locally and keep their changes across updates.
- Each criterion should be provable by an automated check or by AI inspection.
- A deleted bundled rule is only recognized as deliberate once its id is added to `rules_unused` in `spex.yaml`; otherwise `update` treats its absence as new upstream and redeploys it.

## Checks

A check proves a rule's acceptance criterion.

- Every criterion a script can verify is proven by a check; AI inspection covers only the criteria that require judgment.
- Has type `regexp` (file contents), `glob` (file paths), or `script` (external program).
- `negate` inverts a regexp or glob: a forbidden pattern versus a required pattern. Negated checks are skipped when `check --files` narrows the file set, since they need every file to prove a forbidden pattern is absent.
- `include` / `exclude` globs narrow the file set.
- `severity` is `error` (default) or `warn`.
- A repository inherits a rule's checks through `rules`.
- `ignored_errors` baselines existing errors so a check can be enforced going forward, skipping listed errors.
- Script checks resolve at `scripts/<rule-id-lowercase>/<script>`; `run` is `per-file`, `per-repo`, or `workspace`.
- `per-file` runs once per in-scope file with the file paths as arguments (batched); `per-repo` runs once per repository with no arguments; `workspace` runs exactly once for the whole workspace with no arguments.
- `per-repo` and `workspace` scripts receive no files and discover their own, relative to the repository and the spex directory respectively; `include`/`exclude` only narrow `per-file` and `regexp`/`glob` checks.
- Choose `per-repo` when a check needs to run once per governed repository and `workspace` when script needs to run once.
- A failing script reports one finding per line as `<location><TAB><message>`.
- A script that needs data from `spex.yaml` (the project name, the repository list) runs `spex info` rather than parsing `spex.yaml` by hand; see `spex info` in `docs/cli.md`.

# Config (spex.yaml)

## Structure

Declares the project and its repositories.

- Declares `project_name`, `cli_version`, an optional `description`, and the `repositories` list.
- Each repository has `repository_name` (the full `<project-name>-<repository-short-name>`, matching its directory), `repository_short_name`, `path` (relative to the spex directory), an optional `description`, an optional `contracts` list, and `rules`.
- `--repo` accepts either a repository's full name or its short name.
- Each `contracts` entry has a `path` (relative to the spex directory, inside `contracts/`) and a `description`, both required.
- Each repository may add per-repo `ignored_errors`.
- Validated against `schemas/spex.schema.json`.

## Versioning

Records and reconciles the CLI version that wrote the config.

- `spex.yaml` records the CLI version that wrote it as `cli_version`.
- spex warns on every command when `cli_version` differs from the installed binary.
- A mismatch is resolved by running `spex update`.
