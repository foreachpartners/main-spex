# `spex-review-rule` skill

Rule-axis review of the changes since a starting point against the governing cross-cutting conventions and rules. Loaded when the user wants to check a diff, branch, or PR against the package's rules, or asks to "review rules since X".

## Acceptance criteria

### Resolve and gather

- MUST resolve the starting point: the one the user gave (a commit, branch, tag, or PR), otherwise `HEAD` for uncommitted changes or the merge-base with the base branch for a feature branch.
- MUST ask the user when the starting point is unclear.
- MUST use the packages the user provided, otherwise the packages affected by the changes.
- MUST run `spex rules --package <package-a> <package-b> --ai --output <tmp-path>` for those packages, then read `<tmp-path>` for the governing rules.
- MUST run `spex check --package <package-a> <package-b>` for those packages, and treat every reported violation as a hard finding, since the check already proved it.

### Smell baseline

- MUST carry, on top of whatever the package documents, the smell baseline below: a fixed set of Fowler code smells that applies even when a package documents nothing.
- MUST let a documented package standard win: where it endorses something the baseline would flag, suppress the smell.
- MUST treat each smell as a labelled heuristic ("possible Feature Envy"), always a judgement call and never a hard violation.
- MUST skip anything tooling already enforces.
- MUST match the diff against each smell, which reads what it is -> how to fix:
  - **Mysterious Name**: a function, variable, or type whose name does not reveal what it does or holds. -> Rename it; if no honest name comes, the design is murky.
  - **Duplicated Code**: the same logic shape appears in more than one hunk or file in the change. -> Extract the shared shape and call it from both; if the duplication is simple and the abstraction makes it less readable, leave it, since simplicity and readability always win.
  - **Feature Envy**: a method that reaches into another object's data more than its own. -> Move the method onto the data it envies.
  - **Data Clumps**: the same few fields or params keep travelling together (a type wanting to be born). -> Bundle them into one type and pass that.
  - **Primitive Obsession**: a primitive or string standing in for a domain concept that deserves its own type. -> Give the concept its own small type.
  - **Repeated Switches**: the same `switch`/`if`-cascade on the same type recurs across the change. -> Replace with polymorphism, or one map both sites share.
  - **Shotgun Surgery**: one logical change forces scattered edits across many files in the diff. -> Gather what changes together into one module.
  - **Divergent Change**: one file or module is edited for several unrelated reasons. -> Split so each module changes for one reason.
  - **Speculative Generality**: abstraction, parameters, or hooks added for needs the spec does not have. -> Delete it; inline back until a real need shows.
  - **Message Chains**: long `a.b().c().d()` navigation the caller should not depend on. -> Hide the walk behind one method on the first object.
  - **Middle Man**: a class or function that mostly just delegates onward. -> Cut it and call the real target directly.
  - **Refused Bequest**: a subclass or implementer that ignores or overrides most of what it inherits. -> Drop the inheritance and use composition.

### Review

- MUST make enforcing the coding rules and ensuring simplicity the primary objective: code is self-contained and locally clear, so anyone can fully understand the logic and impact of functions and methods strictly within their immediate context.
- MUST report, per file or hunk where relevant, every `spex check` violation, cited as reported.
- MUST report every place the diff violates a documented rule, citing the rule (file and the rule).
- MUST report any baseline smell it spots, naming it and quoting the hunk.
- MUST distinguish hard violations (`spex check` violations and documented-standard breaches) from judgement calls (baseline smells).
- MUST end with a one-line summary: total findings, and the worst issue (if any).
- MUST NOT report anything tooling enforces beyond what `spex check` already reported.
