# Framework principles

These principles form the foundation of spex. They are non-negotiable. Everything else — the artifact flow, specs and rules, the CLI, the skills — is a consequence of them. See [glossary.md](glossary.md) for terms.

## Goal: unleash AI's full speed without slop

AI operates fast — the framework's job is to unleash its speed while ensuring slop-free verifiable results.

## 1. AI over Manual Coding

Used right, AI writes code faster than a developer while keeping quality high. It is cheap — it can even run locally for the price of electricity — and it keeps getting faster and better over time. Developers who insist on writing every line by hand will simply lose the competition.

spex is built around this premise: the agent writes every artifact — specs, contracts, plans, code, and tests — while the developer validates each one and owns the decisions by approving it before the next is generated. The other principles make that delegation safe — they give the agent unambiguous input and prove its output.

**Violation test**: a team adopts spex but still routes every change through manual implementation, treating the agent as optional — the speed advantage the framework is built for is forfeited.

## 2. Specs over Guesswork

Casual requests force an AI to guess, leading to inaccurate repository navigation and assumption-based task completion. Ask an AI to "make me a snake game" and it will — but it will be the snake game the AI imagined, not the one you wanted, because you wanted a blue snake with big eyes.

Clear, well-defined specs eliminate guesswork: they provide explicit acceptance criteria for the AI to follow and for the developer to verify. Specs stay relevant for a long time even as models get faster and better, because they describe an idea from your head that must meet your — or your customer's — expectations, not someone else's.

Asking an agent to "implement me a snake game where snake is blue and has big eyes" will produce expected results most of the time.

The same problem shows up at the repository level: without a skill, every session re-explains the same commands and conventions from scratch. A repository skill (for example `shopping-notes-api`) writes that down once and autoloads before any read, write, test, or run, so the agent doesn't have to guess or ask.

**Violation test**: an agent implements a feature with no acceptance criteria to verify against — the result is whatever the model imagined, not what was wanted. Likewise, a developer explains a repo's structure and commands by hand instead of relying on an autoloaded skill — the knowledge stays in the developer's head, not in the framework.

## 3. Rules over Tribal Knowledge

Cross-cutting technical rules and patterns — code style, structure, testing, security — are captured as rules (`*`) instead of living as tribal knowledge, to ensure project maintainability over the long term.

They apply repo-wide and are proven by automated checks (linters, CI gates) or AI, so every change follows the same rules no matter who — human or AI model — writes the code. A one-off technical implementation — a specific endpoint, integration, or mechanism — is not a cross-cutting rule; it is a capability.

Task skills (`spex-spec`, `spex-plan`, `spex-implement`, `spex-test`, `spex-review`, ...) do the same for workflow steps: invoke one by name and it runs that step with the right rules and acceptance criteria already built in, instead of everyone reconstructing them from memory. Chat still handles the judgment calls a skill leaves open, but it doesn't replace the skill.

**Violation test**: a code-style or security rule lives only in someone's head or a chat message — it is applied inconsistently and cannot be enforced. Likewise, someone runs a workflow step ad hoc instead of using its task skill — the result depends on what they remembered that day, not on what the framework enforces.

## 4. Human in control

Vibe coding ships working code fast, but every unreviewed decision adds mess that compounds — eroding maintainability, trust, and the team's comprehension of the codebase. spex keeps the speed without the mess: the AI writes everything — specs, contracts, plans, code, and tests — and the human validates everything, owning every decision by approving the artifact that records it.

Each artifact is approved before the next is generated: spec before contract, contract before plan, plan before code and tests. This makes work streamlined without haphazard backward edits, keeps the codebase maintainable, and means every change traces back to a decision a human approved.

**Violation test**: an artifact ships that no human reviewed and approved — control was lost.

## 5. Deterministic enforcement

AI writes code fast, but along the way it may change things nobody asked it to and invent acceptance criteria that no spec actually contains. Human review alone can't catch all of it. Deterministic checks (tests, rule checks, linting, schema validation, coverage) prove the change does what the spec demands and that nothing else broke. They give the same answer every time, so they catch what both the AI and the human overlook.

That is why every check that can be a script must be a script. AI is left to handle what a script can't: writing specs and contracts, generating code and tests, splitting plans, triaging check output, reviewing logic.

The tool's language is irrelevant (Bash, Python, Rust); what matters is deterministic input -> output. In spex, rule checks are the deterministic layer: `regexp`, `glob`, and `script` checks run through `spex check` and gate on `error`-severity findings. See [specification.md](specification.md) for the full list of deterministic checks.

**Violation test**: a check that could be a deterministic pass/fail is left to human or AI judgment instead — it will eventually miss something a script would have caught every time.

## 6. Traceability

Every behavioral change traces to a spec and down to the acceptance criterion. Bidirectional: top-down (spec -> contract -> code -> test) and bottom-up (test -> code -> contract -> spec). No orphans, and no acceptance criterion left uncovered.

- The Spec ID (`delivery-time`) scoped to an acceptance criterion (`delivery-time#created-order-status`), is the traceability unit.
- Annotations link artifacts to a spec at the layer boundary: `@spec contract|impl|test <spec-id>#<ac-id>` in code.
- `spex coverage` computes coverage from those annotations, per spec and per acceptance criterion.
- Checking out a commit shows exactly which specs — and which acceptance criteria — the code was built against.

**Violation test**: a code change affects behavior and carries no Spec ID, or an acceptance criterion has no annotation tracing to it — either is a violation.
