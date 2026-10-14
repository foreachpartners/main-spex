# Bundled skills

Agent skills drive the artifact chain: each packages the instructions an AI coding agent follows to run one workflow step, applying every rule that governs the packages it touches. [glossary.md](../../docs/glossary.md) defines an Agent Skill.

## Acceptance criteria

- MUST live in the agent root, installed under `.agents/skills/`; the bundled ones ship from `templates/skills/`.
- MUST describe, in each `SKILL.md`, what the skill does and when to load it.
- MUST trigger a skill by name (e.g. `/spex-spec`) or load it on demand by the agent from its description.
- MUST apply every rule that governs the packages the skill touches.
- MUST end every bundled `SKILL.md` with a `# Project` section for project-specific additions ([managed-section-framework.md](../framework/managed-section-framework.md)).
- MUST follow the [Agent Skills specification](https://agentskills.io/specification) in a skill's `SKILL.md` frontmatter.
- MUST decide whether `spex update` overwrites a skill by its `metadata.sync-with-upstream` ([spex-update-cli-cmd.md](../cli/spex-update-cli-cmd.md)).
- MUST name a skill in lowercase, hyphen-separated, matching the containing directory exactly.
