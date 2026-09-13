---
name: spex-review-rule
description: Review the changes since a fixed point (commit, branch, tag, or merge-base) against governing cross-cutting rules and coding standards. Use when the user wants to check a diff, branch, or PR against the repo's rules, or asks to "review rules since X".
---

Rule-axis review of a diff — committed changes since a fixed point, or uncommitted working-tree changes — against governing cross-cutting conventions and rules.

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

A bad ref or empty diff should fail here rather than partway through the review.

### 2. Identify the rules source

Run `spex rules --repo <repo-a> <repo-b> --ai --output <tmp-path>` for the affected scope, then read `<tmp-path>` for the governing rules.

On top of whatever the repo documents, the review always carries the **smell baseline** below — a fixed set of Fowler code smells that applies even when a repo documents nothing. Two rules bind it:

- **The repo overrides.** A documented repo standard always wins; where it endorses something the baseline would flag, suppress the smell.
- **Always a judgement call.** Each smell is a labelled heuristic ("possible Feature Envy"), never a hard violation — and, like any standard here, skip anything tooling already enforces.

Each smell reads _what it is_ -> _how to fix_; match it against the diff:

- **Mysterious Name** — a function, variable, or type whose name doesn't reveal what it does or holds. -> rename it; if no honest name comes, the design's murky.
- **Duplicated Code** — the same logic shape appears in more than one hunk or file in the change -> extract the shared shape, call it from both. If duplication is simple and abstraction makes it less readable -> leave it as is, simplicity and redability always wins.
- **Feature Envy** — a method that reaches into another object's data more than its own. -> move the method onto the data it envies.
- **Data Clumps** — the same few fields or params keep travelling together (a type wanting to be born). -> bundle them into one type, pass that.
- **Primitive Obsession** — a primitive or string standing in for a domain concept that deserves its own type. -> give the concept its own small type.
- **Repeated Switches** — the same `switch`/`if`-cascade on the same type recurs across the change. -> replace with polymorphism, or one map both sites share.
- **Shotgun Surgery** — one logical change forces scattered edits across many files in the diff. -> gather what changes together into one module.
- **Divergent Change** — one file or module is edited for several unrelated reasons. -> split so each module changes for one reason.
- **Speculative Generality** — abstraction, parameters, or hooks added for needs the spec doesn't have. -> delete it; inline back until a real need shows.
- **Message Chains** — long `a.b().c().d()` navigation the caller shouldn't depend on. -> hide the walk behind one method on the first object.
- **Middle Man** — a class or function that mostly just delegates onward. -> cut it, call the real target direct.
- **Refused Bequest** — a subclass or implementer that ignores or overrides most of what it inherits. -> drop the inheritance, use composition.

### 3. Run automated checks

Run `spex check --repo <repo-a> <repo-b>` for the affected scope. Every reported violation is a hard finding — the check already proved it, not a judgement call.

### 4. Review

The primary objective of a review is to enforce coding rules and ensure simplicity. Code should be self-contained and locally clear, allowing anyone to fully understand the logic and impact of functions and methods strictly within their immediate context.

Against the diff captured in step 1, report — per file/hunk where relevant:

- Every `spex check` violation from step 3: cite it as reported.
- Every place the diff violates a documented rule: cite the rule (file + the rule).
- Any baseline smell you spot: name it and quote the hunk.

Distinguish hard violations from judgement calls — `spex check` violations and documented-standard breaches are hard, but baseline smells are always judgement calls, and a documented repo standard overrides the baseline. Skip anything tooling enforces beyond what `spex check` already reported.

End with a one-line summary: total findings, and the worst issue (if any).
