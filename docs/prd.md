# Product Requirements Document

Terms are defined in [glossary.md](glossary.md).

## Problem

Ask an AI to write you a snake game and it will. But will the snake have big blue eyes and eat donuts, as you wanted? Unlikely — the AI fills every detail you left out with its own guesses, so you get the game it imagined, not the one you wanted. A spec closes that gap: it states what you want and the AI follows it.

AI writes code fast, but fast is not the same as right. Given a request, AI can add changes nobody asked for, ignore or forget some requirements, do the same thing differently each time, or just hallucinate. Conventions drift the same way: every contributor, human or AI, applies them a little differently. Every unreviewed decision and unverified result adds slop that compounds — eroding maintainability, trust, and the team's comprehension of the codebase. Reviewing everything by hand throws away the speed; accepting everything ships the slop.

## Goal

spex helps to produce slop-free, verifiable results fast, with minimal human intervention — unleashing AI's full speed. The AI writes everything, following the specs, skills, and rules. The human decides, reviews, and approves what matters. Verification layers of agent reviews, tests, and deterministic checks prove the result.

## Success

A project adopts spex quickly and builds with it successfully. spex is:

- **Quick to start** — a project can initialize it in five minutes.
- **Easy to understand** — natural to reason about from minimal docs, or none at all.
- **Agnostic** — works with any AI model, AI coding agent, CLI, and IDE.
- **Flexible** — easy to adapt to a project's needs.

## Capabilities

- Specify each capability with acceptance criteria that can be verified.
- Capture cross-cutting conventions as rules every applicable package must follow, provided as context to the agent and proven by automated checks.
- Trace every acceptance criterion to its contract, implementation, and test, and report what is not covered.
- Guide the AI coding agent through each workflow step using skills.
- Run and automate all of it using the spex CLI.

## Principles

These principles are the non-negotiable foundation of spex. Everything else — the artifact chain, specs and rules, the CLI, the skills — is a consequence of them.

### AI over Manual Coding

Used right, AI writes code faster than a developer while keeping quality high. It is cheap — it can even run locally for the price of electricity — and it keeps getting faster and better over time. A team that writes every line by hand cannot match the speed of one that delegates it to AI.

spex is built around this premise: the agent writes every artifact — specs, contracts, plans, code, and tests. The other principles make that delegation safe — they give the agent unambiguous input and prove its output.

**Violation test**: a team adopts spex but still routes every change through manual implementation, treating the agent as optional — the speed advantage the framework is built for is forfeited.

### Specs over Guesswork

Casual requests force an AI to guess, leading to inaccurate repository navigation and assumption-based task completion.

Clear, well-defined specs eliminate guesswork: they provide explicit acceptance criteria for the AI to follow and for the developer to verify. Specs stay relevant for a long time even as models get faster and better, because they describe an idea from your head that must meet your — or your customer's — expectations, not someone else's.

**Violation test**: an agent implements a feature with no acceptance criteria to verify against — the result is whatever the model imagined, not what was wanted.

### Rules over Tribal Knowledge

Cross-cutting technical conventions — code style, structure, testing, security — are captured as rules instead of living as tribal knowledge, to keep the project maintainable over the long term.

They apply package-wide and are proven by automated checks (linters, CI gates) or AI, so every change follows the same rules no matter who — human or AI model — writes the code. A one-off technical implementation — a specific endpoint, integration, or mechanism — is not a cross-cutting rule; it is a capability.

**Violation test**: a code-style or security rule lives only in someone's head or a chat message — it is applied inconsistently and cannot be enforced.

### Skills over Ad Hoc Prompting

Left to chat prompts, every workflow step is improvised: what to read, which rules apply, what to update afterward — all depend on what the human or AI remembered that day. Skills are the instructions an AI agent follows to do the job consistently and correctly. Each skill packages one workflow step as a written, repeatable procedure that gathers the governing specs and applicable rules itself.

The same holds for a package's structure and commands: a package skill is written down once and loaded automatically whenever the agent works on that package, instead of being re-explained every session. The agent keeps it current as the package changes.

Skills load progressively: only each skill's description is always in context; the body loads when the skill is invoked or the request matches its description. This keeps the context small and the output better. Bundled skills stay in sync with upstream through `spex update`, and a project extends or replaces them with its own.

**Violation test**: a workflow step runs from an ad hoc prompt instead of its skill, or a package's structure and commands are explained by hand each session — the result depends on what was remembered that day, not on what the framework holds.

### Focused Human Review over Vibe Coding

Vibe coding ships code fast, but every unreviewed decision adds mess that compounds. spex keeps the speed without the mess: the AI writes everything, and every change passes layers of validation — agent reviews against the spec, contract and rules, deterministic checks, and tests — so the human can focus on what matters most.

Human review concentrates on specs and contracts, which are small and fast to reason about, and on the domain model and other important changes. Minor changes can rely on agent review, tests, and checks alone. Approving the spec and contract before implementation keeps work streamlined, without haphazard backward edits, and means every behavior change traces back to a decision a human reviewed. The other layers keep the rest of the codebase maintainable.

Minimal human intervention means the human reviews and decides on what matters most, never that the human is out of the loop.

**Violation test**: a spec, contract, or important change ships that no human reviewed — control was lost.

### Deterministic Checks over Judgment

AI writes code fast, but along the way it may change things nobody asked it to and invent acceptance criteria that no spec actually contains. Human review alone can't catch all of it. Deterministic checks (tests, rule checks, linting, schema validation, coverage) prove the change does what the spec demands and that nothing else broke. They give the same answer every time, so they catch what both the AI and the human overlook.

That is why every check that can be a script must be a script. AI is left to handle what a script can't: writing specs and contracts, generating code and tests, splitting plans, triaging check output, reviewing logic. The tool's language is irrelevant (Bash, Python, Rust); what matters is deterministic input -> output.

**Violation test**: a check that could be a deterministic pass/fail is left to human or AI judgment instead — it will eventually miss something a script would have caught every time.

### Traceability over Assumption

Every behavioral change traces to a spec and down to its acceptance criterion, in both directions: top-down (spec -> contract -> code -> test) and bottom-up (test -> code -> contract -> spec). No orphans, and no acceptance criterion left uncovered. Checking out a commit shows exactly which specs — and which acceptance criteria — the code was built against.

Traceability pays off twice:

- **Focused context** — a human or agent finds everything related to a spec at once instead of searching the codebase. That saves tokens, improves output, and speeds up the work.
- **Nothing forgotten** — an agent can forget to implement or test an acceptance criterion. With every criterion traced, the deterministic coverage check proves each one is implemented and tested.

**Violation test**: a code change affects behavior and carries no trace to a spec, or an acceptance criterion has no trace to a contract, implementation, or test — either is a violation.

## Values

Engineering values spex follows but didn't invent — good practice for any codebase, not axioms unique to spex. They support the principles above.

### Simplicity

The tool should help the developer get work done, not stand in the way. Simplicity is a constraint on the framework itself, not only on the code it governs: a new command, artifact type, or skill earns its place only when it removes more friction than it adds.

**Violation test**: a feature adds a step to the workflow without removing more friction than it introduces — the tool now stands in the way.

### Tool-agnostic

To avoid lock-in and restrictions, the framework works with any IDE, terminal, or AI model.

Specs, contracts, and checks are plain files and deterministic commands, so no part of the workflow depends on a particular editor or vendor. Skills are portable instructions, not proprietary integrations.

**Violation test**: a step in the workflow functions only in one specific editor or with one specific model — the framework has acquired a lock-in.

### Ubiquitous language

A shared vocabulary connects business, specs, and code so humans and AI mean the same thing by the same term.

spex defines its terms in the [glossary](glossary.md) and uses them consistently across specs, contracts, code, and docs, removing ambiguity and the translation errors it causes. A shared term used the same way everywhere is what lets a spec, a contract, and an implementation refer to one concept without restating it.

**Violation test**: the same concept is named two different ways across specs, code, and docs — every reader, human or agent, must translate, and some translate wrong.

### Single source of truth

Every fact other artifacts depend on has exactly one authoritative source. Everything else references it: a native contract for an interface, a spec for a capability, a rule for a cross-cutting convention, `spex.yaml` for configuration, the glossary for term definitions.

This covers load-bearing facts — behavior, interfaces, conventions, configuration, definitions — not wording. Restating an idea, summarizing, or repeating a phrase where it helps the reader is fine: if one copy changes, nothing else becomes wrong.

Duplication has always been a source of bugs and drift. With AI-assisted development it becomes critical: agents generate code fast, copies diverge faster, and a duplicated fact falls out of sync before anyone notices.

**Violation test**: the same load-bearing fact is stated in two places, so changing one makes the other wrong — one is already wrong, or soon will be.

**Agent-specific risk**: AI agents copy context between files for convenience ("let me add this to the agent rules too"). Every such copy of a load-bearing fact is a future inconsistency. Reference, do not copy.

### Parsimony

The minimum representation that preserves full semantics and enforceability.

The token budget is finite. Every token spent on filler, boilerplate, or irrelevant context is a token not available for substantive content — specs, contracts, code, examples. Parsimony is not about spending less; it is about allocating the budget where it has the highest impact on result quality.

Size matters beyond the budget: the bigger the context, the worse the output. A model buries the instructions that matter under everything around them, loses track of them, and fills the gaps with invention — so as context grows, quality drops and hallucinations become more likely. A small, focused context is not just economical; it makes the agent more accurate.

Three requirements:

- **Minimality** — exclude everything that does not affect the agent's behavior in the current task: redundant explanations, repeated instructions, irrelevant fragments.
- **Sufficiency** — compression must not introduce ambiguity. The context stays complete enough for the agent to reconstruct goal, constraints, and result format.
- **Budget prioritization** — tokens saved on operational description are reallocated to substantive artifacts, which have the strongest impact on output quality.

**Violation test**: a context holds an element that could be removed without introducing ambiguity or degrading quality, or has been compressed until it became ambiguous.

**Parsimony is not unreadability**: imperative, filler-free prose reads easily for a competent person who knows the domain. Parsimony removes noise, not meaning.

### Adaptable

spex fits the project, not the other way around. The framework itself has no built-in opinion on language, framework, or architecture — those come only from the rules and skills a project chooses to use. Templates ship ready-made rules and skills for common stacks, but none are mandatory: a project picks the ones that match its stack, or writes its own.

The same framework scales from a solo script to a large multi-team repo: no minimum ceremony is forced on a small project, no ceiling is placed on a large one, and adoption is incremental.

**Violation test**: the framework core, a command, or a scaffolded default hard-codes a specific language, framework, or all-or-nothing adoption instead of leaving it to the project's chosen rules and skills — a project that doesn't match breaks or is forced into unneeded ceremony.

## Non-goals

- **Not an AI model, coding agent, or IDE** — it works with the ones a project already chose.
- **Not unsupervised AI coding** — a human approves every spec and contract and reviews important changes.
- **No opinion on language, framework, or architecture** — those come only from the rules and skills a project chooses.
- **Not a replacement for tests, linters, or CI** — it proves that a change follows its specs and rules; the project's own tooling still does the running.
