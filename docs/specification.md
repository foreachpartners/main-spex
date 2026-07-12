# Specification

How the spex framework behaves, written as specs. Each spec has a short
description and a list of acceptance criteria (`ac`). See [glossary.md](glossary.md)
for terms, [cli.md](cli.md) for command flags, and the bundled `schemas/` for
field rules.

# Framework

## Artifact chain

Framework helps to turns business goals into verified code through a chain of artifacts, each
traceable by a shared Spec ID.

Flow runs BRD (why) → PRD (what) → capability spec (what it provides) → contract (interface, if applicable) → plan (how to build it, optional) → implementation (code) → test (proof).

### BRD

Business goals and stakeholder needs that product owner must identify, prioritize, and translate into PRD.

### PRD

Features and user-facing behavior extracted from the BRD by product owner as actionable user stories and specifications for the development team.

### Capability Spec

Extracted from the PRD by product owner and/or developer. It describes a capability the system must provide, with its individual rules captured as acceptance criteria. CS serves as single source of truth of product behavior. It's implemented and can be extended or changed over time.

### Contract

The machine-readable interface (OpenAPI, proto, GraphQL SDL, AsyncAPI) derived from a capability spec if applicable (API, cross-service communication contracts). It's small, easy to create, review and reason before bigger implementation and testing artifacts.

### Plan

Plan turns capability specs (and their contracts) into step by step implementation instructions for agent and human for review, with a status on every step and task so progress is trackable.

Plan should be simple, small and focused on interfaces instead of implementation or people will not review it.

A plan helps with large changes and is optional for small, self-contained ones.

## Spec kinds

Two kinds of spec capture two kinds of requirement: a capability the system
provides and a cross-cutting convention.

- A capability spec (CS) captures one coherent capability the system must provide — a product behavior, or a technical implementation needed to support the product's rules — with its individual rules in the acceptance criteria.
- A rule spec (RS) captures one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules in the acceptance criteria.
- A rule spec applies repo-wide, not per feature. The test: a specific capability the system provides → CS; a rule every change must follow, enforceable repo-wide → RS.

# Workspace

## Layout

A project spans several workspace repositories under one workspace root folder.

- Each workspace repository is its own git repository, with its directory named `<project-name>-<repository-short-name>` per RS-NAMING-CONVENTION (for example `pepperoni-pizza-spex`, `pepperoni-pizza-api`, `pepperoni-pizza-frontend`).
- The spex repository (`<project-name>-spex`) holds the specs, schemas, scripts, skills, config, and contracts — the single source of truth.
- spex locates the spex repository by finding the `spex.yaml` it contains, so commands work from the workspace root or any workspace repository regardless of the repository directory names.
- Every fact has one authoritative source and is referenced, not copied: the interface lives only in its native contract, a capability only in its capability spec, a convention only in its rule spec.
- Specs, contracts, and checks are plain files and deterministic commands, so no step of the workflow depends on a specific editor, terminal, or AI model.
- Each repository's interface contracts live in the `spex` repository's `contracts/` folder, declared per repository in `spex.yaml`.
- An editor workspace file (`<project-name>.code-workspace`) opens every repository together.

## Traceability

Every behavioral change traces to a spec, and one Spec ID traces across the repositories that satisfy it.

- The Spec ID (CS, RS) is the unit of traceability; every behavioral change traces to one, with no orphans.
- Traceability is bidirectional: top-down (spec → contract → plan → code → test) and bottom-up (test → code → plan → contract → spec).
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
- Scaffolds the bundled templates: `schemas/`, `scripts/`, `rs-specs/`, and skills.
- Creates empty `cs-specs/`, `plans/`, and `contracts/` folders.
- Installs skills under `.agents/skills/` and symlinks `.claude` → `.agents` so different agent tools detect the same skills.
- Writes a root `AGENTS.md` and symlinks `CLAUDE.md` → `AGENTS.md`; both are user-owned, so `init` seeds them only when absent and `update` never touches them.
- Adds each default rs-spec to repositories per its `include_for_all_repos` / `include_for_spex` flags.
- Directs the user to finish setup by running the `/spex-setup` skill.

## update

Refreshes the managed framework files from the installed binary.

- Rewrites the bundled skills and schemas with the versions in the installed binary.
- Rewrites a bundled rs-spec (and its scripts) only when the local copy is new or already has `managed: true`; a local spec without it is left untouched, letting consumers override a global spec.
- Overwrites managed files only; leaves plans and any files the user added untouched.
- Updates `cli_version` in `spex.yaml`.

## check

Runs each repository's rule-spec checks and gates on errors.

- Runs the rule-spec checks each repository lists in its `rs_specs`, reporting findings per repository.
- When the spex repository is in scope, also validates the spec set itself and attributes the findings to it: a capability spec whose acceptance criteria are all checked off (`[x]`) but whose `status` is not `active` or `deprecated` warns; a `blocked_by` entry that references a non-existing spec errors; a `blocked_by` entry that references an already-`active` spec (or any rule spec, since those always apply) warns, since a satisfied blocker should be removed.
- Exits non-zero when any `error`-severity finding is reported.
- `warn` findings do not affect the exit status.
- `--repo <name>...` limits the run to named repositories.
- `--files <path>...` limits the run to given files; a `per-repo` script then runs only in the repositories that own a listed file, unioned with any `--repo` named, while a `workspace` script always runs once.
- Skips files ignored by `.gitignore`, plus well-known generated files (e.g. `Cargo.lock`, `package-lock.json`).
- `--fill-preexisting-errors` baselines the current errors into `spex.yaml`, then exits without gating.
- `--verbose` reports each repository and each check once it finishes, with timings, on stderr.

## coverage

Reports annotation coverage for capability specs.

- Reports annotation coverage for every capability spec whose `status` is `active`.
- Requires an implementation, test, and contract annotation per active spec; a single one anywhere in any repository counts.
- Reports how many of each active spec's acceptance criteria are implemented, read from each criterion's `[ ]`/`[x]` checkbox.
- `coverage_ignore` relaxes named layers; it does not relax the acceptance-criteria checkboxes.
- Non-active specs appear only in the per-status totals.
- Marks each spec ✓ (all layers present and every criterion implemented), ⚠ (only an ignored layer missing), or ✗ (a required layer missing or a criterion unimplemented).
- Exits non-zero when any active spec has a missing required layer or an unimplemented acceptance criterion.

## check-schema

Validates JSON or YAML files against a JSON Schema.

- Validates JSON or YAML files against a JSON Schema, resolving cross-file `$ref`s offline.
- Requires `--schema` and `--files`.
- Exits non-zero when any file fails.
- Used by `RS-SPEX-CONFORMANCE` to keep spex's own spec files valid.

# Skills

Skills are agent workflows that drive the spec → plan → execute → check loop and other use cases.

- A skill is triggered by name (for example `/spex-capability-spec`) or loaded on demand by the AI agent.
- Skills live in the `spex` repository.
- Skills are portable instructions that work with any editor, terminal, or AI model, with no dependency on a specific vendor.

## spex-setup

Fills in the descriptions and icons left blank by `spex init` and collects each repository's contracts.

- Fills the missing `description` fields in `spex.yaml` for the project and each repository by inspecting their contents.
- Fills the folder icons in the editor workspace file.
- Copies each repository's existing contract files into the spex repository's `contracts/` folder, renaming each with its repository short name as a prefix (for example `openapi.yaml` from the `api` repository becomes `contracts/api.openapi.yaml`).
- Records each copied contract under its repository's `contracts` in `spex.yaml`.
- Is run right after `spex init`.

## spex-capability-spec

Turns a product need into schema-valid capability specs.

- Turns a PRD into one or more schema-valid capability specs `cs-specs/<spec-id>.yaml`.

## spex-rule-spec

Turns a convention into a schema-valid rule spec with checks.

- Turns a convention into one schema-valid rule spec.
- Proposes the automated checks that verify it.
- Writes `rs-specs/<spec-id>.yaml` plus any script bodies under `scripts/<spec-id>/`.

## spex-contract

Derives the machine-readable interface contract from approved capability specs.

- Reads the target specs, the affected repositories, their existing contracts, and the governing contract conventions.
- Writes or updates the endpoints, messages, and schemas in the `spex` repository's `contracts/` folder, annotated to their spec, and records new files in `spex.yaml`.
- Requires each target spec to already exist; when a spec is missing, stops and creates it with `spex-capability-spec` first.
- Runs after `spex-capability-spec` and before `spex-plan`.

## spex-plan

Splits approved specs into an ordered, parallel-safe plan.

- Reads the target specs, the affected repositories, and the governing rs-specs, then inspects the current code.
- Splits the work into ordered steps of parallel-safe tasks with disjoint write scopes.
- Requires each target spec to already exist; when a spec is missing, stops and creates it with `spex-capability-spec` first.
- Requires the contract to be settled with `spex-contract` first when any acceptance criterion crosses a repository boundary; when the work needs no interface change, proceeds straight from spec to plan.
- Writes `plans/<spec-id>/<date>-<slug>.md` with a status on every step and task.

## spex-implement

Executes a plan step by step, or a plain user request directly, and checks the result.

- Works through the units of work in order: a plan's steps top to bottom, or a bare request as one or more independent tasks; runs each unit's tasks as parallel agents.
- Marks each task and step `completed` as it lands when a plan exists, so an interrupted run resumes from the first incomplete step.
- Runs `spex check` on the affected repositories and fixes `error` findings.
- Sets each capability spec the work implemented or advanced to `active`, and ticks each implemented acceptance criterion's checkbox (`[ ]` → `[x]`), leaving any criterion that is not yet implemented as `[ ]`.

# Specs

## Capability spec (CS)

One coherent capability the system must provide — a product behavior, or a technical implementation needed to support the product's rules.

- Has `id`, `title`, `description`, `acceptance_criteria`, `scope`, and optional `blocked_by`, `status`, `coverage_ignore`.
- Holds its individual rules in the acceptance criteria as verifiable, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY - RFC 2119.
- Each acceptance criterion begins with an implementation checkbox — `[ ]` not implemented, `[x]` implemented — that `spex coverage` reads to report how many of an active spec's criteria are implemented.
- `scope` lists the repositories that implement the capability.
- `blocked_by` lists CS or RS specs that must be in place before this spec can be implemented; it holds only genuine, unsatisfied blockers, so a spec already shipped or established is never listed.
- `status` defaults to `planned`.

## Rule spec (RS)

One coherent cross-cutting convention or rule.

- Has `id`, `title`, `description`, `acceptance_criteria`, and optional `checks`, `examples`.
- Is a global rule that always applies; it has no lifecycle status and its acceptance criteria carry no implementation checkbox.
- Holds its individual rules in the acceptance criteria, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY - RFC 2119.
- `include_for_all_repos` adds it to every repository at init.
- `include_for_spex` overrides `include_for_all_repos` for the spex repository.
- `managed` marks a global framework spec that `spex update` overwrites (with its scripts) from the bundled template on every run. Consumers can unset it to customize the spec locally and keep their changes across updates.
- Each criterion should be provable by an automated check or by AI inspection.

## Checks

A check proves an acceptance criterion.

- Every criterion a script can verify is proven by a check; AI inspection covers only the criteria that require judgment.
- Has type `regexp` (file contents), `glob` (file paths), or `script` (external program).
- `negate` inverts a regexp or glob: a forbidden pattern versus a required pattern.
- `include` / `exclude` globs narrow the file set.
- `severity` is `error` (default) or `warn`.
- A repository inherits a spec's checks through `rs_specs`; `checks_override` patches one, `checks_additional` adds one.
- `checks_preexisting_errors` baselines existing errors so a check can be enforced going forward, skipping listed errors.
- Script checks resolve at `scripts/<spec-id>/<script>`; `run` is `per-file`, `per-repo`, or `workspace`.
- `per-file` runs once per in-scope file with the file paths as arguments (batched); `per-repo` runs once per repository with no arguments; `workspace` runs exactly once for the whole workspace with no arguments.
- `per-repo` and `workspace` scripts receive no files and discover their own, relative to the repository and the spex directory respectively; `include`/`exclude` only narrow `per-file` and `regexp`/`glob` checks.
- A failing script reports one finding per line as `<location><TAB><message>`.

## Spec ID annotations

Annotations link a contract, implementation, or test to its spec.

- Link a contract, implementation, or test to its spec at the layer boundary.
- Capability spec traceability is required and enforced by `spex coverage`; rule spec traceability is optional.
- Comment form is `@spec contract|impl|test <spec-id>`; OpenAPI form is `x-spec: <spec-id>`.
- Annotate once at the boundary, never nested.
- Never reference a Spec ID with no spec file.

# Config (spex.yaml)

## Structure

Declares the project and its repositories.

- Declares `project_name`, `cli_version`, an optional `description`, and the `repositories` list.
- Each repository has `repository_name` (the full `<project-name>-<repository-short-name>`, matching its directory), `repository_short_name`, `path` (relative to the spex directory), an optional `description`, an optional `contracts` list, and `rs_specs`.
- `--repo` accepts either a repository's full name or its short name.
- Each `contracts` entry has a `path` (relative to the spex directory, inside `contracts/`) and a `description`, both required.
- Each repository may add per-repo `checks_override`, `checks_additional`, and `checks_preexisting_errors`.
- Validated against `schemas/spex.schema.json`.

## Versioning

Records and reconciles the CLI version that wrote the config.

- `spex.yaml` records the CLI version that wrote it as `cli_version`.
- spex warns on every command when `cli_version` differs from the installed binary.
- A mismatch is resolved by running `spex update`.
