# Specification

How the spex framework behaves, written as specs: each section states one behavior with its acceptance criteria. [framework-principles.md](framework-principles.md) is the non-negotiable foundation, [conventions.md](conventions.md) the engineering values it follows, [glossary.md](glossary.md) the terms, [cli/](cli/README.md) the command behavior, and the bundled `schemas/` the field rules.

# Framework

## Artifact chain

Turn business goals into verified code through a chain of artifacts, each traceable by a shared Spec ID; [glossary.md](glossary.md) defines every artifact in it and its lifecycle.

`BRD` (why, optional) -> `PRD` (what, optional) -> spec (capability) -> contract (interface, if applicable) -> implementation (code) -> test (proof)

- The product owner extracts each artifact from the one before it: the PRD from the BRD, and a spec from the PRD — or from the BRD or a feature idea directly when there is no PRD.
- A contract is derived from a spec when applicable, and stays small next to the implementation, so it is cheap to create, review, and reason about first.
- A plan is optional for small, self-contained changes: an ordered set of steps turning approved specs and their contracts into implementation instructions.
- Each artifact is approved before the next is generated ([framework-principles.md](framework-principles.md#4-human-in-control) §4).

# Workspace

## Layout

A project spans several workspace repositories under one workspace root folder; [glossary.md](glossary.md) defines the workspace, its repositories, and their naming.

- The spex repository holds the specs, rules (with their scripts), schemas, skills, config, and the contracts shared across repositories. An editor workspace file (`<project-name>.code-workspace`) opens every repository together.
- The layout follows [conventions.md](conventions.md): tool-agnostic (specs, contracts, and checks are plain files and deterministic commands) and single-source-of-truth (every fact one authoritative source, referenced rather than copied).

## Traceability

Every behavioral change traces to a spec, and one Spec ID traces across the repositories that satisfy it. The principle, including the bidirectional trace and commit-level provenance, is [framework-principles.md](framework-principles.md#6-traceability) §6; the annotation grammar that carries it is [below](#spec-id-annotations).

# CLI

The CLI behavior — every command, its flags, reports, and exit codes — is specified in [cli/](cli/README.md).

# Skills

Agent skills drive the artifact chain: each packages the instructions an AI coding agent follows to run one workflow step, applying every rule that governs the repositories it touches.

- Skills live in the spex repository, installed under `.agents/skills/`; the bundled ones ship from `templates/skills/`, each `SKILL.md` describing what it does and when to load it.
- A skill is triggered by name (e.g. `/spex-spec`) or loaded on demand by the agent from its description.
- Every bundled `SKILL.md` ends with a `# Project` section for project-specific additions; `spex update` rewrites everything above it and never touches it.
- A skill's `SKILL.md` frontmatter follows the [Agent Skills specification](https://agentskills.io/specification); whether `spex update` overwrites it is in [cli/update.md](cli/update.md).

# Specs

## Spec

A spec, defined in [glossary.md](glossary.md), lives at `specs/<spec-id>.yaml`, nested subdirectories allowed; the Spec ID is the filename, whatever the nesting.

- The `id` follows `schemas/spec-id.schema.json`: lowercase letters, digits and hyphens only, at most 40 characters.
- Has required `id`, `title`, `description`, `acceptance_criteria`, and `contract`; optional `scope`, `blocked_by`, `status`, `coverage_ignore`, and `examples`.
- `contract` sets the contract coverage granularity: `single` (the migration default) requires one spec-level `@spec contract <spec-id>` annotation for the whole spec, `for-each-ac` requires a `@spec contract <spec-id>#<ac-id>` annotation for every acceptance criterion, `none` disables contract coverage.
- `examples` holds named illustrative snippets showing the spec applied in context — placement and usage that prose cannot carry.
- Holds its specifics in the acceptance criteria, ordered by RFC 2119 commitment level: `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`.
- Each acceptance criterion is `{id, must|must_not|should|should_not|may, is_done}`:
  - Exactly one of `must`, `must_not`, `should`, `should_not`, `may` is present, its value the verifiable statement.
  - `id` follows `schemas/ac-id.schema.json`: lowercase letters, digits and hyphens only, at most 40 characters. It is unique within the spec and names the criterion's behavior rather than its position.
  - `is_done` is optional and defaults to `true`; set it to `false` to flag a criterion as not actually done yet. `spex check` reads it — not annotations — to decide whether a spec's status can be `active`. `spex coverage` reads it only as a marker that a criterion with no impl annotation yet is pending rather than a gap; see [Coverage states](#coverage-states).
- `scope` lists the `repository_short_name`s of the repositories that implement the capability.
- `blocked_by` lists specs that must be in place before this spec can be implemented; it holds only genuine, unsatisfied blockers, so an already shipped or established one is never listed.
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

A waiver can also be declared outside the spec file, in `spex.yaml` or a repository's `spex.repo.yaml`, so the repository that cannot satisfy a layer owns the waiver without editing the spec; see [Config](#config-spexyaml).

## Spec ID annotations

Annotations link a contract, implementation, or test to its spec, and optionally to one specific acceptance criterion.

- Form is `@spec contract|impl|test <spec-id>[#<ac-id>][, <spec-id>[#<ac-id>]]...`, matched anywhere in a file's text — as a code comment, or as the string value of any OpenAPI vendor extension (`x-*`) property for contracts, whose key name is not itself checked.
- Place one annotation on the smallest unit that actually satisfies the acceptance criterion — a specific field, function, or test case — not on the file, the test suite's top-level describe, or the enclosing type; widen the placement only when the criterion's scope genuinely spans that boundary (e.g. an endpoint, contract). Annotate once at the boundary, never nested: list every acceptance criterion id the boundary satisfies rather than adding a second annotation.
- An impl or test annotation carries the `#<ac-id>` of the acceptance criterion it satisfies: `spex coverage` never counts a bare, unqualified annotation toward a criterion of a spec.
- A contract annotation carries `#<ac-id>` only under `contract: for-each-ac`, where every acceptance criterion needs its own annotated contract; under `contract: single` one bare `@spec contract <spec-id>` suffices, and a criterion-anchored annotation also counts as the spec-level contract.
- `<spec-id>` and `<ac-id>` follow the same id grammar as the ids they reference; an annotation naming an id outside it is malformed.
- Never reference a Spec ID with no spec file. `spex check` enforces the grammar and the no-orphan rule, built in, in every repository regardless of which rules it opts into ([cli/check.md](cli/check.md#self-validation)).

## Coverage states

`spex coverage` classifies each acceptance criterion of an `active` or `in-progress` spec from its annotations and its `is_done` flag. A required layer is the criterion's impl and test, plus its contract when the spec's `contract` is `for-each-ac`, minus whatever `coverage_ignore` waives.

| Criterion state                                                | Result    |
| -------------------------------------------------------------- | --------- |
| Every required layer annotated                                 | `covered` |
| A required layer missing, `is_done` `true` or absent           | `gap`     |
| A required layer missing, `is_done: false`, impl annotated     | `gap`     |
| A required layer missing, `is_done: false`, no impl annotation | `pending` |

- A gap is an error and makes `spex coverage` exit non-zero, whatever the spec's status. Pending is work not yet started and never fails.
- `is_done: false` is mostly a marker: `spex check` still reads it for the spec's `status`, and coverage uses it only to keep a criterion pending until it has an impl annotation. It never excuses a criterion that already has an impl annotation, so a change is verified in the repository that makes it, without first updating the spec in another repository.
- The impl annotation is the trigger, not the test or the contract, so contract-first and test-first work stays pending until the impl lands. With the impl waived by `coverage_ignore` there is normally no impl annotation to trigger on, so such a criterion stays pending until `is_done` is cleared.
- Under `contract: single` the spec-level contract is one more requirement of the spec: a gap once any criterion is `is_done` `true` or absent or has an impl annotation, pending while every criterion is `is_done: false` with no impl annotation.
- A spec in any other status (`planned`, `blocked`, `deprecated`) is not classified at all, however much of it is annotated.

# Rules

## Rule

A rule, defined in [glossary.md](glossary.md), lives at `rules/<rule-id>.yaml`, nested subdirectories allowed; the Rule ID is the filename, whatever the nesting.

- The `id` follows `schemas/rule-id.schema.json`: lowercase letters, digits and hyphens only, at most 40 characters.
- Has required `id`, `title`, `description`, and `acceptance_criteria`; optional `checks`, `examples`, and `sync_with_upstream`.
- A repository opts into a rule by listing it in `rules` in `spex.yaml`; the rule then applies repo-wide, not per feature. A rule has no lifecycle status, and its acceptance criteria carry no implementation flag.
- Holds its individual rules in the acceptance criteria, ordered `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`, each criterion a string starting with that commitment.
- `sync_with_upstream: true` marks a global framework rule kept in sync from the bundled template by `spex update` (with its scripts); unsetting it customizes the rule locally and keeps those changes across updates.
- A deleted bundled rule is recognized as deliberate only once its id is added to `rules_unused` in `spex.yaml`.

## Checks

A check proves a rule's acceptance criterion.

- Has type `regexp` (file contents), `glob` (file paths), `ast-grep` (structural code patterns), or `script` (external program).
- `include` / `exclude` globs narrow the file set of every check except `per-repo` and `workspace` scripts, which discover their own files.
- `negate` inverts a `regexp`, `glob`, or `ast-grep` check: a forbidden pattern versus a required pattern.
- `severity` is `error` (default) or `warn`.
- A repository inherits a rule's checks through `rules`.
- A script check resolves in the rule's script folder, named after the rule file and beside it (`rules/<rule-id>.yaml` → `rules/<rule-id>/<script>`), with `run` `per-file` (default), `per-repo`, or `workspace`; how each runs, and what a script must report, is in [cli/check.md](cli/check.md#script-checks).
- A script receives the absolute path of the spex repository as `SPEX_DIR`; when it needs data from `spex.yaml` (the project name, the repository list) it reads it with `spex jq` rather than parsing `spex.yaml` by hand.

# Config (spex.yaml)

## Structure

Declares the project and its repositories.

- Root fields: required `project_name` and `cli_version`, plus optional `description`, `rules_unused` (Rule IDs `spex update` must not redeploy), `repositories`, and `coverage_ignore`.
- Each repository entry has `repository_name` (the full `<project-name>-<repository-short-name>`, matching its directory), `repository_short_name`, and `path` (relative to the spex directory) required, plus an optional `description`, `contracts` list, `rules` list, and `ignored_errors` baseline.
- Each `contracts` entry has a `path` and a `description`, both required; the path is user-owned, typically inside the owning repository, and relative to the spex directory.
- `coverage_ignore` waives coverage requirements by spec id, one `{spec_id, layer, ac}` entry per waiver: `spec_id` and `layer` are required, `ac` is optional, and `layer` and `ac` mean what they do in a [spec's `coverage_ignore`](#spec).
- Validated against `schemas/spex.schema.json`.

```yaml
coverage_ignore:
  - spec_id: place-pizza-order
    layer: contract
  - spec_id: place-pizza-order
    layer: impl
    ac: no-null-typed-string
```

## Repository config (spex.repo.yaml)

Lets a workspace repository own the waivers and the baseline that concern it.

- Optional: `spex.repo.yaml` sits at the root of a workspace repository, and a repository without one has nothing to merge.
- Has optional `coverage_ignore`, with the entries of the root `coverage_ignore` of `spex.yaml`, and `ignored_errors`, with the entries of a repository's `ignored_errors` there, its `files` relative to this repository's root.
- Everything is merged by union, none overrides another:
  - A spec's coverage waivers are its own `coverage_ignore`, the `coverage_ignore` entries of `spex.yaml` naming its id, and those of every repository's `spex.repo.yaml` naming its id, whichever repositories the spec is `scope`d to.
  - A repository's `ignored_errors` baseline is its entry's in `spex.yaml` plus its own `spex.repo.yaml`'s.
- Validated against `schemas/spex.repo.schema.json`.

```yaml
coverage_ignore:
  - spec_id: place-pizza-order
    layer: impl
    ac: no-null-typed-string
ignored_errors:
  - rule_id: writing-style
    check: no-cyrillic
    files: [README.md]
```

## Versioning

Records and reconciles the CLI version that wrote the config.

- `spex.yaml` records the CLI version that wrote it as `cli_version`.
- A mismatch with the installed binary is reported by the commands that load the workspace and resolved by running `spex update` ([cli/update.md](cli/update.md)).
