# Migration guide

Steps required to move a project from one spex CLI version to a newer one, one section per version that needs them. A version not listed here needs no migration steps.

## 0.9.0

The `x-spec` annotation grammar changed: an `x-spec` (or any other `x-*` vendor extension) value that is a bare Spec ID no longer counts toward contract coverage. Rewrite every `x-spec: <spec-id>` into `x-spec: "@spec contract <spec-id>"` (or any other `x-*` key) across `<workspace-root>/<repository-name>/**` contract files.
