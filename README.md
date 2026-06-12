# spex

AI-first specification-driven development framework with human validation.

_The name `spex` comes from a casual shorthand for specifications._

## Prerequisites

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/) (required for building the spex CLI)
- **Git**: [Install Git](https://git-scm.com/install/)
- **spex CLI**: Clone spex repository and install spex cli tool:
  ```sh
  git clone https://github.com/ziimakc/spex
  cd spex
  cargo install --path . --force
  ```

## Quickstart

1. Put all project git repositories in one shared folder (workspace root folder)
2. Run `spex init` in workspace root to create `pizza-shop/spex` repository

This creates the following structure:

```
pizza-shop/              # workspace root
├── server/              # existing server repository
├── client/              # existing client repository
└── spex/                # newly spex repository
```

## Why

**Specs over Guesswork**: Casual requests force an AI to guess, leading to inaccurate repository navigation and assumption-based task completion. Clear, well-defined business specifications (`BIZ`) eliminate guesswork, providing explicit acceptance criteria for both the AI to follow and the developer to verify.

**Technical Rules as Specs**: Cross-cutting technical rules — code style, structure, testing, security — are captured as technical specs (`TECH`) instead of living as tribal knowledge. They apply repo-wide and are proven by automated checks (linters, CI gates) or AI, so every change follows the same rules no matter who — human or AI model — writes the code.

**Human validation**: Vibe coding ships working code fast, but every unreviewed decision adds mess that compounds — eroding maintainability and trust in the codebase. With spex, the human stays in control: specs and contracts are reviewed and approved before implementation, and tests prove the result. The codebase stays maintainable, and every change traces back to a decision a human approved.

## Description

### The flow from business goal to verified code:

For example, a pizza ordering feature moves through the flow like this:

1. **BRD** — "sell pizza online to increase sales"
2. **PRD** — "customers can pick a pizza on the website and pay online"
3. **Business Spec** — `BIZ-ORDER-PLACE`: "POST /orders returns 201 with the created order"
4. **Contract** — OpenAPI definition of `placeOrder` returning `201`
5. **Implementation** — the `placeOrder()` handler in code
6. **Test** — verifies that placing an order returns `201`

```mermaid
flowchart TD
    BRD["BRD — why"] --> PRD["PRD — what"]
    PRD --> Biz

    subgraph Traced ["traced by spec ID"]
        Biz["Business Spec — how it behaves"] --> Contract["Contract — interface"]
        Contract --> Impl["Implementation — code"]
        Impl --> Test["Test — proof"]
    end
```

### Technical specs live outside the feature flow — they govern how every artifact is built:

For example, the rule `TECH-NO-SECRETS` works like this:

1. **Technical Spec** — `TECH-NO-SECRETS`: "no secrets in the repository"
2. **Governs** — every contract, file, and test in the repo, regardless of feature
3. **Proven by** — a secret scan check passing on every commit in CI

```mermaid
flowchart TD
    Tech["Technical Spec — how to built"] -.governs.-> Flow["Contracts, code, tests"]
    Tech -.proven by.-> Checks["Automated an AI checks"]
```
