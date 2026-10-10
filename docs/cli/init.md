# `spex init`

Initializes the spex workspace. Run it from the workspace root or from inside an empty spex repository directory (`<project-name>-spex`), which is reused instead of created.

It sets up a fresh project only: it fails when the spex repository directory is not empty, so it never overwrites anything. Use `spex update` for an initialized project.

It prompts to confirm the project name (defaulting to the workspace root folder name), then writes:

- `spex.yaml`, seeded from the bundled template with the project name filled in and the installed CLI's version as `cli_version`. It lists only the spex repository (`<project-name>-spex`) and the rules that apply to it
- `<project-name>.code-workspace`, seeded from the bundled template with a folder for the spex repository only
- the bundled templates: schemas, docs, skills, and every bundled rule with its scripts, plus empty `specs/`, `plans/`, and `contracts/` folders
- `.agents/skills/` with the bundled skills, and a `.claude` -> `.agents` symlink, so different agent tools detect the same skills
- a root `AGENTS.md` and a `CLAUDE.md` -> `AGENTS.md` symlink. A `# Project` marker splits `AGENTS.md`: content above it is managed and kept in sync by `spex update`, content from the marker onward is the user's own and is never touched

It does not look for, ask about, or configure the other workspace repositories.

```sh
spex init
```

Finish setup afterwards by running the `/spex-setup` skill: it adds the other workspace repositories to `spex.yaml` and the workspace file, and creates a skill for every workspace repository, including the spex repository.
