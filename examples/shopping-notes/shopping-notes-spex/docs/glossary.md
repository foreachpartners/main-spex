# Glossary

- **PRD** (Product Requirements Document) — the root source. Describes _why_ something is being built and _what_ the product should do: goals, success, features, and user-facing behavior. Written by/for product, design, and engineering.

  ```md
  Goal: let customers order pizza online to increase sales.
  Success: 100 online orders per week.

  Customers can pick a pizza on the website, place an order, and pay online with a credit card.
  The daily order count should be tracked.
  ```

- **Spec** (specification) — a single structured requirement, verifiable and precise enough for engineers and AI agents to follow directly, describing _a capability the system must provide_: one coherent product behavior, or a technical implementation needed to support the product's rules, with its specifics captured as acceptance criteria proven by `spex coverage`. Most are extracted from the PRD, which typically breaks down into many specs; some capture supporting technical capabilities (an endpoint, integration, health check, performance target). Lives in `specs/`, distinct from a rule, which lives in `rules/`. The dividing line is capability versus convention.

  ```yaml
  id: place-pizza-order
  title: Customer can place an order
  description: Lets a customer turn their cart into a confirmed order.
  acceptance_criteria:
    - id: created-order-status
      must: "return 201 with the created order from POST /orders"
  contract: single
  ```

- **Rule** — describes _how the code and artifacts are built_: one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules captured as acceptance criteria proven by `spex check`. Not tied to a single feature — it applies across whole packages. A specific capability the system provides is a spec, not a rule.

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

- **Spex directory** (spex dir) — the directory holding `spex.yaml`, the specs, rules, schemas, docs, plans, and the contracts shared across packages. It is the spex repository in a multi-repo project, and the `spex/` folder at the repository root in a single-repo or monorepo project.

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

- **Package config** (`spex.package.yaml`) — optional file at the root of a package that holds the coverage waivers and the pre-existing error baseline concerning that package, merged by spex with the same fields of `spex.yaml`.

- **AI coding agent** (AI agent, agent) — the tool a person picks to do the work: an autonomous assistant that reads, runs, and edits code on their behalf. Hosted inside an editor (Zed, VS Code, Cursor) or run from the terminal (Claude Code, Codex).

- **Agent Skill** (skill) — a modular, reusable capability that packages specialized instructions an AI agent loads on demand when a request matches the skill's description. Defined as a folder with a `SKILL.md` entry point, following the [Agent Skills](https://agentskills.io/) spec. In spex, a skill is the instructions an AI agent follows to do the job, taking all applicable rules and implementing a spec.
