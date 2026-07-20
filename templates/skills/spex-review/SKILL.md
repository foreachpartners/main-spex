---
name: spex-review
description: Review the changes since a fixed point (commit, branch, tag, or merge-base) along two axes — governing rule specs (does the code follow cross-cutting conventions, rules and coding standards?) and functional specs (does the code match what the spec asked for?). Runs both reviews in parallel sub-agents and reports them side by side. Use when the user wants to review a branch, a PR, work-in-progress changes, or asks to "review since X".
---

Two-axis review of a diff — committed changes since a fixed point, or uncommitted working-tree changes:

- **Rule specs** — does the code conform to governing cross-cutting conventions and rules?
- **Functional specs** — does the code faithfully implement the functional specs?

Both axes run as **parallel sub-agents** so they don't pollute each other's context, then this skill aggregates their findings.

The functional spec should have been provided to you, ask user if not.

## Process

### 1. Pin the fixed point

If the user specified a fixed point — a commit SHA, branch name, tag, `main`, `HEAD~5`, etc. — use the committed diff. Otherwise, review **uncommitted changes** (staged and unstaged) against `HEAD`.

**Committed diff** (user gave a fixed point):

- Capture: `git diff <fixed-point>...HEAD` (three-dot, merge-base diff).
- Commit list: `git log <fixed-point>..HEAD --oneline`.
- Confirm the ref resolves: `git rev-parse <fixed-point>`.

**Uncommitted diff** (no fixed point):

- Capture: `git diff HEAD` (staged + unstaged vs `HEAD`).
- Commit list: "working tree (uncommitted)".
- If the working tree is clean, fail — nothing to review.

A bad ref or empty diff should fail here — not inside two parallel sub-agents.

### 2. Identify the specs source

Read provided functional spec and all rule specs for affected scope.

On top of whatever the repo documents, the Standards axis always carries the **smell baseline** below — a fixed set of Fowler code smells (_Refactoring_, ch.3) that applies even when a repo documents nothing. Two rules bind it:

- **The repo overrides.** A documented repo standard always wins; where it endorses something the baseline would flag, suppress the smell.
- **Always a judgement call.** Each smell is a labelled heuristic ("possible Feature Envy"), never a hard violation — and, like any standard here, skip anything tooling already enforces.

Each smell reads _what it is_ → _how to fix_; match it against the diff:

- **Mysterious Name** — a function, variable, or type whose name doesn't reveal what it does or holds. → rename it; if no honest name comes, the design's murky.
- **Duplicated Code** — the same logic shape appears in more than one hunk or file in the change. → extract the shared shape, call it from both.
- **Feature Envy** — a method that reaches into another object's data more than its own. → move the method onto the data it envies.
- **Data Clumps** — the same few fields or params keep travelling together (a type wanting to be born). → bundle them into one type, pass that.
- **Primitive Obsession** — a primitive or string standing in for a domain concept that deserves its own type. → give the concept its own small type.
- **Repeated Switches** — the same `switch`/`if`-cascade on the same type recurs across the change. → replace with polymorphism, or one map both sites share.
- **Shotgun Surgery** — one logical change forces scattered edits across many files in the diff. → gather what changes together into one module.
- **Divergent Change** — one file or module is edited for several unrelated reasons. → split so each module changes for one reason.
- **Speculative Generality** — abstraction, parameters, or hooks added for needs the spec doesn't have. → delete it; inline back until a real need shows.
- **Message Chains** — long `a.b().c().d()` navigation the caller shouldn't depend on. → hide the walk behind one method on the first object.
- **Middle Man** — a class or function that mostly just delegates onward. → cut it, call the real target direct.
- **Refused Bequest** — a subclass or implementer that ignores or overrides most of what it inherits. → drop the inheritance, use composition.

### 3. Spawn both sub-agents in parallel

Send a single message with two `Agent` tool calls. Use the `general-purpose` subagent for both.

**Rule spec sub-agent prompt** — include:

- The full diff command and commit list.
- The list of rule spec-source files you found in step 2 and smell baseline pasted in full — the sub-agent has no other access to it.
- The brief: "Report — per file/hunk where relevant — (a) every place the diff violates a documented specs: cite the rule (file + the rule); and (b) any baseline smell you spot: name it and quote the hunk. Distinguish hard violations from judgement calls — documented-standard breaches can be hard, but baseline smells are always judgement calls, and a documented repo standard overrides the baseline. Skip anything tooling enforces. Under 400 words."

**Functional spec sub-agent prompt** — include:

- The diff command and commit list.
- The path or fetched contents of the functional spec.
- The brief: "Report: (a) requirements the spec asked for that are missing or partial; (b) behaviour in the diff that wasn't asked for (scope creep); (c) requirements that look implemented but where the implementation looks wrong. Quote the spec line for each finding. Under 400 words."

If the spec is missing, skip the Spec sub-agent and note this in the final report.

### 5. Aggregate

Present the two reports under `## Rule specs` and `## Functional specs` headings, verbatim or lightly cleaned. Do **not** merge or rerank findings — the two axes are deliberately separate (see _Why two axes_).

End with a one-line summary: total findings per axis, and the worst issue _within each axis_ (if any). Don't pick a single winner across axes — that's the reranking the separation exists to prevent.

## Why two axes

A change can pass one axis and fail the other:

- Code that follows every rule but implements the wrong thing → **Rule pass, Functional Spec fail.**
- Code that does exactly what the issue asked but breaks the project's conventions → **Functional Spec pass, Rule spec fail.**

Reporting them separately stops one axis from masking the other.
