# `spex-create-skill` skill

A reference for writing and editing agent skills well: the vocabulary and principles that make a skill predictable. Loaded when the user wants to create a new skill, edit an existing skill, or review a skill's `SKILL.md` for quality. The bold terms it uses are defined in `references/GLOSSARY.md`.

## Acceptance criteria

### Predictability

- MUST treat predictability, the agent taking the same process every run (not producing the same output), as the root virtue, with every lever serving it.
- MUST define the bold terms it uses in `references/GLOSSARY.md`, and point to it for their full meaning.

### Invocation

- MUST offer two invocation choices, trading different costs: a model-invoked skill, which keeps a description so the agent can fire it autonomously and other skills can reach it (omit `disable-model-invocation`, write a model-facing description with rich trigger phrasing), and a user-invoked skill, which strips the description from the agent's reach (set `disable-model-invocation: true`, write a human-facing one-line description with trigger lists stripped).
- MUST pick model-invocation only when the agent must reach the skill on its own, or another skill must; a skill that only ever fires by hand is user-invoked.
- MUST cure the cognitive load of many user-invoked skills with a router skill: one user-invoked skill that names the others and when to reach for each.

### Description

- MUST write a model-invoked description that states what the skill is and lists the branches that should trigger it.
- MUST front-load the skill's leading word in the description.
- MUST give one trigger per branch, collapsing synonyms that rename a single branch.
- MUST cut from the description any identity already in the body, keeping the triggers plus any "when another skill needs..." reach clause.

### Information hierarchy

- MUST build a skill from steps and reference, mixed freely, ranked on the information hierarchy: in-skill step, in-skill reference, external reference.
- MUST end each in-skill step on a completion criterion that is checkable, and, where it matters, exhaustive.
- MUST push reference out of `SKILL.md` into a linked file under `references/`, named for what it holds, reached by a context pointer, when it is needed only by some branches; inline what every branch needs.
- MUST keep a concept's definition, rules, and caveats under one heading (co-location).

### When to split

- MUST split off a model-invoked skill by invocation only when a distinct leading word should trigger it on its own, or another skill must reach it, since each cut pays context load for a new always-loaded description.
- MUST split a run of steps by sequence only when the steps still ahead tempt the agent to rush the one in front of it (premature completion).

### Pruning

- MUST keep each meaning in a single source of truth.
- MUST check every line for relevance to what the skill does.
- MUST hunt no-ops sentence by sentence, deleting a whole failing sentence rather than trimming words from it.
- SHOULD hunt for opportunities to refactor a skill to use leading words that collapse restatements into a single pretrained token.

### Failure modes

- MUST diagnose a skill's issues using these failure modes: premature completion (sharpen the completion criterion first, hide the post-completion steps by splitting only if the rush is observed), duplication, sediment, sprawl (disclose reference behind pointers, split by branch or sequence), no-op (use a stronger leading word, not a different technique), and negation (state the target behavior positively; keep a prohibition only as a hard guardrail, paired with what to do instead).

### Where spex skills live

- MUST put a new spex-authored skill under `templates/skills/<skill-name>/SKILL.md` in this repository, scaffolded into `.agents/skills/<skill-name>/SKILL.md` of every spex project by `spex init` and `spex update`.
- MUST name the skill in the frontmatter `name` in lowercase, hyphen-separated, and matching the containing directory exactly ([bundled-skills-skill.md](bundled-skills-skill.md)).
