# Bundled `testing-rules` rule

The bundled [`testing-rules`](../../templates/rules/testing-rules.yaml) rule keeps every test running and able to fail. The rule file masters its criteria; this spec covers what spex ships and checks for it.

## Acceptance criteria

- MUST ship the rule at `templates/rules/testing-rules.yaml`, deployed by `spex init` and `spex update`.
- MUST report a `warn` finding for `#[ignore]`, `@pytest.mark.skip`, `@unittest.skip`, or `t.Skip(` in a test file: a file named `*_test.*`, `*.test.*`, or `*.spec.*`, or a file under a `tests/` or `test/` directory.
- MUST report a `warn` finding for an `xit(...)` or `xdescribe(...)` call, or a `.skip(...)` call on `describe`, `it`, `test`, or `context`, in a `.ts`, `.tsx`, `.js`, or `.jsx` test file.
- MUST report an `error` finding for a `fit(...)` or `fdescribe(...)` call, or an `.only(...)` call on `describe`, `it`, `test`, or `context`, in a `.ts`, `.tsx`, `.js`, or `.jsx` test file.
- MUST NOT report a `.skip(...)` or `.only(...)` call on any object other than `describe`, `it`, `test`, or `context`.
