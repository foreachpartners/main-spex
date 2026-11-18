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

### Implementation and verification

- MUST assemble the governing task inputs with [`spex context`](../cli/spex-context-cli-cmd.md) once the relevant spec IDs and packages are known. Pass the same bundle to review and tests while its governing specs, rules and contracts stay current.
- MUST use the short path for a change to the implementation of one existing acceptance criterion in one package when the user has delegated the choice and the current specification, contract and architecture still apply. The short path uses one implementation pass, one review of both axes, targeted tests and the relevant rule and trace checks.
- MUST allow ordinary implementation, review and testing with a single agent. Use independent contexts for separate roles when the runtime supports them and the user permits it.
- MUST run a final rule review over the changed scope when a spec fix follows the last clean rule review. Keep unresolved findings visible when a review reaches its pass cap.
- MUST distinguish trace coverage, rule checks, contract validation, tests executed and manual acceptance in the result. Record the evidence in [`spex receipt`](../cli/spex-receipt-cli-cmd.md) when a journal is explicitly requested.
