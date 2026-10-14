# Spec ID annotations

Annotations link a contract, implementation, or test to its spec, and optionally to one specific acceptance criterion.

## Acceptance criteria

- MUST write an annotation as `@spec contract|impl|test <spec-id>[#<ac-id>][, <spec-id>[#<ac-id>]]...`.
- MUST match an annotation anywhere in a file's text: as a code comment, or, for contracts, as the string value of any OpenAPI vendor extension (`x-*`) property, whose key name is not itself checked.
- MUST place one annotation on the smallest unit that actually satisfies the acceptance criterion: a specific field, function, or test case.
- MUST annotate once at the boundary, never nested: list every acceptance criterion id the boundary satisfies rather than adding a second annotation.
- MUST carry the `#<ac-id>` of the acceptance criterion it satisfies on an impl or test annotation.
- MUST carry `#<ac-id>` on a contract annotation only under `contract: for-each-ac`, where every acceptance criterion needs its own annotated contract.
- MUST accept, under `contract: single`, one bare `@spec contract <spec-id>` for the whole spec.
- MUST count a criterion-anchored contract annotation also as the spec-level contract.
- MUST make `<spec-id>` and `<ac-id>` follow the same id grammar as the ids they reference; an annotation naming an id outside it is malformed.
- MUST have `spex check` enforce the grammar and the no-orphan rule, built in, in every package regardless of which rules it opts into ([spex-check-cli-cmd.md](../cli/spex-check-cli-cmd.md)).
- MUST NOT place an annotation on the file, the test suite's top-level describe, or the enclosing type.
- MUST NOT widen the placement beyond the smallest unit unless the criterion's scope genuinely spans that boundary (e.g. an endpoint, contract).
- MUST NOT count a bare, unqualified annotation toward a criterion of a spec in `spex coverage`.
- MUST NOT reference a Spec ID with no spec file.
