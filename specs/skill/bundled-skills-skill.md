# Bundled skills

Agent skills are the instructions an AI agent follows to do the job consistently and correctly.

## Acceptance criteria

- MUST live in the agent root, installed under `.agents/skills/`; the bundled ones ship from `templates/skills/`.
- MUST describe, in each `SKILL.md`, what the skill does and when to load it or not.
- MUST trigger a skill by name (e.g. `/spex-spec`) or load it on demand by the agent from its description.
- MUST end every bundled `SKILL.md` with a `# Project` section for project-specific additions ([managed-section-framework.md](../framework/managed-section-framework.md)).
- MUST follow the [Agent Skills specification](https://agentskills.io/specification) in a skill's `SKILL.md` frontmatter.
- MUST decide whether `spex update` overwrites a skill by its `metadata.sync-with-upstream` ([spex-update-cli-cmd.md](../cli/spex-update-cli-cmd.md)).
- MUST name a skill in lowercase, hyphen-separated with `spex` prefix for all skills except special one like `spex` or package skills.
