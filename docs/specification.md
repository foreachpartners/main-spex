# Specification

How the spex framework behaves, written as specs. Each spec has a short
description and a list of acceptance criteria (`ac`). See [glossary.md](glossary.md)
for terms, [cli.md](cli.md) for command flags, and the bundled `schemas/` for
field rules.

# Framework

## Artifact chain

Turns business goals into verified code through a chain of artifacts, each
traceable by a shared Spec ID.

- Flow runs BRD (why) → PRD (what) → business spec (how it behaves) → contract (interface) → plan (how to build it) → implementation (code) → test (proof).
- A BIZ Spec ID names a spec and links the contract, code, and tests that satisfy it.
- A contract is the machine-readable interface (OpenAPI, proto, GraphQL SDL, AsyncAPI) derived from a business spec.
- A contract is required wherever a machine-readable interface applies — a REST API must have an OpenAPI contract — and is optional otherwise, so a frontend spec may not have a contract.
- A plan turns approved specs into ordered steps of parallel-safe tasks that agents execute.
- A plan helps with large changes and is optional for small, self-contained ones.
- The AI writes every artifact and the human validates each one before the next is generated: a spec is approved before its contract, the contract before the plan, and the plan before implementation and tests.
- No artifact ships without a human review and approval, so every change traces back to a decision a human approved.

## Spec kinds

Two kinds of spec capture two kinds of requirement: product behavior and
cross-cutting convention.

- A business spec (BIZ) captures one coherent product capability or behavior, with its individual rules in the acceptance criteria.
- A technical spec (TECH) captures one coherent cross-cutting convention (style, structure, security, traceability, documentation), with its individual rules in the acceptance criteria.
- A technical spec applies repo-wide, not per feature.

# Workspace

## Layout

A project spans several workspace repositories under one workspace root folder.

- Each workspace repository is its own git repository, with its directory named `<project-name>-<repository-short-name>` per TECH-NAMING-CONVENTION (for example `pepperoni-pizza-spex`, `pepperoni-pizza-api`, `pepperoni-pizza-frontend`).
- The spex repository (`<project-name>-spex`) holds the specs, schemas, scripts, skills, config, and contracts — the single source of truth.
- spex locates the spex repository by finding the `spex.yaml` it contains, so commands work from the workspace root or any workspace repository regardless of the repository directory names.
- Every fact has one authoritative source and is referenced, not copied: the interface lives only in its native contract, product behavior only in its business spec, a convention only in its technical spec.
- Specs, contracts, and checks are plain files and deterministic commands, so no step of the workflow depends on a specific editor, terminal, or AI model.
- Each repository's interface contracts live in the `spex` repository's `contracts/` folder, declared per repository in `spex.yaml`.
- An editor workspace file (`<project-name>.code-workspace`) opens every repository together.

## Traceability

Every behavioral change traces to a spec, and one Spec ID traces across the repositories that satisfy it.

- The Spec ID (BIZ, TECH) is the unit of traceability; every behavioral change traces to one, with no orphans.
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
- Scaffolds the bundled templates: `schemas/`, `scripts/`, `tech-specs/`, and skills.
- Creates empty `biz-specs/`, `plans/`, and `contracts/` folders.
- Installs skills under `.agents/skills/` and symlinks `.claude` → `.agents` so different agent tools detect the same skills.
- Adds each default tech-spec to repositories per its `include_for_all_repos` / `include_for_spex` flags.
- Directs the user to finish setup by running the `/spex-setup` skill.

## update

Refreshes the managed framework files from the installed binary.

- Rewrites the bundled skills, schemas, scripts, and tech-specs with the versions in the installed binary.
- Overwrites managed files only; leaves specs, plans, and any files the user added untouched.
- Updates `cli_version` in `spex.yaml`.

## check

Runs each repository's TECH-spec checks and gates on errors.

- Runs the TECH-spec checks each repository lists in its `tech_specs`, reporting findings per repository.
- Exits non-zero when any `error`-severity finding is reported.
- `warn` findings do not affect the exit status.
- `--repo <name>...` limits the run to named repositories.
- `--files <path>...` limits the run to given files; a `per-repo` script then runs only in the repositories that own a listed file, unioned with any `--repo` named, while a `workspace` script always runs once.
- Skips files ignored by `.gitignore`, plus well-known generated files (e.g. `Cargo.lock`, `package-lock.json`).
- `--fill-preexisting-errors` baselines the current errors into `spex.yaml`, then exits without gating.
- `--verbose` reports each repository and each check once it finishes, with timings, on stderr.

## coverage

Reports annotation coverage for business specs.

- Reports annotation coverage for every business spec whose `status` is `active`.
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
- Used by `TECH-SPEX-CONFORMANCE` to keep spex's own spec files valid.

## spec-context

Prints the combined context of the named specs as Markdown for an AI agent.

- Takes one or more spec IDs, each a BIZ or TECH spec.
- For every ID, emits its `id`, `title`, `description`, and `acceptance_criteria`; TECH specs also emit their `examples`.
- Resolves each ID against the business specs first, then the technical specs.
- Prints one section per ID, in the order requested, to stdout.
- Exits non-zero when any ID matches no spec.
- Used by `spex-execute-plan` to load the governing specs in one call.

# Skills

Skills are agent workflows that drive the spec → plan → execute → check loop and other use cases.

- A skill is triggered by name (for example `/spex-create-biz-spec`) or loaded on demand by the AI agent.
- Skills live in the `spex` repository.
- Skills are portable instructions that work with any editor, terminal, or AI model, with no dependency on a specific vendor.

## spex-setup

Fills in the descriptions and icons left blank by `spex init` and collects each repository's contracts.

- Fills the missing `description` fields in `spex.yaml` for the project and each repository by inspecting their contents.
- Fills the folder icons in the editor workspace file.
- Copies each repository's existing contract files into the spex repository's `contracts/` folder, renaming each with its repository short name as a prefix (for example `openapi.yaml` from the `api` repository becomes `contracts/api.openapi.yaml`).
- Records each copied contract under its repository's `contracts` in `spex.yaml`.
- Is run right after `spex init`.

## spex-create-biz-spec

Turns a product need into schema-valid business specs.

- Turns a PRD into one or more schema-valid business specs `biz-specs/<spec-id>.yaml`.

## spex-create-tech-spec

Turns a convention into a schema-valid technical spec with checks.

- Turns a convention into one schema-valid technical spec.
- Proposes the automated checks that verify it.
- Writes `tech-specs/<spec-id>.yaml` plus any script bodies under `scripts/<spec-id>/`.

## spex-create-contract

Derives the machine-readable interface contract from approved business specs.

- Reads the target specs, the affected repositories, their existing contracts, and the governing contract conventions.
- Writes or updates the endpoints, messages, and schemas in the `spex` repository's `contracts/` folder, annotated to their spec, and records new files in `spex.yaml`.
- Requires each target spec to already exist; when a spec is missing, stops and creates it with `spex-create-biz-spec` first.
- Runs after `spex-create-biz-spec` and before `spex-create-plan`.

## spex-create-plan

Splits approved specs into an ordered, parallel-safe plan.

- Reads the target specs, the affected repositories, and the governing tech-specs, then inspects the current code.
- Splits the work into ordered steps of parallel-safe tasks with disjoint write scopes.
- Requires each target spec to already exist; when a spec is missing, stops and creates it with `spex-create-biz-spec` first.
- Requires the contract to be settled with `spex-create-contract` first when any acceptance criterion crosses a repository boundary; when the work needs no interface change, proceeds straight from spec to plan.
- Writes `plans/<spec-id>/<date>_<slug>.md` with a status on every step and task.

## spex-execute-plan

Executes a plan step by step and checks the result.

- Works through a plan step by step: runs each step's tasks as parallel agents, and waits between steps.
- Marks each task and step `completed` as it lands, so an interrupted run resumes from the first incomplete step.
- Runs `spex check` on the affected repositories and fixes `error` findings.
- Sets each implemented business spec's `status` to `active` once the steps pass, and ticks each implemented acceptance criterion's checkbox (`[ ]` → `[x]`), leaving any criterion that is not yet implemented as `[ ]`.

# Specs

## Business spec (BIZ)

One coherent product capability or behavior.

- Has `id`, `title`, `description`, `acceptance_criteria`, `scope`, and optional `depends_on`, `status`, `coverage_ignore`.
- Holds its individual rules in the acceptance criteria as verifiable, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY - RFC 2119.
- Each acceptance criterion begins with an implementation checkbox — `[ ]` not implemented, `[x]` implemented — that `spex coverage` reads to report how many of an active spec's criteria are implemented.
- `scope` lists the repositories that implement the capability.
- `status` defaults to `planned`.

## Technical spec (TECH)

One coherent cross-cutting convention.

- Has `id`, `title`, `description`, `acceptance_criteria`, and optional `checks`, `examples`.
- Holds its individual rules in the acceptance criteria.
- `include_for_all_repos` adds it to every repository at init.
- `include_for_spex` overrides `include_for_all_repos` for the spex repository.
- `readonly` marks a global framework spec, changed only upstream and pulled in by `spex update`.
- Each criterion should be provable by an automated check or by AI inspection.

## Checks

A check proves an acceptance criterion.

- Every criterion a script can verify is proven by a check; AI inspection covers only the criteria that require judgment.
- Has type `regexp` (file contents), `glob` (file paths), or `script` (external program).
- `negate` inverts a regexp or glob: a forbidden pattern versus a required pattern.
- `include` / `exclude` globs narrow the file set.
- `severity` is `error` (default) or `warn`.
- A repository inherits a spec's checks through `tech_specs`; `checks_override` patches one, `checks_additional` adds one.
- `checks_preexisting_errors` baselines existing errors so a check can be enforced going forward, skipping listed errors.
- Script checks resolve at `scripts/<spec-id>/<script>`; `run` is `per-file`, `per-repo`, or `workspace`.
- `per-file` runs once per in-scope file with the file paths as arguments (batched); `per-repo` runs once per repository with no arguments; `workspace` runs exactly once for the whole workspace with no arguments.
- `per-repo` and `workspace` scripts receive no files and discover their own, relative to the repository and the spex directory respectively; `include`/`exclude` only narrow `per-file` and `regexp`/`glob` checks.
- A failing script reports one finding per line as `<location><TAB><message>`.

## Spec ID annotations

Annotations link a contract, implementation, or test to its spec.

- Link a contract, implementation, or test to its spec at the layer boundary.
- Business spec traceability is required and enforced by `spex coverage`; technical spec traceability is optional.
- Comment form is `@spec contract|impl|test <spec-id>`; OpenAPI form is `x-spec: <spec-id>`.
- Annotate once at the boundary, never nested.
- Never reference a Spec ID with no spec file.

# Config (spex.yaml)

## Structure

Declares the project and its repositories.

- Declares `project_name`, `cli_version`, an optional `description`, and the `repositories` list.
- Each repository has `repository_name` (the full `<project-name>-<repository-short-name>`, matching its directory), `repository_short_name`, `path` (relative to the spex directory), an optional `description`, an optional `contracts` list, and `tech_specs`.
- `--repo` accepts either a repository's full name or its short name.
- Each `contracts` entry has a `path` (relative to the spex directory, inside `contracts/`) and a `description`, both required.
- Each repository may add per-repo `checks_override`, `checks_additional`, and `checks_preexisting_errors`.
- Validated against `schemas/spex.schema.json`.

## Versioning

Records and reconciles the CLI version that wrote the config.

- `spex.yaml` records the CLI version that wrote it as `cli_version`.
- spex warns on every command when `cli_version` differs from the installed binary.
- A mismatch is resolved by running `spex update`.
