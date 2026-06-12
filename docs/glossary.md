# Glossary

- **BRD** (Business Requirements Document) — the root source. Describes _why_ something is being built: business goals and stakeholder needs. Written by/for business stakeholders.

  ```md
  Goal: let customers order pizza online to increase sales.
  Success: 100 online orders per week.
  ```

- **PRD** (Product Requirements Document) — extracted from the BRD. Describes _what_ the product should do: features and user-facing behavior. Written by/for product, design, and engineering.

  ```md
  Customers can pick a pizza on website, place an order and pay online with credit card.
  Daily orders amount should be tracked.
  ```

- **Spec** (specification) — a single structured requirement, verifiable and precise enough for engineers and AI agents to follow directly. Comes in two types, distinguished by the Spec ID prefix: business (`BIZ`) and technical (`TECH`).

- **Business Spec** (`BIZ`) — extracted from the PRD. Describes _how the product behaves_: a single business rule with acceptance criteria. A PRD typically breaks down into many business specs.

  ```yaml
  id: BIZ-PLACE-PIZZA-ORDER
  title: Customer can place an order
  acceptance_criteria:
    - "POST /orders returns 201 with the created order."
  ```

- **Technical Spec** (`TECH`) — describes _how the code is built_: a cross-cutting technical rule on code style, structure, testing, or security. Not tied to a single feature — it applies repo-wide.

  ```yaml
  id: TECH-SECRET-HANDLING
  title: No secrets in the repository
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

- **Workspace** — the top-level folder that groups everything for a single project: the `spex` directory and the sibling repositories it manages `api`, `frontend` and so on.

  ```text
  pizza-shop/        <- workspace root folder
    spex/            <- workspace repository
    api/             <- workspace repository
    frontend/        <- workspace repository
  ```
