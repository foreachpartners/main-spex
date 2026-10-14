# Managed section

`AGENTS.md` and every bundled `SKILL.md` split at a `# Project` marker into a managed section kept in sync by `spex update` and a user section that `spex update` never touches. [spex-init-cli-cmd.md](../cli/spex-init-cli-cmd.md) writes the split, [spex-update-cli-cmd.md](../cli/spex-update-cli-cmd.md) honors it.

## Acceptance criteria

- MUST split `AGENTS.md` and every bundled `SKILL.md` at a `# Project` heading, the marker.
- MUST end every bundled `SKILL.md` with a `# Project` section for project-specific additions.
- MUST treat everything above the marker as managed: `spex update` rewrites it with the version shipped in the installed binary.
- MUST treat everything from the marker onward as the user's own: `spex update` preserves it.
- MUST overwrite a local file that predates the marker in full, since managed and user content cannot be told apart in it.
- MUST keep an existing `AGENTS.md` content, when `spex init` adopts it, as the user's own section below the marker.
