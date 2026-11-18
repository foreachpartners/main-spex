# `spex context`

Assembles the governing context for a task from explicit spec IDs and packages. The same JSON can be inspected by a reviewer, passed to an agent, or queried with `spex jq`.

## Acceptance criteria

- MUST select the specs named in `--id <id>...` and recursively include their `blocked_by` references once each.
- MUST reject an unknown spec ID, a missing dependency, or an ambiguous spec ID. The diagnostic names the ID involved.
- MUST exclude deprecated specs from task context. When a selected spec or dependency is deprecated, require explicit `--include-deprecated` to include it.
- MUST include each selected spec's description, lifecycle status, file path, acceptance criteria with source lines, and contract level.
- MUST include the packages declared by the selected specs and add any packages named in `--package <name>...`, accepting a package's full name or short name.
- MUST include each selected package's description, path, current Git commit and dirty state when known, applicable rules, and declared contracts. For each contract include its configured path, description and SHA-256 of the file. Read the file to compute its digest and keep its body out of the bundle.
- MUST reject an unknown selected package or rule, and identify a selected package directory or contract file that is missing.
- MUST include the project name and short description, the spec repository's current Git commit and dirty state when known, and the configured PRD path. The task bundle refers to the PRD by path.
- MUST show why each spec, package and rule is included, using explicit selections, dependent specs, or governing packages as applicable.
- MUST emit pretty JSON containing `schema_version`, `context_digest` and `context`. The digest is the lowercase hexadecimal SHA-256 of the compact JSON serialization of `context`.
- MUST sort selected specs, packages, rules and selection reasons by identifier. The bundle uses paths relative to the spex directory wherever the configuration uses relative paths. Identical effective inputs yield identical output regardless of checkout location or argument order.
- MUST keep the command read-only and retrieve historical receipts only on an explicit `spex receipt` invocation.

## Examples

```sh
spex context --id add-list-item
spex context --id add-list-item list-items --package api frontend
spex context --id add-list-item | spex jq -r .context_digest
```
