# Bundled `writing-style` rule

The bundled [`writing-style`](../../templates/rules/writing-style.yaml) rule keeps prose direct, English, and free of filler. The rule file masters its criteria; this spec covers what spex ships and checks for it.

## Acceptance criteria

- MUST ship the rule at `templates/rules/writing-style.yaml` with its script folder, deployed by `spex init` and `spex update`.
- MUST report a `warn` finding for each line that contains `maybe`, `perhaps`, `I think`, `sorry`, or `unfortunately` as a whole word, `Great job!`, `Perfect!`, `Excellent!`, `Make sense?`, or `Got it?`, case-insensitively.
- MUST report an `error` finding for each file that contains a Cyrillic character.
- MUST NOT apply the phrasing check to `writing-style.yaml`, `writing-style.sh`, markdown files under a `plans/` directory, or files under `.agents/skills/` and `.claude/skills/`.
