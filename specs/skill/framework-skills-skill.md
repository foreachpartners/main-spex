# Framework skills

Agent skills are the instructions an AI agent follows to do the job consistently and correctly.

## Acceptance criteria

- MUST live in the agent root, installed under `.agents/skills/`; the framework ones ship from `templates/skills/`.
- MUST describe, in each `SKILL.md`, what the skill does and when to load it or not.
- MUST trigger a skill by name (e.g. `/spex-spec`) or load it on demand by the agent from its description.
- MUST end every framework `SKILL.md` with a `# Project` section for project-specific additions ([managed-section-framework.md](../framework/managed-section-framework.md)).
- MUST follow the [Agent Skills specification](https://agentskills.io/specification) in a skill's `SKILL.md` frontmatter.
- MUST decide whether `spex update` overwrites a framework skill by its `metadata.sync-with-upstream` ([spex-update-cli-cmd.md](../cli/spex-update-cli-cmd.md)).
- MUST name a skill in lowercase, hyphen-separated with `spex` prefix for all skills except special one like `spex` or package skills.
- MUST give each package its own package skill, named `<package-name>`, created by `spex-setup` from its package skill template; unlike a framework skill, `spex update` never overwrites it.
- MUST give each package skill a `references/review.md` of what reviews of the package kept running into, created by `spex-setup` and listed in the package skill's `# Index`; the package skill itself says to read it before a review and to add only what cost a review real time or a question to the user and would recur, so most reviews add nothing.
