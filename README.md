# spex

AI-first specification-driven development framework with human in control.

_The name `spex` comes from a casual shorthand for specifications._

## Prerequisites

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/) (required for building the spex CLI)
- **Git**: [Install Git](https://git-scm.com/install/)
- **spex CLI**: Clone spex repository (repo) and install spex cli tool:
  ```sh
  git clone https://github.com/ziimakc/spex
  cd spex
  cargo install --path . --force
  ```

## Quickstart

### Setup

We will create the following structure:

```
pepperoni-pizza/         # workspace root
├── api/                 # existing api repository
├── frontend/            # existing frontend repository
├── ...                  # repeat for other repositories
└── spex/                # new spex repository (created by cli)
    └── pepperoni-pizza.code-workspace  # editor workspace file
```

1. Put all project git repos in one shared folder (workspace root folder)
2. Run `spex init` in workspace root to create `pepperoni-pizza/spex` repo
3. Open workspace to work across all repositories at once:
   - Cursor/VSCode: open workspace from file `spex/pepperoni-pizza.code-workspace`
   - Claude CLI: run `claude --add-dir ../api ../contracts ../frontend ..` from `spex` repo
   - Zed: add project, pick all workspace repos (folders)
4. Run your first agent skill (skill) `/spex-setup` to complete setup

### Workflow

Workflow core is AI agent skills and CLI commands.

1. Create your first specification (spec):
   - business spec `/spex-create-biz-spec [backend place pizza order endpoint]`. Will create `spex/biz-specs/BIZ-PLACE-PIZZA-ORDER.yaml`
   - technical spec `/spex-create-tech-spec [secret handling]`. Will create `spex/tech-specs/TECH-SECRET-HANDLING.yaml`
2. Create implementation plan for spec `/spex-create-plan [secret handling]`. Will create `spex/plans/TECH-SECRET-HANDLING/2024-11-25_secret-handling.md`
3. Execute plan `/spex-execute-plan [secret handling]`. Will execute the `TECH-SECRET-HANDLING` plan using multiple agents working in parallel per step.
4. Check result with `spex check`. Will run all TECH spec checks related to your changes.

## Why

**AI over Manual Coding**: Used right, AI writes code faster than a developer while keeping quality high. It's cheap — it can even run locally for the price of electricity — and it keeps getting faster and better over time. Developers who insist on writing every line by hand will simply lose the competition.

**Specs over Guesswork**: Casual requests force an AI to guess, leading to inaccurate repository navigation and assumption-based task completion. If I ask an AI to "make me a snake game", it will — but it will be the snake game the AI imagined, not the one I wanted, because I wanted blue snake with big eyes! Clear, well-defined business specifications (`BIZ`) eliminate guesswork, providing explicit acceptance criteria for both the AI to follow and the developer to verify. Specs will stay relevant for a long time even if AI becomes much better and faster, because you need to describe an idea from your head that will meet your or your customer's expectations, not someone else's.

**Technical rules and patterns as Specs**: Cross-cutting technical rules and patterns — code style, structure, testing, security — are captured as technical specs (`TECH`) instead of living as tribal knowledge to ensure project maintainability over the long term. They apply repo-wide and are proven by automated checks (linters, CI gates) or AI, so every change follows the same rules no matter who — human or AI model — writes the code.

**Human in control**: Vibe coding ships working code fast, but every unreviewed decision adds mess that compounds — eroding maintainability and trust in the codebase. With spex, the human stays in control: specs and contracts are reviewed and approved before implementation, and tests prove the result. The codebase stays maintainable, and every change traces back to a decision a human approved.

## Framework Principles

**Simplicity**: The tool should help the developer get work done, not stand in the way. Everything about spex is kept simple, so it can be adopted and put to use in five minutes.

**Tool-agnostic**: To avoid lock-in and restrictions, the framework works with any IDE, terminal, or AI model.

**Ubiquitous language**: A shared vocabulary connects business, specs, and code so humans and AI mean the same thing by the same term. spex defines its terms in the [glossary](docs/glossary.md) and uses them consistently across specs, contracts, code, and docs, removing ambiguity and the translation errors it causes.

## Description

### The flow from business goal to verified code:

For example, a pizza ordering feature moves through the flow like this:

1. **BRD** — "sell pizza online to increase sales"
2. **PRD** — "customers can pick a pizza on the website and pay online"
3. **Business Spec** — `BIZ-PLACE-PIZZA-ORDER`: "POST /orders returns 201 with the created order"
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

For example, the rule `TECH-SECRET-HANDLING` works like this:

1. **Technical Spec** — `TECH-SECRET-HANDLING`: "Secret Protection"
2. **Governs** — every contract, file, and test in the repo, regardless of feature
3. **Proven by** — a secret scan check passing on every commit in CI

```mermaid
flowchart TD
    Tech["Technical Spec — how it's built"] -.governs.-> Flow["Contracts, code, tests"]
    Tech -.proven by.-> Checks["Automated and AI checks"]
```

## Learn more

For an in-depth walkthrough of concepts and the full workflow, see the [Guide](docs/guide.md).
