# `spex-test` skill

Adds or updates tests for an already-implemented change, from a plan, a diff, or an ad-hoc request, and finalizes spec completion. Loaded after `spex-implement`, or whenever the user wants test coverage added for existing code.

## Acceptance criteria

- MUST follow each process step without skipping, and only write tests and finalize spec completion.
- MUST NOT edit plan files, a spec's acceptance criteria, or non-test code.
- MUST gather context before writing tests: the plan, or without one the packages, governing specs, and rules read with `spex rules --package ... --ai`, reading all specs and rules and inspecting the actual implementation.
- MUST run `spex coverage` and add tests for every acceptance criterion with an `impl` annotation but no `test` annotation, plus any other tests the implementation needs.
- MUST annotate new tests with `@spec test <spec-id>#<ac-id>` where the governing rules require test-layer traceability, on the smallest unit that proves the criterion.
- MUST test against the specification, fixing a small implementation gap or reporting a bigger one, and MUST NOT fit a test to the implementation.
- MUST clear `is_done: false` for each criterion `spex coverage` shows fully impl+test traced, set `status: active` once every criterion is done, and touch only those lines.
- MUST verify by running formatters, linters, the test suite, and `spex check --package ...`.
- MUST report the units completed, the specs and criteria covered, the verification outcome, and which specs were set to `active`.
