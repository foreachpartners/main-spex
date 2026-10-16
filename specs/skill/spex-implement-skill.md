# `spex-implement` skill

Carries out an existing plan, or an ad-hoc user request. Loaded when the user wants to carry out a plan from `<spex-dir>/plans/<plan-name>`, or asks for a change without a formal plan.

## Acceptance criteria

- MUST follow each process step without skipping.
- MUST gather context before editing code: read the plan, or without one determine the packages to work on, read the governing rules with `spex rules --package ... --ai`, determine the governing specs, read all of them, and note the governing rules and specs.
- MUST, with a plan, resume at the first incomplete step and edit its `Status` in place using only `not-started`, `in-progress`, and `completed`.
- MUST, without a plan, split the request into tasks only when it spans independent packages or clearly separable files, and run every task in a unit in parallel.
- MUST annotate each acceptance criterion it implements with `@spec impl <spec-id>#<ac-id>` (and the contract annotation where required), on the smallest unit that satisfies it, keeping annotations minimal and never on a file, type, or module.
- MUST NOT write tests, and MUST stop and ask the user when a task fails rather than pressing on with dependent work.
- MUST set each implemented spec's `status` to `in-progress` when it is absent or `planned`, touching only that line.
- MUST verify by running formatters, linters, the test suite, `spex check --package ...`, and `spex coverage`, fixing every gap except a missing test for a just-annotated criterion.
- MUST delete the plan once every step is `completed`, leaving it in place for a partial run, and delete nothing without a plan.
- MUST update each touched package's skill documentation to match the code, without duplicating specs or rules.
- MUST report the units completed, the specs set to `in-progress`, the verification outcome, whether the plan was deleted, and the package skill docs updated.
- MUST offer a review with `spex-review`, then test coverage with `spex-test`.
