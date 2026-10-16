# `spex-create-skill` skill

A reference for writing and editing agent skills well: the vocabulary and principles that make a skill predictable. Loaded when the user wants to create a new skill, edit an existing skill, or review a skill's `SKILL.md` for quality.

## Acceptance criteria

- MUST treat predictability — the agent taking the same process every run, not producing the same output — as the root virtue, and define the bold terms it uses in `references/GLOSSARY.md`.
- MUST pick invocation correctly: a model-invoked skill keeps a description so the agent can fire it autonomously and other skills can reach it, a user-invoked skill (`disable-model-invocation: true`) strips it and spends cognitive load instead; pick model-invocation only when the agent or another skill must reach it.
- MUST cure the cognitive load of many user-invoked skills with a router skill.
- MUST write a model-invoked description that front-loads the skill's leading word, states what the skill is, gives one trigger per branch, and cuts identity already in the body.
- MUST rank content on the information hierarchy (in-skill step, in-skill reference, external reference), end each in-skill step on a checkable completion criterion, and push branch-specific reference behind context pointers under `references/`.
- MUST co-locate a concept's definition, rules, and caveats under one heading.
- MUST split a skill by invocation only when a distinct leading word should trigger it or another skill must reach it, and by sequence only when later steps would tempt the agent to rush the current one.
- MUST keep each meaning in a single source of truth, check every line for relevance, and delete whole no-op sentences.
- MUST diagnose issues with the failure modes (premature completion, duplication, sediment, sprawl, no-op, negation) and refactor with leading words where they collapse restatements.
- MUST put a new spex-authored skill under `templates/skills/<skill-name>/SKILL.md`, scaffolded into `.agents/skills/<skill-name>/SKILL.md`, named lowercase and hyphen-separated, matching the directory.
