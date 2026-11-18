# `spex receipt`

Keeps compact decisions and verification receipts in an explicitly selected journal. The receipt records the inputs and evidence reported by a person or a tool.

## Acceptance criteria

- MUST require an explicit `--journal <directory>` for every operation. Only `append` creates the journal directory.
- MUST accept `append <file>` with a JSON receipt, reading standard input for `append -`. The input is bounded to 64 KiB.
- MUST require schema version `1`, a receipt ID using the spec ID grammar, and a lowercase SHA-256 `context_digest`. A supplied package revision is a full lowercase hexadecimal Git commit ID.
- MUST record each stage separately under `verification`. The required stages are `rules`, `trace_coverage`, `contract_validation`, `executed_tests` and `manual_acceptance`.
- MUST accept stage statuses `passed`, `failed`, `not-run`, `unavailable` and `waived`. Passes, failures and waivers include a reference to their evidence or exception.
- MUST allow a compact decision with its approval state, reason, and reviewer or issue reference, plus lists of exception and unresolved issue references.
- MUST reject unexpected JSON fields and empty or multiline reasons or references. Each reason or reference is at most 512 bytes.
- MUST validate that a supplied parent ID already exists in the same journal. Multiple receipts can name the same parent.
- MUST store each receipt as a separate `<id>.json` file with its append time in Unix seconds. Publish the complete record atomically and preserve an existing record when writers run concurrently.
- MUST return the existing record when its receipt ID and data match an append request. An attempt to append different data under that ID is an error.
- MUST provide `list`, showing the IDs, parents, append times and context digests in receipt ID order, and `show <id>`, showing the stored record. The output is pretty JSON.
- MUST leave receipt creation opt-in. This command records reported evidence references and does not execute the underlying checks or verify those references.

## Examples

```sh
spex receipt --journal .spex-journal append run-result.json
spex receipt --journal .spex-journal list
spex receipt --journal .spex-journal show change-42
```

A receipt supplied to `append` can use this shape.

```json
{
  "schema_version": 1,
  "id": "change-42",
  "context_digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "revisions": {
    "api": "0123456789abcdef0123456789abcdef01234567"
  },
  "decision": {
    "approval": "approved",
    "reason": "Use the existing order endpoint.",
    "reference": "issue/42"
  },
  "verification": {
    "rules": { "status": "passed", "reference": "artifacts/rules.json" },
    "trace_coverage": {
      "status": "passed",
      "reference": "artifacts/coverage.json"
    },
    "contract_validation": { "status": "not-run" },
    "executed_tests": {
      "status": "passed",
      "reference": "artifacts/tests.xml"
    },
    "manual_acceptance": { "status": "not-run" }
  }
}
```
