# Artifact chain

Turns business goals into verified code through a chain of artifacts, each traceable by a shared Spec ID: `PRD` (why and what) -> spec (capability) -> contract (interface, if applicable) -> implementation (code) -> test (proof). [glossary.md](../../docs/glossary.md) defines every artifact in the chain and its lifecycle.

## Acceptance criteria

- MUST order the artifacts as PRD, spec, contract (if applicable), implementation, test.
- MUST require one PRD per project, the short overview of what is built and why, without details, for people and AI agents to write specs that fit the project's purpose.
- MUST write each spec in line with the PRD, extracted from it or from a feature idea by the product owner.
- MUST derive a contract from a spec when applicable, it's cheap to create, review, and reason about first.
- MAY have a plan, an ordered set of steps turning approved specs and their contracts into implementation instructions.
