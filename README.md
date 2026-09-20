# spex

The AI-first development framework that turns natural specifications into deterministic code — keeping AI running at full speed, with humans safely in the driver’s seat.

_The name `spex` comes from a casual shorthand for specifications._

## Goal: unleash AI's full speed without slop

AI operates fast and getting even faster — the framework's job is unleash it speed while ensuring slop-free results.

## Core Concepts

spex is built on four simple pieces:

- **Specs** define _what_ a feature does — a capability with acceptance criteria.
- **Rules** govern _how_ everything is built — cross-cutting conventions like style, security, or testing that apply repo-wide.
- **Skills** are the instructions an AI agent follows to do the job — taking all applicable rules and implementing a spec.
- **Traceability** is the thread that stitches together spec, contract, implementations, and tests — ensuring full coverage and tracing for spec and every acceptance criterion.

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

### Typical workflow

Say you're want to add pizza ordering functionality to the backend. Here's what you will do:

1. **Create spec** `/spex-spec backend place pizza order endpoint`. Will create `pepperoni-pizza-spex/specs/place-pizza-order.yaml`
2. **Derive the contract** for a spec `/spex-contract place-pizza-order`. Will add the endpoint to `pepperoni-pizza-spex/contracts/api.openapi.yaml`.
3. **Implement it** `/spex-implement place-pizza-order`. Will implement a spec as writen following applicable rules and skill instructions.
4. **Review** review implementation with `/spex-review place-pizza-order` to ensure conformance with spec and applicable rules.
5. **Add tests** `/spex-test place-pizza-order`. Will add tests coverage for the implemented spec.

Tip: you can combine skills `/spex-spec /spex-contract` or run `spex-assembly-line` to implement, review and fix in loop and test.

Say you're want to add rule. Here's what you will do:

**Create a rule**: `/spex-rule secret handling`. Will create `pepperoni-pizza-spex/rules/secret-handling.yaml` with scripts to verify it.

## Framework Principles

- **AI over Manual Coding**: used right, AI writes code faster than a developer while keeping quality high.
- **Specs over Guesswork**: clear, well-defined specs eliminate guesswork, giving explicit acceptance criteria for both AI and developer.
- **Rules over Tribal Knowledge**: cross-cutting conventions are captured as rules instead of living as tribal knowledge.
- **Human in control**: AI writes everything — specs, contracts, plans, code, and tests — and the human validates everything.
- **Deterministic enforcement**: every check that can be a script must be a script, proving the change does what the spec demands.
- **Traceability**: every behavioral change traces to a spec, bidirectionally — spec -> contract -> code -> test, and back.

See full principles in [framework-principles.md](docs/framework-principles.md).

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
5. **Implementation** — the `placeOrder()` handler in code
6. **Test** — verifies that placing an order returns `201`

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
3. **Proven by** — a secret scan check running in CI or manually with `spex check`

Even if a deterministic check is not possible, rules are read and followed by AI and can be used for a review by a human or another agent, which gives pretty good results.

```mermaid
flowchart TD
    Eng["Rule — how it's built"] -.governs.-> Flow["Contracts, code, tests"]
    Eng -.proven by.-> Checks["Automated checks, human and agent review"]
```

## Learn more

Framework [specification](docs/specification.md).

Example [project](examples/pepperoni-pizza/README.md) with spex implemented.

## Try tutorial

Ready to try it yourself? Follow the [Tutorial](docs/tutorial.md) to take the
`pepperoni-pizza` example from a plain multi-repo project to a fully spec-driven
workspace, end to end.
