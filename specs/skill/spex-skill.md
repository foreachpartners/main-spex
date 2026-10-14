# `spex` skill

The framework context skill: the paths, the artifacts, and the commands of a spex project.

## Acceptance criteria

- MUST be loaded before reading, writing, testing, or running any code in a spex project.
- MUST tell the agent to run `spex info` for the project configuration; its `spex_dir` is the directory the skill describes, holding the specs and config that govern the whole project.
- MUST describe `spex.yaml` as the project and each package's configuration: description, paths, contracts, and applicable rules.
- MUST describe `specs/` as the capabilities the system must provide, with acceptance criteria, and as the single source of truth for the project: all behaviour, contracts, code, tests and other artifacts come from it and adhere to it.
- MUST describe `rules/` as cross-cutting conventions with acceptance criteria, which each package adheres to as listed for it in `spex.yaml`, and a rule's `script` checks as keeping their scripts in a folder named after the rule beside its file (`rules/<rule-id>.yaml` and `rules/<rule-id>/`).
- MUST describe `contracts/` as the interface definitions (OpenAPI, `.proto`, GraphQL SDL, AsyncAPI) shared across packages; a contract file's location is user-owned, most commonly inside the owning package, and recorded in `spex.yaml`, and the spex directory's `contracts/` folder is only a convention for contracts genuinely shared across packages.
- MUST describe `plans/` as the temporary implementation plans derived from an approved spec and contract, to plan and track a large task split in multiple steps, deleted after completion.
