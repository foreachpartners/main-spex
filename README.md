# spex

AI-first specification-driven development framework with human validation.

_The name `spex` comes from a casual shorthand for specifications._

## Why

**Specs over Guesswork**: Casual requests force an AI to guess, leading to inaccurate repository navigation and assumption-based task completion. Clear, well-defined functional specifications (`FUNC`) eliminate guesswork, providing explicit acceptance criteria for both the AI to follow and the developer to verify.

**Architecture as Specs**: Cross-cutting technical rules — code style, structure, testing, security — are captured as architecture specs (`ARCH`) instead of living as tribal knowledge. They apply repo-wide and are proven by automated checks (linters, CI gates) or AI, so every change follows the same rules no matter who — human or AI model — writes the code.

**Human validation**: Vibe coding ships working code fast, but every unreviewed decision adds mess that compounds — eroding maintainability and trust in the codebase. With spex, the human stays in control: specs and contracts are reviewed and approved before implementation, and tests prove the result. The codebase stays maintainable, and every change traces back to a decision a human approved.

## Descripion

### The flow from business goal to verified code:

For example, a pizza ordering feature moves through the flow like this:

1. **BRD** — "sell pizza online to increase sales"
2. **PRD** — "customers can pick a pizza on the website and pay online"
3. **Functional Spec** — `FUNC-ORDER-PLACE`: "POST /orders returns 201 with the created order"
4. **Contract** — OpenAPI definition of `placeOrder` returning `201`
5. **Implementation** — the `placeOrder()` handler in code
6. **Test** — verifies that placing an order returns `201`

```mermaid
flowchart TD
    BRD["BRD — why"] --> PRD["PRD — what"]
    PRD --> Func

    subgraph Traced ["traced by spec ID"]
        Func["Functional Spec — how it behaves"] --> Contract["Contract — interface"]
        Contract --> Impl["Implementation — code"]
        Impl --> Test["Test — proof"]
    end
```

### Architecture specs live outside the feature flow — they govern how every artifact is built:

For example, the rule `ARCH-NO-SECRETS` works like this:

1. **Architecture Spec** — `ARCH-NO-SECRETS`: "no secrets in the repository"
2. **Governs** — every contract, file, and test in the repo, regardless of feature
3. **Proven by** — a secret scan check passing on every commit in CI

```mermaid
flowchart TD
    Arch["Architecture Spec — how to built"] -.governs.-> Flow["Contracts, code, tests"]
    Arch -.proven by.-> Checks["Automated an AI checks"]
```
