# Glossary

- **BRD** (Business Requirements Document) — the root source. Describes _why_ something is being built: business goals and stakeholder needs. Written by/for business stakeholders.

  ```md
  Goal: let customers order pizza online to increase sales.
  Success: 100 online orders per week.
  ```

- **PRD** (Product Requirements Document) — extracted from the BRD. Describes _what_ the product should do: features and user-facing behavior. Written by/for product, design, and engineering.

  ```md
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

- **Rule** — describes _how the code and artifacts are built_: one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules captured as acceptance criteria proven by `spex check`. Not tied to a single feature — it applies repo-wide. A specific capability the system provides is a spec, not a rule.

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

- **Rule ID** — a unique identifier assigned to a rule, used for referencing and traceability. Should convey the rule's contents so it can be understood without opening the rule.

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

- **Project** — the whole product being built, named after its workspace root folder (e.g. `shopping-notes`). One project spans several workspace repositories that share a single set of specs.

- **Project name** — the identifier of the project, taken from the name of its workspace root folder.

  ```text
  shopping-notes
  ```

- **Workspace** — the set of workspace repositories that spex manages together for one project, declared in `spex.yaml`.

- **Workspace root** — the top-level folder that holds a project: it groups the workspace repositories that spex manages.

  ```text
  shopping-notes/             <- workspace root
    shopping-notes-spex/      <- workspace repository
    shopping-notes-api/       <- workspace repository
    shopping-notes-frontend/  <- workspace repository
  ```

- **Workspace repository** (repository, repo) — a single git repository inside the workspace root that spex manages (`spex`, `api`, `frontend`, and so on). Each lives in its own repository so access can be granted per repository and one Spec ID can trace across many of them. Always call these workspace repositories (repos) or by more specific role (service, mobile).

- **Workspace repository name** (repository name, repo full name) — the identifier of a workspace repository, taken from the name of its folder inside the workspace root/git.

  ```text
  shopping-notes-spex
  shopping-notes-api
  shopping-notes-frontend
  ```

- **Workspace repository short name** (repository short name, repo short name) — workspace repository name excluding project name.

  ```text
  spex
  api
  frontend
  ```

- **AI coding agent** (AI agent, agent) — the tool a person picks to do the work: an autonomous assistant that reads, runs, and edits code on their behalf. Hosted inside an editor (Zed, VS Code, Cursor) or run from the terminal (Claude Code, Codex).

- **Agent Skill** (skill) — a modular, reusable capability that packages specialized instructions an AI agent loads on demand when a request matches the skill's description. Defined as a folder with a `SKILL.md` entry point, following the [Agent Skills](https://agentskills.io/) spec. In spex, a skill is the instructions an AI agent follows to do the job, taking all applicable rules and implementing a spec.
