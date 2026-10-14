# Coverage states

`spex coverage` classifies each acceptance criterion of an `active` or `in-progress` spec from its annotations and its `is_done` flag. A required layer is the criterion's impl and test, plus its contract when the spec's `contract` is `for-each-ac`, minus whatever `coverage_ignore` waives.

## Acceptance criteria

- MUST classify a criterion as `covered` when every required layer is annotated.
- MUST classify a criterion as `gap` when a required layer is missing and `is_done` is `true` or absent.
- MUST classify a criterion as `gap` when a required layer is missing, `is_done` is `false`, and the impl is annotated.
- MUST classify a criterion as `pending` when a required layer is missing, `is_done` is `false`, and there is no impl annotation.
- MUST treat a gap as an error that makes `spex coverage` exit non-zero, whatever the spec's status.
- MUST use the impl annotation as the trigger, not the test or the contract, so contract-first and test-first work stays pending until the impl lands.
- MUST keep a criterion whose impl is waived by `coverage_ignore` pending until `is_done` is cleared, since there is normally no impl annotation to trigger on.
- MUST treat, under `contract: single`, the spec-level contract as one more requirement of the spec: a gap once any criterion is `is_done` `true` or absent or has an impl annotation, pending while every criterion is `is_done: false` with no impl annotation.
- MUST NOT fail `spex coverage` on pending: pending is work not yet started.
- MUST NOT let `is_done: false` excuse a criterion that already has an impl annotation, so a change is verified in the package that makes it, without first updating the spec in another package.
- MUST NOT classify a spec in any other status (`planned`, `blocked`, `deprecated`), however much of it is annotated.
- SHOULD use `is_done: false` mostly as a marker: `spex check` still reads it for the spec's `status`, and coverage uses it only to keep a criterion pending until it has an impl annotation.
