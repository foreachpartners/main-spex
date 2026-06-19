# Guide

In-depth guide to working with spex, picking up where the [README](../README.md) quickstart leaves off.

## Updating spex

After upgrading the `spex` binary, run `spex update` to pull the latest framework files into your spex repository:

```sh
spex update
```

See the [CLI reference](cli.md) for the full command set.

## Per-repository checks

Every workspace repository inherits the checks declared by the tech specs in its `tech_specs`. A repository can adjust those inherited checks, or add new ones, by declaring them in `spex.yaml`. A check's identity is `<spec>/<name>`, so every per-repo entry names both the `spec` and the `name`.

A `script` check's `script:` value is just the filename; spex resolves it at `scripts/<spec-id>/<script>` in the spex repository. Every check script lives there — inherited, `checks_override`, and `checks_additional` alike. The `<spec-id>` folder namespaces scripts so checks from different specs never share a path. A script's `run:` scope decides where it executes (`per-repo`/`per-file` run with the repository as the working directory), but the file itself always lives in spex.

### `checks_override` — patch an inherited check

Name the inherited check by `spec` + `name`, then set only the fields you want to change. Every other field is inherited. The merged check runs instead of the inherited one.

```yaml
repositories:
  - repository_name: frontend
    path: ../frontend
    tech_specs: [TECH-WRITING-STYLE]
    checks_override:
      - spec: TECH-WRITING-STYLE # only the severity changes
        name: no-cyrillic
        severity: warn
      - spec: TECH-WRITING-STYLE # point the check at a different spex script
        name: writing
        script: writing-style.sh
      - spec: TECH-WRITING-STYLE # turn the inherited check off entirely
        name: no-emoji
        disable: true
```

When an override sets `script`, it resolves in the spex repository (`spex/scripts/<spec-id>/...`), the same place the inherited script lives. Set `disable: true` to turn the inherited check off completely; all other override fields are then ignored. An override that targets a `spec`/`name` the repository does not inherit is reported as an error.

### `checks_additional` — add a new check

Each entry is a complete check plus the `spec` it belongs to (which must be one of the repository's `tech_specs`). It runs in addition to everything inherited. Script paths resolve in the spex repository (`spex/scripts/<spec-id>/...`).

```yaml
repositories:
  - repository_name: frontend
    path: ../frontend
    tech_specs: [TECH-WRITING-STYLE]
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
