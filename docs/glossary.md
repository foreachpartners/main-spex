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

- **Spec** (specification) — a single structured requirement, verifiable and precise enough for engineers and AI agents to follow directly. Comes in two types, distinguished by the Spec ID prefix: functional (`FS`) and rule (`RS`). The dividing line is capability versus convention: a specific capability the system must provide → `FS`; a rule every change must follow, enforceable repo-wide → `RS`.

- **Functional Spec** (`FS`) — describes _a capability the system must provide_: one coherent product behavior, or a technical implementation needed to support the product's rules, with its specifics captured as acceptance criteria. Most are extracted from the PRD, which typically breaks down into many functional specs; some capture supporting technical capabilities (an endpoint, integration, health check, performance target).

  ```yaml
  id: FS-PLACE-PIZZA-ORDER
  title: Customer can place an order
  acceptance_criteria:
    - "[ ] MUST return 201 with the created order from POST /orders"
  ```

- **Rule Spec** (`RS`) — describes _how the code and artifacts are built_: one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules captured as acceptance criteria. Not tied to a single feature — it applies repo-wide. A specific capability the system provides is a functional spec (`FS`), not a rule spec.

  ```yaml
  id: RS-SECRET-HANDLING
  title: Secret protection
  acceptance_criteria:
    - "Secret scan check passes on every commit in CI."
  ```

- **Spec ID** — a unique identifier assigned to a spec, used for referencing and traceability. The prefix encodes the spec type.

  ```text
  FS-PLACE-PIZZA-ORDER
  RS-SECRET-HANDLING
  ```

- **Contract** — derived from a functional spec. The machine-readable _interface_ definition: endpoints, methods, status codes, errors, auth. OpenAPI for HTTP APIs, `.proto` for gRPC, GraphQL SDL, AsyncAPI for events. AI implements from spec and contract. A repository's contracts live in the `spex` repository's `contracts/` folder.

  ```yaml
  paths:
    /orders:
      post:
        operationId: placeOrder
        responses:
          "201":
            description: Order created
  ```

- **Design reference** — a contract variant for UI-facing specs: a markdown document linking each spec to its visual design (e.g. a Figma frame), annotated like any other contract. Used when the interface a spec exposes is visual rather than machine-readable.

- **Plan** — derived from one or more approved functional specs (and their contracts). Turns specs into an ordered set of steps whose tasks AI agents execute, with a status on every step and task so progress is trackable. A spec's plans live in the `spex` repository under `plans/<spec-id>/<date>-<slug>.md`.

  ```md
  # Plan: Customer can place an order

  ## Step 1 — Add endpoint

  Status: not-started
  ```

- **Project** — the whole product being built, named after its workspace root folder (e.g. `pepperoni-pizza`). One project spans several workspace repositories that share a single set of specs.

- **Project name** — the identifier of the project, taken from the name of its workspace root folder.

  ```text
  pepperoni-pizza
  ```

- **Workspace** — the set of workspace repositories that spex manages together for one project, declared in `spex.yaml`.

- **Workspace root** — the top-level folder that holds a project: it groups the workspace repositories that spex manages.

  ```text
  pepperoni-pizza/             <- workspace root
    pepperoni-pizza-spex/      <- workspace repository
    pepperoni-pizza-api/       <- workspace repository
    pepperoni-pizza-frontend/  <- workspace repository
  ```

- **Workspace repository** (repository, repo) — a single git repository inside the workspace root that spex manages (`spex`, `api`, `frontend`, and so on). Each lives in its own repository so access can be granted per repository and one Spec ID can trace across many of them. Always call these workspace repositories (repos) or by more specific role (service, mobile).

- **Workspace repository name** (repository name, repo full name) — the identifier of a workspace repository, taken from the name of its folder inside the workspace root/git.

  ```text
  pepperoni-pizza-spex
  pepperoni-pizza-api
  pepperoni-pizza-frontend
  ```

- **Workspace repository short name** (repository short name, repo short name) — workspace repository name excluding project name.

  ```text
  spex
  api
  frontend
  ```

- **AI coding agent** (AI agent, agent) — the tool a person picks to do the work: an autonomous assistant that reads, runs, and edits code on their behalf. Hosted inside an editor (Zed, VS Code, Cursor) or run from the terminal (Claude Code, Codex).

- **Agent Skill** (skill) — a modular, reusable capability that packages specialized instructions an AI agent loads on demand when a request matches the skill's description. Defined as a folder with a `SKILL.md` entry point, following the [Agent Skills](https://agentskills.io/) spec.
