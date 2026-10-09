# Migration guide

Steps required to move a project from one spex CLI version to a newer one, one section per version that needs them. A version not listed here needs no migration steps.

## 0.12.0

`spex info --print <field>` was removed. In every script under `<workspace-root>/<project-name>-spex/scripts/**` that calls `spex info --print <field>`, replace it with `spex info | jq -r .<field>` (e.g. `spex info --print project_name` becomes `spex info | jq -r .project_name`), and make sure `jq` is installed. Scripts bundled with rules that set `sync_with_upstream: true` are rewritten by `spex update` and need no change.

## 0.10.4

The spex repository skill is now the bundled `spex` skill, which replaces the per-project `<project-name>-spex` skill, and `spex update` doesn't delete files, so the old skill lingers. Delete the folder `<workspace-root>/<project-name>-spex/.agents/skills/<project-name>-spex/`.

## 0.10.3

`docs/cli.md` and `docs/framework-principles.md` are no longer bundled into projects (only `docs/glossary.md` still is), and `spex update` doesn't delete files, so the old copies linger. Delete `<workspace-root>/<project-name>-spex/docs/cli.md` and `<workspace-root>/<project-name>-spex/docs/framework-principles.md`.

## 0.10.2

The spex repository folder icon in the editor workspace file changed from 🧰 to 🦉. In `<workspace-root>/<project-name>-spex/<project-name>.code-workspace`, rename the folder `🧰 spex` (with `path` `.`) to `🦉 spex`.

## 0.10.0

The spec format gained a required `contract` field (`none`, `single`, or `for-each-ac`) that sets the contract coverage granularity. Add `contract: single` to every spec in `<workspace-root>/<project-name>-spex/specs/**/*.yaml` (its current default behavior — one contract annotation per spec), or choose `none` (no contract coverage) or `for-each-ac` (one contract annotation per acceptance criterion) where they fit better.

`coverage_ignore` contract-layer entries changed meaning: a per-criterion entry (`ac` set) is now only valid under `contract: for-each-ac`, and any contract-layer entry is pointless under `contract: none` — `spex check` errors on both mismatches, so remove or fix such entries.

## 0.9.0

The `x-spec` annotation grammar changed: an `x-spec` (or any other `x-*` vendor extension) value that is a bare Spec ID no longer counts toward contract coverage. Rewrite every `x-spec: <spec-id>` into `x-spec: "@spec contract <spec-id>"` (or any other `x-*` key) across `<workspace-root>/<repository-name>/**` contract files.
