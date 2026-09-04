# spex

AI-first specification-driven development framework with human in control.

_The name `spex` comes from a casual shorthand for specifications._

## Goal

spex helps you move fast with AI, not stand in your way. You just formulate the task and review the result. Specs, rules, and skill instructions give AI the context it needs to implement it right, quickly.

## Core Concepts

spex is built on three simple pieces:

- **Specs** define _what_ a feature does — a capability with acceptance criteria.
- **Rules** govern _how_ everything is built — cross-cutting conventions like style, security, or testing that apply repo-wide.
- **Skills** are the instructions an AI agent follows to do the job — taking all applicable rules and implementing a spec.

## Quickstart

### Prerequisites

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/) (required for building the spex CLI)
- **Git**: [Install Git](https://git-scm.com/install/)
- **spex CLI**: Clone spex repository (repo) and install spex cli tool:
  ```sh
  git clone https://github.com/foreachpartners/spex
  cd spex
  cargo install --path . --force
  ```

#### Setup

We will create the following structure:

```
pepperoni-pizza/                       # workspace root
├── pepperoni-pizza-api/               # existing api repository
├── pepperoni-pizza-frontend/          # existing frontend repository
├── ...                                # repeat for other repositories
└── pepperoni-pizza-spex/              # new spex repository (created by spex cli)
```

Repository directories follow `<project-name>-<repository-short-name>` (naming-convention), so rename existing repositories to match before running `spex init`.

1. Put all project git repos in one shared folder (workspace root folder)
2. Run terminal command `spex init` in workspace root to create the `pepperoni-pizza-spex` repo
3. Open workspace to work across all repositories at once and integrate with AI coding agent:
   - Cursor/VSCode: open workspace from file `pepperoni-pizza-spex/pepperoni-pizza.code-workspace`
   - Cursor agentic: open workspace file in editor mode first, then pick it in agentic mode.
   - Claude CLI: run `claude --add-dir ../pepperoni-pizza-api ../pepperoni-pizza-frontend ..` from `pepperoni-pizza-spex` repo
   - Zed: add project, pick all workspace repos (folders)
4. Run your first agent skill (skill) `/spex-setup` in the AI chat window to complete setup. Skills will be detected by the AI coding agent (agent) if the setup is done right.

### Adoption

After setup is completed you get basic skills and rules which you can customize for your project or leave them as is to be synced from upstream.

For existing project you can generate all specs based on existing code or gradually add them based on features you are working on. For example if you want to change user nickname registration length, ask AI to generate spec based on how user registration currently works, then do your changes.

### Workflow

Workflow core is AI agent skills and CLI commands.

1. **Create spec** `/spex-spec backend place pizza order endpoint`. Will create `pepperoni-pizza-spex/specs/place-pizza-order.yaml`
2. **Derive the contract** for a spec `/spex-contract place-pizza-order`. Will add the endpoint to `pepperoni-pizza-spex/contracts/api.openapi.yaml`.
3. **Create an implementation plan** for a spec `/spex-plan place-pizza-order`. Will create `pepperoni-pizza-spex/plans/place-pizza-order/2024-11-25-place-pizza-order.md`
4. **Implement the plan** `/spex-implement place-pizza-order`. Will execute the plan using multiple agents working in parallel per step and delete after completion.
5. **Review** self review with `/spex-review place-pizza-order` to ensure conformance with applicable rules, `spec check` and implemented spec.
6. **Add tests** `/spex-test place-pizza-order`. Will add test coverage for the implemented acceptance criteria.

Tip: you can combine skills `/spex-spec /spex-contract` or run implementation/review and test loop `spex-assembly-line`

**Create a rule**: `/spex-rule secret handling`. Will create `pepperoni-pizza-spex/rules/secret-handling.yaml`

## Framework Principles

**AI over Manual Coding**: Used right, AI writes code faster than a developer while keeping quality high. It's cheap — it can even run locally for the price of electricity — and it keeps getting faster and better over time. Developers who insist on writing every line by hand will simply lose the competition.

**Specs over Guesswork**: Casual requests force an AI to guess, leading to inaccurate repository navigation and assumption-based task completion. If I ask an AI to "make me a snake game", it will — but it will be the snake game the AI imagined, not the one I wanted, because I wanted a blue snake with big eyes! Clear, well-defined specs eliminate guesswork, providing explicit acceptance criteria for both the AI to follow and the developer to verify. Specs will stay relevant for a long time even if AI becomes much better and faster, because you need to describe an idea from your head that will meet your expectations, not someone else's.

**Rules over Tribal Knowledge**: Cross-cutting technical rules and patterns — code style, structure, testing, security — are captured as rules instead of living as tribal knowledge to ensure project maintainability over the long term. They apply repo-wide and are proven by automated checks (linters, CI gates) or AI, so every change follows the same rules no matter who — human or AI model — writes the code. A one-off technical implementation — a specific endpoint, integration, or mechanism — is a spec, not a rule.

**Skills as Instructions**: Ad hoc prompting reproduces the guesswork specs exist to eliminate — every request re-explains context that already lives in the repository, and the outcome depends on what the developer remembers to say to the agent that day. A skill writes that context and workflow down once, so any developer or any session gets the same instructions instead of reconstructing them from memory. Skills, not chat, are how instructions reach the agent.

**Human in control**: Vibe coding ships working code fast, but every unreviewed decision adds mess that compounds — eroding maintainability, trust, and the team's comprehension of the codebase. With spex, the AI writes everything — specs, contracts, plans, code, and tests — and the human validates everything. The codebase stays maintainable, and every change traces back to a decision a human approved.

**Deterministic enforcement**: AI writes code fast, but along the way it may change things nobody asked it to and invent acceptance criteria that no spec actually contains. Human review alone can't catch all of it. Deterministic checks (tests, rule checks, linting, schema validation, coverage) are what prove the change does what the spec demands and that nothing else broke. They give the same answer every time, so they catch what both the AI and the human overlook. That's why every check that can be a script must be a script.

See all principles in [framework-principles.md](docs/framework-principles.md).

## Description

### The flow from business goal to verified code:

For example, a pizza ordering feature moves through the flow like this:

1. **BRD** — sell pizza online to increase sales
2. **PRD** — customers can pick a pizza on the website and pay online
3. **Spec** — `place-pizza-order`: Customer can place an order.

   Acceptance criteria:
   - MUST return 201 from POST /orders with the created order
   - MUST let the customer place the order from the order page

4. **Contract** — OpenAPI definition of `placeOrder` returning `201`, plus a Figma reference for the order page
5. **Plan** — step-by-step build plan for the `placeOrder` endpoint
6. **Implementation** — the `placeOrder()` handler in code
7. **Test** — verifies that placing an order returns `201`

```mermaid
flowchart TD
    BRD["BRD (optional) — why"] --> PRD["PRD (optional) — what"]
    PRD --> Func

    subgraph Traced ["traced by spec ID"]
        Func["Spec — what it provides"] --> Contract["Contract (if applicable) — interface"]
        Contract --> Plan["Plan (optional) — how to build it"]
        Plan --> Impl["Implementation — code"]
        Impl --> Test["Test — proof"]
    end
```

### Rules live outside the feature flow — they govern how every artifact is built:

For example, the rule `secret-handling` works like this:

1. **Rule** — `secret-handling`: "Secret protection"
2. **Governs** — every contract, file, and test in the repo, regardless of feature
3. **Proven by** — a secret scan check passing on every commit in CI, review by human or another agent

Even if a deterministic check is not possible, rules are read and followed by AI and can be used for a review by a human or another agent, which gives pretty good results.

```mermaid
flowchart TD
    Eng["Rule — how it's built"] -.governs.-> Flow["Contracts, code, tests"]
    Eng -.proven by.-> Checks["Automated checks, human or another agent review"]
```

## Learn more

Framework [specification](docs/specification.md).

Example [project](examples/pepperoni-pizza/README.md) with spex implemented.

## Try tutorial

Ready to try it yourself? Follow the [Tutorial](docs/tutorial.md) to take the
`pepperoni-pizza` example from a plain multi-repo project to a fully spec-driven
workspace, end to end.
