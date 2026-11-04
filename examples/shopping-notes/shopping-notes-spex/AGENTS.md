# spex

This project is governed by `spex`. The AI-first development framework that turns specifications into verified code — keeping AI running at full speed while ensuring slop-free results.

## Artifact chain

PRD (why and what) -> spec (capability) -> contract (interface, if applicable) -> implementation (code) -> test (proof).

## Traceability

Every behavioral change traces to a spec acceptance criterion via a trace comment `@spec impl|test|contract <spec-id>#<ac-id>` e.g `@spec impl place-pizza-order#auth-required` for implementation acceptance criterion.

For `@spec test` and `@spec impl` ac-id for each acceptance criterion is required. Contract type is defined in spec, e.g `contract: for-each-ac`:

- `for-each-ac` - contract for each ac is required, same as for `@spec impl` and `@spec test`.
- `single` - one contract required for entire spec, traced by `spec-id` only, e.g. `@spec contract place-pizza-order`.
- `none` - no contract required.

Trace comment should be placed on the smallest unit that actually satisfies the AC — a specific field, function, or test case — not on the file, the test suite's top-level describe, or the enclosing type. Widen the placement only when the AC's scope genuinely spans that boundary (e.g. an endpoint, contract). Target minimal tracing amount that make sense.

## Definition of done

A change isn't finished until:

- `spex check` passes (no error-severity findings) for every affected package.
- `spex coverage` shows no gap for any spec acceptance criterion the change touches.
- Every acceptance criterion touched carries `@spec impl <spec-id>#<ac-id>` and `@spec test <spec-id>#<ac-id>` tracing annotations, e.g. `@spec impl place-pizza-order#auth-required` and `@spec test place-pizza-order#auth-required`.

## CLI commands

Run from anywhere inside the project; `spex <command> --help` lists every option.

- `spex info [--package <name>...]` — print `spex.yaml` merged with each `spex.package.yaml`: project and per-package configuration, including paths.
- `spex check [--package <name>...] [--files <path>...]` — run the rule checks; fails on error findings.
- `spex coverage [--package <name>...] [-v]` — report `@spec` annotation coverage per acceptance criterion.
- `spex specs [--id <spec-id>...] [--package <name>...] [--search <keyword>...] [--fields <field>...] [--output <file>]` — print specs, each with its file `path` after its `id`; only those containing any keyword, or only the named fields of each (e.g. `--fields id title description` for a catalog).
- `spex rules [--id <rule-id>...] [--package <name>...] [--search <keyword>...] [--fields <field>...] [--output <file>]` — print the active rules per package the same way; `--fields id path title description acceptance_criteria examples` leaves out the `checks`.
- `spex jq [-r] [-c] <filter> [file]` — run a jq filter on JSON or YAML from a file or stdin, e.g. `spex info | spex jq -r .project_name`.
- `spex check-schema --schema <schema> --files <path>...` — validate JSON or YAML files against a JSON Schema.
- `spex lib <specs|rules> [--id <id>...] [--search <keyword>...] [--fields <field>...] [--output <file>]` — print the bundled template specs or rules as a YAML list, in full, only the named ones, only those containing any keyword, or only the named fields of each (e.g. `--search password --fields id description` for a catalog of matches).
- `spex lib rules --id <rule-id>... --copy <dir>` — write the named bundled rules, with their script folders, into `<dir>` (e.g. `<spex-dir>/rules`) to take them into the project; never overwrites.

Prefer `--output <file>` for `specs` and `rules`: long output gets truncated in a terminal. `spex init` and `spex update` run through the `spex-setup` and `spex-update` skills.

## Core discipline

- **Simplicity**: All technical assets, tools, and processes must be simple and intuitive for any teammate or AI to quickly understand, execute, and safely change.
- **Single source of truth**: every business rule, behavior, contract, and configuration value must be mastered in exactly one place. Any other system that needs this information must reference, link to, or generate from this primary source rather than duplicating it. Repeating wording or an idea for readability is not duplication.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables. Add a comment only when the name cannot carry the meaning alone — a non-obvious why, hidden constraint, or surprising behavior — in one short line. Never comment what the code does, restate a name, or narrate an extraction or refactor.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Systemic Fix Over Quick Fix**: Always implement the proper, maintainable solution. Never apply a workaround, hack, band-aid, or temporary fix, unless the user explicitly asks for one in that request. If the proper fix requires refactoring, architectural changes, or expanding the scope of the current task, stop immediately, present the proper solution, and ask the user for explicit permission before proceeding.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless triggered by the "Systemic Fix Over Quick Fix" rule.
- **Stop on major findings**: When investigation reveals a major problem — a bug, a spec/code mismatch, a failing check, a design flaw — related to the task or not, stop. Report it with evidence and ask the user how to proceed. Never fix it silently, work around it, or ignore it. Report minor problems at the end of the task.
- **Propose major improvements**: When, at any step, a substantially simpler or better approach than the requested one emerges, stop, propose it, and wait for the user's decision before continuing. Propose a behavior change as a spec change first, since the spec is the source of truth. Report minor improvements at the end of the task.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/): `<type>[optional scope]: <description>`, with an optional body separated by a blank line. See `rules/commit-message-format.yaml`.

Tag every spec-related commit with its ID(s) using @spec: `@spec <Spec ID>[#<AC ID>][, <Spec ID>[#<AC ID>]]`.

Examples:

```
style: format md files
feat(ui): list available pizzas @spec list-pizza-menu
fix(cart): reject negative quantities @spec cart-add-item, cart-validate
feat(orders): set status on order creation @spec place-pizza-order#created-order-status
```

## Glossary

- **PRD** (Product Requirements Document) — the short project overview that comes first: _what_ we are building and _why_, without details. It gives people and AI agents the big picture they need to write specs that fit the project's purpose. One per project, kept at the level of the overall goal, success, and main capabilities: the details live in specs.

  ```md
  Goal: let customers order pizza online to increase sales.
  Success: 100 online orders per week.

  Customers can pick a pizza on the website, place an order, and pay online with a credit card.
  The daily order count should be tracked.
  ```

- **Spec** (specification) — a single structured requirement, verifiable and precise enough for engineers and AI agents to follow directly, describing _a capability the system must provide_: one coherent product behavior, or a technical implementation needed to support the product's rules, with its specifics captured as acceptance criteria proven by `spex coverage`. Each is written in line with the PRD: most break down its main capabilities, some capture supporting technical capabilities (an endpoint, integration, health check, performance target). Specs are the single source of truth for the project: its contracts, code, tests and other artifacts come from them and must adhere to them. Lives in `specs/`, distinct from a rule, which lives in `rules/`. The dividing line is capability versus convention.

  ```yaml
  id: place-pizza-order
  title: Customer can place an order
  description: Lets a customer turn their cart into a confirmed order.
  acceptance_criteria:
    - id: created-order-status
      must: "return 201 with the created order from POST /orders"
  contract: single
  ```

- **Rule** — describes _how the code and artifacts are built_: one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules captured as acceptance criteria proven by `spex check`. Not tied to a single feature — it applies across whole packages. A specific capability the system provides is a spec, not a rule. Each package must adhere to the rules applicable to it, listed in `spex.yaml`.

  ```yaml
  id: secret-handling
  title: Secret protection
  description: Keeps credentials out of the repository and out of build output.
  acceptance_criteria:
    - "MUST pass a secret scan on every commit in CI"
  ```

- **Spec ID** — a unique identifier assigned to a spec, used for referencing and traceability. Should convey the spec's contents so it can be understood without opening the spec.

  ```text
  place-pizza-order
  secret-handling
  ```

- **Acceptance Criterion ID** (AC ID) — an identifier assigned to one acceptance criterion, unique within its own spec. Names the criterion's behavior, not its position, so it survives reordering or inserting other criteria, and should convey the criterion's contents so it can be understood without opening the spec.

  ```text
  auth-required
  ```

- **Rule ID** — a unique identifier assigned to a rule, used for referencing. Should convey the rule's contents so it can be understood without opening the rule.

  ```text
  secret-handling
  ```

- **Specs library** — template specs for capabilities many projects need, bundled in the `spex` binary. List them with `spex lib specs`.

- **Rules library** — rules for conventions many projects share, bundled in the `spex` binary. List them with `spex lib rules`.

- **Traceability** (tracing, trace) — the `@spec` annotations stitching together spec, contracts, implementations, and tests, used to ensure coverage and to look up any of spec, contract, implementation, or test from any other, in either direction.

  ```text
  @spec impl place-pizza-order#created-order-status
  ```

- **Contract** — derived from a spec, if applicable. The interface to build against: OpenAPI, `.proto`, GraphQL SDL, AsyncAPI, or a Figma mockup for UI. AI implements from spec and contract.

  ```yaml
  paths:
    /orders:
      post:
        operationId: placeOrder
        responses:
          "201":
            description: Order created
  ```

- **Plan** — derived from one or more approved specs (and their contracts). Turns specs into an ordered set of steps whose tasks AI agents execute, with a status on every step and task so progress is trackable. A spec's plans live in the `spex` repository under `plans/<date>-<slug>.md`. A plan is a working document: it exists to split a large change into reviewable, resumable steps while it's in flight. Once every step is `completed`, `spex-implement` deletes the plan file — the spec, code, and tests already record what shipped, more accurately and durably than a step-by-step plan would.

  ```md
  # Plan: Customer can place an order

  ## Step 1 — Add endpoint

  Status: not-started
  ```

- **Project** — the whole product being built, named after its project root folder (e.g. `shopping-notes`). One project spans one or several packages, declared in `spex.yaml`, that share a single set of specs.

- **Project name** — the identifier of the project, taken from the name of its project root folder.

  ```text
  shopping-notes
  ```

- **Project root** — the top-level folder that holds a project. In a multi-repo project it groups the packages that spex manages, each a git repository; in a single-repo or monorepo project it is the root of the git repository.

  ```text
  shopping-notes/             <- project root
    shopping-notes-spex/      <- package
    shopping-notes-api/       <- package
    shopping-notes-frontend/  <- package
  ```

- **Spex directory** (spex dir) — the directory holding `spex.yaml`, the specs, rules, schemas, docs, plans, and the contracts shared across packages. It is the spex repository in a multi-repo project, and the `spex/` folder at the repository root in a single-repo or monorepo project. It is itself a package of the project in every layout.

- **Package** — one deliverable part of the project that spex manages (`spex`, `api`, `frontend`, `mobile`, `infra`, `docs`, and so on), with its own code, tests, contracts, and rules. In a multi-repo project each package is its own git repository inside the project root, so access can be granted per repository and one Spec ID can trace across many of them. In a monorepo each package is a folder of the repository, listed in `spex.yaml` without being a git repository of its own; in a single-repo project the one repository is the single package. Not a dependency or a package published to a registry. Always call these packages, or by a more specific role (service, mobile, infra, docs).

- **Package name** — the identifier of a package, taken from the name of its folder inside the project root; in a single-repo project, the project name.

  ```text
  shopping-notes-spex
  shopping-notes-api
  shopping-notes-frontend
  ```

- **Package short name** — package name excluding project name; in a single-repo project, the project name itself.

  ```text
  spex
  api
  frontend
  ```

- **Package config** (`spex.package.yaml`) — optional file at the root of a package that holds the excluded files, the coverage waivers and the pre-existing error baseline concerning that package, merged by spex into the same fields of that package's entry in `spex.yaml`, so they apply to that package only.

- **AI coding agent** (AI agent, agent) — the tool a person picks to do the work: an autonomous assistant that reads, runs, and edits code on their behalf. Hosted inside an editor (Zed, VS Code, Cursor) or run from the terminal (Claude Code, Codex).

- **Agent Skill** (skill) — a modular, reusable capability that packages specialized instructions an AI agent loads on demand when a request matches the skill's description. Defined as a folder with a `SKILL.md` entry point, following the [Agent Skills](https://agentskills.io/) spec. In spex, a skill is the instructions an AI agent follows to do the job, taking all applicable rules and implementing a spec.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
