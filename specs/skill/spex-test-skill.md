# `spex-test` skill

Adds or updates tests for an already-implemented change, from a plan, a diff, or an ad-hoc request, and finalizes spec completion. Loaded after `spex-implement`, or whenever the user wants test coverage added for existing code.

## Acceptance criteria

### Process

- MUST follow each process step without skipping.
- MUST only write tests and finalize spec completion.
- MUST NOT edit plan files, a spec's acceptance criteria, or non-test code: implementation is `spex-implement`'s job.

### Step 1: gather context

- MUST finish this step before writing any test file.
- MUST, with a plan, read the plan and run `spex specs --ai --id <spec-id-a> <spec-id-b>` for the plan's `Specs` to read the governing specs.
- MUST, without a plan (ad-hoc request, or testing an existing diff), determine the packages to work on (e.g. frontend, backend), run `spex rules --package <package-a> <package-b> --ai --output <tmp-path>` for those packages and read `<tmp-path>` for the governing rules, determine all governing spec IDs related to the change, from the request or from `@spec` annotations already in the changed code, and run `spex specs --ai --id ...` for them.
- MUST, without a plan, produce a note in its response with a list of the governing rules and specs.
- MUST read all the specs and the governing rules.
- MUST inspect the actual implementation under test.
- MUST NOT test from the plan or spec text alone.

### Step 2: add tests

- MUST run `spex coverage` for each governing spec and add tests for every acceptance criterion that has an `impl` annotation but is missing its `test` annotation.
- MUST add any other tests the implementation needs based on package testing patterns.
- MUST annotate new tests with their spec ID and the specific acceptance criterion they prove, `@spec test <spec-id>#<ac-id>`, where the governing rules require test-layer traceability.
- MUST place each annotation on the smallest unit that proves the criterion, the specific test case (`it`/`test` block), keeping the total number of annotations minimal.
- MUST split into tasks only when the work spans independent packages or clearly separable files; most requests are a single task, and within a unit every task runs in parallel, one AI agent per task.
- MUST stop, report what failed, and ask the user how to proceed when a task fails or an agent cannot complete it, rather than pressing on with dependent work.
- MUST write tests against the specification: when a test fails to meet a spec acceptance criterion, fix the implementation if the change is small, or report to the user if the problem is bigger.
- MUST NOT fit a test to the implementation.
- MUST NOT annotate the file or the suite's top-level `describe`.

### Step 3: finalize the specs

- MUST run `spex coverage` for every governing spec and open `<spex-dir>/specs/<spec-id>.yaml`.
- MUST clear `is_done: false` (remove the line; absent counts as done) for each acceptance criterion that `spex coverage` now shows fully impl+test traced.
- MUST set `status: active` once every criterion in the spec is done (`is_done` `true` or absent).
- MUST touch only `is_done` lines and `status`, leaving the rest of each spec, including acceptance criteria text and ids, untouched.

### Step 4: verify

- MUST run formatters, linters, and the test suite to confirm the new tests pass and nothing else broke.
- MUST prove the new tests respect the governing rules with `spex check --package <package-a> <package-b>`.

### Step 5: report

- MUST summarize which units of work completed.
- MUST summarize which specs and acceptance criteria it added coverage for.
- MUST summarize the format, lints, tests, and `spex check` run outcome.
- MUST summarize which specs it set to `active`, and which acceptance criteria still show `is_done: false` because `spex coverage` does not yet show them traced.
