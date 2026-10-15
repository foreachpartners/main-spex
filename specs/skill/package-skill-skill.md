# Package skill

A package skill, named `<package-name>`, holds the package-specific context an agent needs and required to be loaded whenever the agent works on that package. [`spex-setup`](spex-setup-skill.md) creates it from `references/GENERIC-PACKAGE-SKILL-TEMPLATE.md`, and [`spex-implement`](spex-implement-skill.md) keeps it current.

## Acceptance criteria

- MUST containt usefull for AI agent information that helps completing tasks.
- MUST live at `<agent-root>/.agents/skills/<package-name>/SKILL.md`.
- MUST have a description stating that it holds information, rules, commands, and other `<package-name>` package context, and MUST be loaded before reading, writing, testing, or running any code in that package.
- MUST have a `# General` section that tells the agent to run `spex info --package <package-short-name>` for general package information and applicable rules, with a one-line description of what the package is and does.
- MUST have a `# Project structure` section listing the package's directories and entry points, each with a one-line role.
- MUST have a `# Commands` section listing the commands that can be run from the package root (serve, test, lint, format).
- MUST have a `# Rules` section naming pakage rules, for example what to run after all changes (format, lint, test).
- MUST state real facts only: layout, manifest scripts, CI config, contributor docs.
- MUST NOT duplicate what specs or rules already state; reference them instead.
- MAY omit a section when the package has nothing genuine for it (for example `# Rules` when no test, lint, or format policy exists).
- MAY have an `# Index` section, when the skill folder holds `references/`, `scripts/`, or `assets/` files, listing each file with a one-line description.
