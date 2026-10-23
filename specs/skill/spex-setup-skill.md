# `spex-setup` skill

Configures one or several new packages for the spex framework: defines each in `spex.yaml`, creates a package skill for each, and adds it to the editor workspace file. Loaded when the user wants to set up new packages, typically after [`spex init`](../cli/spex-init-cli-cmd.md).

## Acceptance criteria

- MUST resolve `<spex-dir>`, `<agent-root>`, and the project layout from `spex info`, and read `<spex-dir>/schemas/spex.schema.json`, the authority for `spex.yaml`'s fields, before editing it.
- MUST find the new packages by layout (`multi-repo`: the non-hidden directories under the project root, or the ones named; `single-repo`: the repository itself; `monorepo`: the workspace manifest, the conventional folders, or the ones named), confirm the list with the user, and drop anything that is not a package or that `packages` already lists.
- MUST inspect each package and propose its `spex.yaml` entry — `package_name`, `package_short_name`, `description`, `path`, `contracts`, and the `rules` that genuinely apply — showing the proposal to the user and writing only what they confirm.
- MUST append the confirmed entries to `packages` sorted by `package_name`, leave every existing field, entry, and rule file untouched, and validate `spex.yaml` against `schemas/spex.schema.json`.
- MUST NOT move or copy a contract file.
- MUST create `<agent-root>/.agents/skills/<package-name>/SKILL.md` for each new package from [`references/GENERIC-PACKAGE-SKILL-TEMPLATE.md`](references/GENERIC-PACKAGE-SKILL-TEMPLATE.md), keeping its frontmatter, sections, and order and filling each with real facts ([package-skill-skill.md](package-skill-skill.md)).
- MUST NOT overwrite an existing `SKILL.md` or invent a path or command.
- MUST add each new package to `<project-name>.code-workspace` (`multi-repo` only), leaving existing folders untouched.
- MUST run the [`spex-prd`](spex-prd-skill.md) skill to write the PRD when the file `prd` names does not exist or is empty, since every project needs one.
- MUST offer to author the first spec with `spex-spec` or the first rule with `spex-rule`.
