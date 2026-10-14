# Traceability

Every behavioral change traces to a spec, and one Spec ID traces across the packages that satisfy it. The principle, including the bidirectional trace and commit-level provenance, is [prd.md](../../docs/prd.md#traceability-over-assumption); the annotation grammar that carries it is [spec-annotations-spec.md](../spec/spec-annotations-spec.md).

## Acceptance criteria

- MUST trace every behavioral change to a spec, down to the acceptance criterion it satisfies.
- MUST trace one Spec ID across every package that satisfies it.
- MUST trace in both directions: top-down (spec -> contract -> implementation -> test) and bottom-up (test -> implementation -> contract -> spec).
- MUST carry the trace in `@spec` annotations that follow [spec-annotations-spec.md](../spec/spec-annotations-spec.md).
- MUST show, for a checked-out commit, which specs and acceptance criteria the code was built against.
- MUST NOT leave an orphan: an annotation with no spec, or an acceptance criterion with no trace to its required contract, implementation, or test.
