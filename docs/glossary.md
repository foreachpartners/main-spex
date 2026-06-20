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

- **Spec** (specification) — a single structured requirement, verifiable and precise enough for engineers and AI agents to follow directly. Comes in two types, distinguished by the Spec ID prefix: business (`BIZ`) and technical (`TECH`).

- **Business Spec** (`BIZ`) — extracted from the PRD. Describes _how the product behaves_: one coherent product capability or behavior, with its individual rules captured as acceptance criteria. A PRD typically breaks down into many business specs.

  ```yaml
  id: BIZ-PLACE-PIZZA-ORDER
  title: Customer can place an order
  acceptance_criteria:
    - "POST /orders returns 201 with the created order."
  ```

- **Technical Spec** (`TECH`) — describes _how the code and artifacts are built_: one coherent cross-cutting convention (style, structure, security, traceability, documentation), with its individual rules captured as acceptance criteria. Not tied to a single feature — it applies repo-wide.

  ```yaml
  id: TECH-SECRET-HANDLING
  title: Secret protection
  acceptance_criteria:
    - "Secret scan check passes on every commit in CI."
  ```

- **Spec ID** — a unique identifier assigned to a spec, used for referencing and traceability. The prefix encodes the spec type.

  ```text
  BIZ-PLACE-PIZZA-ORDER
  TECH-SECRET-HANDLING
  ```

- **Contract** — derived from a business spec. The machine-readable _interface_ definition: endpoints, methods, status codes, errors, auth. OpenAPI for HTTP APIs, `.proto` for gRPC, GraphQL SDL, AsyncAPI for events. AI implements from spec and contract.

  ```yaml
  paths:
    /orders:
      post:
        operationId: placeOrder
        responses:
          "201":
            description: Order created
  ```

- **Project** — the whole product being built, named after its workspace root folder (e.g. `pepperoni-pizza`). One project spans several workspace repositories that share a single set of specs.

- **Project name** — the identifier of the project, taken from the name of its workspace root folder.

  ```text
  pepperoni-pizza
  ```

- **Workspace** — the set of workspace repositories that spex manages together for one project, declared in `spex.yaml`.

- **Workspace root** — the top-level folder that holds a project: it groups the workspace repositories that spex manages.

  ```text
  pepperoni-pizza/   <- workspace root
    spex/            <- workspace repository
    api/             <- workspace repository
    frontend/        <- workspace repository
  ```

- **Workspace repository** (repository, repo) — a single git repository inside the workspace root that spex manages (`spex`, `api`, `frontend`, `contracts`, and so on). Each lives in its own repository so access can be granted per repository and one Spec ID can trace across many of them. Always call these workspace repositories (repos) or by more specific role (service, mobile).

- **Workspace repository name** (repository name) — the identifier of a workspace repository, taken from the name of its folder inside the workspace root.

  ```text
  spex
  api
  frontend
  ```

- **AI coding agent** (AI agent, agent) — the tool a person picks to do the work: an autonomous assistant that reads, runs, and edits code on their behalf. Hosted inside an editor (Zed, VS Code, Cursor) or run from the terminal (Claude Code, Codex).

- **Agent Skill** (skill) — a modular, reusable capability that packages specialized instructions an AI agent loads on demand when a request matches the skill's description. Defined as a folder with a `SKILL.md` entry point, following the [Agent Skills](https://agentskills.io/) spec.
