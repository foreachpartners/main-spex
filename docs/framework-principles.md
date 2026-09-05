# Framework principles

These principles form the foundation of spex. They are non-negotiable. Everything else — the artifact flow, specs and rules, the CLI, the skills — is a consequence of them. See [glossary.md](glossary.md) for terms, [specification.md](specification.md) for how the framework behaves, and [conventions.md](conventions.md) for the engineering values spex follows but didn't invent.

## 1. AI over Manual Coding

Used right, AI writes code faster than a developer while keeping quality high. It is cheap — it can even run locally for the price of electricity — and it keeps getting faster and better over time. Developers who insist on writing every line by hand will simply lose the competition.

spex is built around this premise: the agent writes every artifact — specs, contracts, plans, code, and tests — while the developer validates each one and owns the decisions by approving it before the next is generated. The other principles make that delegation safe — they give the agent unambiguous input and prove its output.

**Violation test**: a team adopts spex but still routes every change through manual implementation, treating the agent as optional — the speed advantage the framework is built for is forfeited.

## 2. Specs over Guesswork

Casual requests force an AI to guess, leading to inaccurate repository navigation and assumption-based task completion. Ask an AI to "make me a snake game" and it will — but it will be the snake game the AI imagined, not the one you wanted, because you wanted a blue snake with big eyes.

Clear, well-defined specs eliminate guesswork: they provide explicit acceptance criteria for the AI to follow and for the developer to verify. Specs stay relevant for a long time even as models get faster and better, because they describe an idea from your head that must meet your — or your customer's — expectations, not someone else's.

Asking an agent to "implement me a snake game where snake is blue and has big eyes" will produce expected results most of the time.

The same guesswork happens at the repository level: ad hoc prompting re-explains context that already lives in the repo — commands, structure, conventions — every time, and the outcome depends on what the developer remembers to say that day. A repository skill (for example `pepperoni-pizza-api`) writes that context down once and is autoloaded before any read, write, test, or run in that repository, so the agent gets it without guessing or asking.

**Violation test**: an agent implements a feature with no acceptance criteria to verify against — the result is whatever the model imagined, not what was wanted. Likewise, a developer explains a repository's structure and commands to the agent freehand instead of the agent autoloading a skill for it — the instructions live in the developer's memory, not in the framework.

## 3. Rules over Tribal Knowledge

Cross-cutting technical rules and patterns — code style, structure, testing, security — are captured as rules (`*`) instead of living as tribal knowledge, to ensure project maintainability over the long term.

They apply repo-wide and are proven by automated checks (linters, CI gates) or AI, so every change follows the same rules no matter who — human or AI model — writes the code. A one-off technical implementation — a specific endpoint, integration, or mechanism — is not a cross-cutting rule; it is a capability.

A task skill (`spex-spec`, `spex-plan`, `spex-implement`, `spex-test`, `spex-review`, ...) carries this the same way for workflow steps: it is invoked by name to run one step of the spec → plan → implement → check loop with the right rules and acceptance criteria already written in, so any developer or any session follows the same steps instead of reconstructing them from memory. Freeform chat fills the judgment calls a skill leaves open — it doesn't replace the skill.

**Violation test**: a code-style or security rule lives only in someone's head or a chat message — it is applied inconsistently and cannot be enforced. Likewise, a developer runs a workflow step ad hoc instead of invoking the task skill for it — the step's rules and inputs depend on what that developer remembers that day, not on what the framework enforces.

## 4. Human in control

Vibe coding ships working code fast, but every unreviewed decision adds mess that compounds — eroding maintainability, trust, and the team's comprehension of the codebase. spex keeps the speed without the mess: the AI writes everything — specs, contracts, plans, code, and tests — and the human validates everything, owning every decision by approving the artifact that records it.

Each artifact is approved before the next is generated: spec before contract, contract before plan, plan before code and tests. This makes work streamlined without haphazard backward edits, keeps the codebase maintainable, and means every change traces back to a decision a human approved.

**Violation test**: an artifact ships that no human reviewed and approved — control was lost.

## 5. Deterministic enforcement

AI writes code fast, but along the way it may change things nobody asked it to and invent acceptance criteria that no spec actually contains. Human review alone can't catch all of it. Deterministic checks (tests, rule checks, linting, schema validation, coverage) prove the change does what the spec demands and that nothing else broke. They give the same answer every time, so they catch what both the AI and the human overlook.

That is why every check that can be a script must be a script. AI fills only the gaps where judgment is required — spec and contract design, code and test generation, plan splitting, check triage, semantic review — the interpretive work no deterministic tool can do.

The tool's language is irrelevant (Bash, Python, Rust); what matters is deterministic input → output. In spex, rule checks are the deterministic layer: `regexp`, `glob`, and `script` checks run through `spex check` and gate on `error`-severity findings. See [specification.md](specification.md) for the full set of deterministic checks (`cargo build`/`tsc`, linters, `spex check`, `spex coverage`, `spex check-schema`, contract validators, test runners).

**Violation test**: a check that could give a deterministic pass/fail is instead left to human or AI judgment — it will eventually miss something a script would have caught every time.

## 6. Traceability

Every behavioral change traces to a spec. Bidirectional: top-down (spec → contract → plan → code → test) and bottom-up (test → code → plan → contract → spec). No orphans.

- The Spec ID (`delivery-time`) is the traceability unit.
- Annotations link artifacts to a spec at the layer boundary: `@spec contract|impl|test <spec-id>` in code, `x-spec: <spec-id>` in OpenAPI.
- `spex coverage` computes coverage from those annotations.
- Checking out a commit shows exactly which specs the code was built against.
- Spec traceability is required, rule traceability is optional.

**Violation test**: a code change affects behavior and carries no Spec ID — it is a violation.
