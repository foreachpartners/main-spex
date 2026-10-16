# Artifact chain

Turns business goals into verified code through a chain of artifacts, each traceable by a shared Spec ID: `PRD` (why and what, optional) -> spec (capability) -> contract (interface, if applicable) -> implementation (code) -> test (proof). [glossary.md](../../docs/glossary.md) defines every artifact in the chain and its lifecycle.

## Acceptance criteria

- MUST order the artifacts as PRD (optional), spec, contract (if applicable), implementation, test.
- MUST derive each spec from the PRD, or from a feature idea directly when there is no PRD; the product owner extracts it.
- MUST derive a contract from a spec when applicable,  it's cheap to create, review, and reason about first.
- MAY skip the plan for a small, self-contained change; a plan is an ordered set of steps turning approved specs and their contracts into implementation instructions.
