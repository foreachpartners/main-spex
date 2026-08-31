# Framework principles

These principles form the foundation of spex. They are non-negotiable. Everything else — the artifact flow, specs and rules, the CLI, the skills — is a consequence of them. See [glossary.md](glossary.md) for terms and [specification.md](specification.md) for how the framework behaves.

## 1. AI over Manual Coding

Used right, AI writes code faster than a developer while keeping quality high. It is cheap — it can even run locally for the price of electricity — and it keeps getting faster and better over time. Developers who insist on writing every line by hand will simply lose the competition.

spex is built around this premise: the agent writes every artifact — specs, contracts, plans, code, and tests — while the developer validates each one and owns the decisions by approving it before the next is generated. The other principles make that delegation safe — they give the agent unambiguous input and prove its output.

**Violation test**: a team adopts spex but still routes every change through manual implementation, treating the agent as optional — the speed advantage the framework is built for is forfeited.

## 2. Specs over Guesswork

Casual requests force an AI to guess, leading to inaccurate repository navigation and assumption-based task completion. Ask an AI to "make me a snake game" and it will — but it will be the snake game the AI imagined, not the one you wanted, because you wanted a blue snake with big eyes.

Clear, well-defined specs eliminate guesswork: they provide explicit acceptance criteria for the AI to follow and for the developer to verify. Specs stay relevant for a long time even as models get faster and better, because they describe an idea from your head that must meet your — or your customer's — expectations, not someone else's.

Asking an agent to "implement me a snake game where snake is blue and has big eyes" will produce expected results most of the time.

**Violation test**: an agent implements a feature with no acceptance criteria to verify against — the result is whatever the model imagined, not what was wanted.

## 3. Rules over Tribal Knowledge

Cross-cutting technical rules and patterns — code style, structure, testing, security — are captured as rules (`*`) instead of living as tribal knowledge, to ensure project maintainability over the long term.

They apply repo-wide and are proven by automated checks (linters, CI gates) or AI, so every change follows the same rules no matter who — human or AI model — writes the code. A one-off technical implementation — a specific endpoint, integration, or mechanism — is not a cross-cutting rule; it is a capability.

**Violation test**: a code-style or security rule lives only in someone's head or a chat message — it is applied inconsistently and cannot be enforced.

## 4. Skills as Instructions

Ad hoc prompting reproduces the guesswork specs exist to eliminate: every request re-explains context that already lives in the repository, and the outcome depends on what the developer remembers to say to the agent that day. A skill writes that context and workflow down once — commands, structure, conventions, a step of the spec → plan → execute → check loop — so any developer or any session gets the same instructions the person who authored the skill intended, instead of reconstructing them from memory each time.

Skills are how instructions reach the agent, not chat. A repository skill (for example `pepperoni-pizza-api`) is autoloaded before any read, write, test, or run in that repository, giving the agent commands and structure it would otherwise have to guess or ask for. A task skill (`spex-spec`, `spex-plan`, `spex-implement`, `spex-test`, `spex-review`, ...) is invoked by name to carry one step of the loop with the right inputs and acceptance criteria already written in. Freeform chat fills the judgment calls a skill leaves open — it doesn't replace the skill.

**Violation test**: a developer explains a repository's structure and commands to the agent freehand instead of the agent autoloading a skill for it — the instructions live in the developer's memory, not in the framework.

## 5. Human in control

Vibe coding ships working code fast, but every unreviewed decision adds mess that compounds — eroding maintainability, trust, and the team's comprehension of the codebase. spex keeps the speed without the mess: the AI writes everything — specs, contracts, plans, code, and tests — and the human validates everything, owning every decision by approving the artifact that records it.

Each artifact is approved before the next is generated: spec before contract, contract before plan, plan before code and tests. This makes work streamlined without haphazard backward edits, keeps the codebase maintainable, and means every change traces back to a decision a human approved.

**Violation test**: an artifact ships that no human reviewed and approved — control was lost.

## 6. Deterministic enforcement

AI writes code fast, but along the way it may change things nobody asked it to and invent acceptance criteria that no spec actually contains. Human review alone can't catch all of it. Deterministic checks (tests, rule checks, linting, schema validation, coverage) prove the change does what the spec demands and that nothing else broke. They give the same answer every time, so they catch what both the AI and the human overlook.

That is why every check that can be a script must be a script. AI fills only the gaps where judgment is required.

The tool's language is irrelevant (Bash, Python, Rust); what matters is deterministic input → output. In spex, rule checks are the deterministic layer: `regexp`, `glob`, and `script` checks run through `spex check` and gate on `error`-severity findings.

**Deterministic — clear logic** (result unambiguous, repeatable):

| Category            | Tools                              | Example                                        |
| ------------------- | ---------------------------------- | ---------------------------------------------- |
| Compilation         | `cargo build`, `tsc`, `protoc`     | Type checking, contract-code alignment         |
| Linting             | `clippy`, `eslint`, `markdownlint` | Code style, formatting                         |
| Spec checks         | `spex check`                       | rule `regexp` / `glob` / `script` findings     |
| Coverage            | `spex coverage`                    | Missing `@spec` annotation on an active spec   |
| Schema validation   | `spex check-schema`                | A spec file invalid against its bundled schema |
| Contract validation | OpenAPI validator, proto lint      | Contract syntax correctness                    |
| Test execution      | `cargo test`, `vitest`             | Pass / fail — a factual result                 |

**AI — fuzzy logic** (requires interpretation, context, judgment):

| Category             | Why it is not automatable                              | Example                         |
| -------------------- | ------------------------------------------------------ | ------------------------------- |
| Spec writing         | Translating a business need into acceptance criteria   | `spex-spec` skill               |
| Contract design      | Trade-off choices, schema shape                        | REST vs gRPC, response shape    |
| Code generation      | Translating a spec and contract into an implementation | Implement `PLACE-PIZZA-ORDER`   |
| Test generation      | Choosing scenarios and edge cases                      | E2E from acceptance criteria    |
| Plan splitting       | Ordering work into parallel-safe steps                 | `spex-plan` skill               |
| Check triage         | False positive vs real violation vs script bug         | `spex check` output review      |
| Semantic code review | Logic errors beyond a linter's scope                   | Business logic, race conditions |

## 7. Simplicity

The tool should help the developer get work done, not stand in the way. Everything about spex is kept simple, so it can be adopted and put to use in five minutes.

Simplicity is a constraint on the framework itself, not only on the code it governs: a new command, artifact type, or skill earns its place only when it removes more friction than it adds.

**Violation test**: a feature adds a step to the workflow without removing more friction than it introduces — the tool now stands in the way.

## 8. Tool-agnostic

To avoid lock-in and restrictions, the framework works with any IDE, terminal, or AI model.

Specs, contracts, and checks are plain files and deterministic commands, so no part of the workflow depends on a particular editor or vendor. Skills are portable instructions, not proprietary integrations.

**Violation test**: a step in the workflow functions only in one specific editor or with one specific model — the framework has acquired a lock-in.

## 9. Ubiquitous language

A shared vocabulary connects business, specs, and code so humans and AI mean the same thing by the same term.

spex defines its terms in the [glossary](glossary.md) and uses them consistently across specs, contracts, code, and docs, removing ambiguity and the translation errors it causes. A shared term used the same way everywhere is what lets a spec, a contract, and an implementation refer to one concept without restating it.

**Violation test**: the same concept is named two different ways across specs, code, and docs — every reader, human or agent, must translate, and some translate wrong.

## 10. Single source of truth

Every fact has exactly one authoritative source. Everything else references it.

Duplication has always been a source of bugs. With AI-assisted development it becomes critical: agents generate code fast, copies diverge faster, and a duplicated fact falls out of sync before anyone notices.

| Fact                              | Single source                                                                                                             | Not duplicated to                                |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| HTTP / gRPC / event interface     | Native contract at the `path` declared in `spex.yaml` (OpenAPI, `.proto`, AsyncAPI), usually inside the owning repository | No markdown copy of the interface                |
| Capability (product or technical) | Spec (`<project-name>-spex/specs/<spec-id-lowercase>.yaml`)                                                               | No copy in code comments or plans                |
| Cross-cutting convention          | Rule (`<project-name>-spex/rules/<rule-id-lowercase>.yaml`)                                                               | No copy in agent rules, READMEs, or service docs |
| Check logic                       | Rule `checks` and `scripts/<rule-id-lowercase>/`                                                                          | No restated rule in CI config prose              |
| Workspace layout                  | `<project-name>-spex/spex.yaml`                                                                                           | No second list of repositories anywhere          |
| Term definitions                  | `docs/glossary.md`                                                                                                        | No re-definition in other docs                   |

**Violation test**: the same fact exists in two places — one is already wrong, or soon will be.

**Agent-specific risk**: AI agents copy context between files for convenience ("let me add this to the agent rules too"). Every such copy is a future inconsistency. Reference, do not copy.

## 11. Parsimony

The minimum representation that preserves full semantics and enforceability.

The token budget is finite. Every token spent on filler, boilerplate, or irrelevant context is a token not available for substantive content — specs, contracts, code, examples. Parsimony is not about spending less; it is about allocating the budget where it has the highest impact on result quality.

Three requirements:

- **Minimality** — exclude everything that does not affect the agent's behavior in the current task: redundant explanations, repeated instructions, irrelevant fragments.
- **Sufficiency** — compression must not introduce ambiguity. The context stays complete enough for the agent to reconstruct goal, constraints, and result format.
- **Budget prioritization** — tokens saved on operational description are reallocated to substantive artifacts, which have the strongest impact on output quality.

**Violation test**: a context is parsimonious when no essential part can be removed without introducing ambiguity or degrading quality, and the whole contains no redundant or irrelevant element.

**Parsimony is not unreadability**: imperative, filler-free prose reads easily for a competent person who knows the domain. Parsimony removes noise, not meaning. The operational rules that implement it are captured in [`WRITING-STYLE`](../templates/rules/writing-style.yaml).

## 12. Traceability

Every behavioral change traces to a spec. Bidirectional: top-down (spec → contract → plan → code → test) and bottom-up (test → code → plan → contract → spec). No orphans.

- The Spec ID (`DELIVERY-TIME`) is the traceability unit.
- Annotations link artifacts to a spec at the layer boundary: `@spec contract|impl|test <spec-id>` in code, `x-spec: <spec-id>` in OpenAPI.
- `spex coverage` computes coverage from those annotations.
- Checking out a commit shows exactly which specs the code was built against.
- Spec traceability is required, rule traceability is optional.

**Violation test**: a code change affects behavior and carries no Spec ID — it is a violation.
