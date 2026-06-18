# Guide

In-depth guide to working with spex, picking up where the [README](../README.md) quickstart leaves off.

## Updating spex

After upgrading the `spex` binary, run `spex update` to pull the latest framework files into your spex repository:

```sh
spex update
```

See the [CLI reference](cli.md) for the full command set.

## Per-repository checks

Every workspace repository inherits the checks declared by the tech specs in its `tech-specs`. A repository can adjust those inherited checks, or add new ones, by declaring them in `spex.yml`. A check's identity is `<spec>/<name>`, so every per-repo entry names both the `spec` and the `name`.

A `script` check's `script:` value is just the filename; spex resolves it at `scripts/<spec-id>/<script>` under the relevant base directory. The `<spec-id>` folder namespaces scripts so checks from different specs never share a path. Shared scripts live in the spex repository's `scripts/`; a repository's own scripts live in that repository's `scripts/`.

### `checks_override` — patch an inherited check

Name the inherited check by `spec` + `name`, then set only the fields you want to change. Every other field is inherited. The merged check runs instead of the inherited one.

```yaml
repositories:
  - repository_name: frontend
    path: ../frontend
    tech-specs: [TECH-WRITING-STYLE]
    checks_override:
      - spec: TECH-WRITING-STYLE # only the severity changes
        name: no-cyrillic
        severity: warn
      - spec: TECH-WRITING-STYLE # use a repo-specific script instead of the shared one
        name: llm-writing
        script: writing-style.sh
      - spec: TECH-WRITING-STYLE # turn the inherited check off entirely
        name: no-emoji
        disable: true
```

When an override sets `script`, it resolves under the repository directory (`frontend/scripts/<spec-id>/...`). When it omits `script`, the inherited spex script is reused (`spex/scripts/<spec-id>/...`). Set `disable: true` to turn the inherited check off completely; all other override fields are then ignored. An override that targets a `spec`/`name` the repository does not inherit is reported as an error.

### `checks_additional` — add a new check

Each entry is a complete check plus the `spec` it belongs to (which must be one of the repository's `tech-specs`). It runs in addition to everything inherited. Script paths resolve under the repository directory.

```yaml
repositories:
  - repository_name: frontend
    path: ../frontend
    tech-specs: [TECH-WRITING-STYLE]
    checks_additional:
      - spec: TECH-WRITING-STYLE
        name: no-todo
        type: regexp
        pattern: "TODO"
        include: ["src/**/*.tsx"]
        severity: warn
```

## See also

- [FAQ](faq.md)
