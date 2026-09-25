# Migration guide

Steps required to move a project from one spex CLI version to a newer one, one section per version that needs them. A version not listed here needs no migration steps.

## 0.10.0

The spec format gained a required `contract` field (`none`, `single`, or `for-each-ac`) that sets the contract coverage granularity. Add `contract: single` to every spec in `<workspace-root>/<project-name>-spex/specs/**/*.yaml` (its current default behavior — one contract annotation per spec), or choose `none` (no contract coverage) or `for-each-ac` (one contract annotation per acceptance criterion) where they fit better.

`coverage_ignore` contract-layer entries changed meaning: a per-criterion entry (`ac` set) is now only valid under `contract: for-each-ac`, and any contract-layer entry is pointless under `contract: none` — `spex check` errors on both mismatches, so remove or fix such entries.

## 0.9.0

The `x-spec` annotation grammar changed: an `x-spec` (or any other `x-*` vendor extension) value that is a bare Spec ID no longer counts toward contract coverage. Rewrite every `x-spec: <spec-id>` into `x-spec: "@spec contract <spec-id>"` (or any other `x-*` key) across `<workspace-root>/<repository-name>/**` contract files.
