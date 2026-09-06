# Conventions

These are engineering values spex follows but didn't invent — good practice for any codebase, not axioms unique to spex. See [framework-principles.md](framework-principles.md) for the non-negotiable foundation these support.

## 1. Simplicity

The tool should help the developer get work done, not stand in the way. Everything about spex is kept simple, so it can be adopted and put to use in five minutes.

Simplicity is a constraint on the framework itself, not only on the code it governs: a new command, artifact type, or skill earns its place only when it removes more friction than it adds.

**Violation test**: a feature adds a step to the workflow without removing more friction than it introduces — the tool now stands in the way.

## 2. Tool-agnostic

To avoid lock-in and restrictions, the framework works with any IDE, terminal, or AI model.

Specs, contracts, and checks are plain files and deterministic commands, so no part of the workflow depends on a particular editor or vendor. Skills are portable instructions, not proprietary integrations.

**Violation test**: a step in the workflow functions only in one specific editor or with one specific model — the framework has acquired a lock-in.

## 3. Ubiquitous language

A shared vocabulary connects business, specs, and code so humans and AI mean the same thing by the same term.

spex defines its terms in the [glossary](glossary.md) and uses them consistently across specs, contracts, code, and docs, removing ambiguity and the translation errors it causes. A shared term used the same way everywhere is what lets a spec, a contract, and an implementation refer to one concept without restating it.

**Violation test**: the same concept is named two different ways across specs, code, and docs — every reader, human or agent, must translate, and some translate wrong.

## 4. Single source of truth

Every fact has exactly one authoritative source. Everything else references it: a native contract for an interface, a spec for a capability, a rule for a cross-cutting convention, `spex.yaml` for workspace layout, the glossary for term definitions.

Duplication has always been a source of bugs. With AI-assisted development it becomes critical: agents generate code fast, copies diverge faster, and a duplicated fact falls out of sync before anyone notices.

**Violation test**: the same fact exists in two places — one is already wrong, or soon will be.

**Agent-specific risk**: AI agents copy context between files for convenience ("let me add this to the agent rules too"). Every such copy is a future inconsistency. Reference, do not copy.

## 5. Parsimony

The minimum representation that preserves full semantics and enforceability.

The token budget is finite. Every token spent on filler, boilerplate, or irrelevant context is a token not available for substantive content — specs, contracts, code, examples. Parsimony is not about spending less; it is about allocating the budget where it has the highest impact on result quality.

Three requirements:

- **Minimality** — exclude everything that does not affect the agent's behavior in the current task: redundant explanations, repeated instructions, irrelevant fragments.
- **Sufficiency** — compression must not introduce ambiguity. The context stays complete enough for the agent to reconstruct goal, constraints, and result format.
- **Budget prioritization** — tokens saved on operational description are reallocated to substantive artifacts, which have the strongest impact on output quality.

**Violation test**: a context is parsimonious when no essential part can be removed without introducing ambiguity or degrading quality, and the whole contains no redundant or irrelevant element.

**Parsimony is not unreadability**: imperative, filler-free prose reads easily for a competent person who knows the domain. Parsimony removes noise, not meaning. The operational rules that implement it are captured in [`writing-style`](../templates/rules/writing-style.yaml).

## 6. Adaptable

spex fits the project, not the other way around. The framework itself has no built-in opinion on language, framework, or architecture — those come only from the rules and skills a project chooses to use. Templates ship ready-made rules and skills for common stacks, but none are mandatory: a project picks the ones that match its stack, or writes its own.

The same framework scales from a solo script to a large multi-team repo: no minimum ceremony is forced on a small project, no ceiling is placed on a large one, adoption is incremental.

**Violation test**: the framework core, a command, or a scaffolded default doesn't allow to change a specific language, framework, or all-or-nothing adoption instead of leaving it to the project's chosen rules and skills — a project that doesn't match breaks or is forced into unneeded ceremony.
