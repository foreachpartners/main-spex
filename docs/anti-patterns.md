# Anti-patterns

Engineering hygiene rules for working in spex. Each one is the rule, the reason it exists, and how to apply it. They follow from the [foundation principles](framework-principles.md); see [glossary.md](glossary.md) for terms and [specification.md](specification.md) for how the framework behaves.

## Bypassing safety checks

**Rule:** Never use `--no-verify`, `--no-gpg-sign`, or any flag that skips a hook, lint, or signing step. Never make `spex check` pass by weakening the check that caught the violation. Never baseline a freshly introduced error with `--fill-preexisting-errors` or `checks_preexisting_errors`.

**Why:** Checks are the deterministic layer that proves a change does what its spec demands and nothing else broke. They give the same answer every time, catching what both the AI and the human overlook. Bypassing a check re-creates the conditions for the break it was added to prevent. Baselining exists to adopt a check going forward over pre-existing debt, not to wave through new errors.

**How to apply:** When a check fails, fix the root cause. If the check itself is wrong, fix it as a separate change with justification — and, for an engineering-spec check, update the spec's acceptance criterion it proves. Reserve `--fill-preexisting-errors` for adopting a new check over errors that already existed.

## Silencing tests

**Rule:** Never mask `test.skip()` with early returns. Never delete a failing test to make a run green. Never lower assertion strictness "until we figure it out".

**Why:** Test execution is a factual pass/fail result and a signal. The signal is information. Deleting the signal does not delete the underlying problem; it only delays discovery until production.

**How to apply:** When a test fails, debug it. Seed missing data, configure missing features, fix rendering issues — find solutions, not excuses. If the test contradicts the spec's acceptance criteria, fix the test (and note why) in a separate commit; if the spec is wrong, change the spec first.

## Dismissing errors as "not ours"

**Rule:** Never dismiss compile / lint / audit errors as "not our code", "generated", "vendored", or "deployment-only". If it fails in your build, you own the fix.

**Why:** Build artifacts, generated code, and vendored deps are part of the system you ship. The line between "yours" and "not yours" exists in code review etiquette, not in the build pipeline. The build doesn't care.

**How to apply:** Trace every error to a root cause you can fix. Root causes are often: missing config, stale generated artifacts (regenerate them), version mismatch (align), or a bug upstream (file a fix or a vendored patch).

## Backwards-compat shims in pre-release code

**Rule:** During pre-release and major refactors, delete obsolete code entirely. No `// removed` comments, no proto `reserved` markers, no deprecated wrappers, no compatibility shims for callers that don't exist yet.

**Why:** Compat shims are insurance against a class of risk that doesn't exist before release. Pre-release code has no external consumers; nothing needs to be preserved. Shims you write "just in case" become permanent residue once written.

**How to apply:** Delete cleanly. Trust the type checker, the tests, and `spex check` to flag actual breakage. Add compat shims only when an external consumer actually exists.

## Premature abstractions

**Rule:** Don't add features, refactor, or introduce abstractions beyond what the spec and plan require. Don't design for hypothetical future requirements. Three similar lines is better than a premature abstraction.

**Why:** Simplicity is a constraint on the work, not only on the tool. Hypothetical requirements are wrong twice as often as concrete ones. Every abstraction has a cost (cognitive load, indirection, harder grep). The cost is paid forever; the benefit may never materialize.

**How to apply:** Implement what the acceptance criteria demand. When the third concrete instance of a pattern appears, then abstract — and refactor the prior two into the new abstraction. Not before.

## Defensive code at internal boundaries

**Rule:** Don't add error handling, fallbacks, or validation for scenarios that can't happen. Trust internal code and framework guarantees. Validate only at system boundaries (user input, external APIs).

**Why:** Defensive code at internal boundaries adds noise that obscures real error handling at the actual boundaries. It also masks bugs: a `try/except: pass` at an internal call hides the real failure mode.

**How to apply:** Validate at the boundary the contract defines. Trust internally. If you find yourself adding a check "just in case", ask: what code path produces the bad value? If you can't name one, the check is wrong.

## Comments that explain WHAT

**Rule:** Default to writing no comments. Don't explain WHAT the code does — well-named identifiers do that. Don't reference the current task, fix, or callers ("used by X", "added for Y", "handles case from issue #123").

**Why:** A shared, ubiquitous vocabulary and clear names carry the meaning. WHAT comments rot the moment the code changes; caller references rot the moment a caller is added or removed. The grep for "what calls this" is more reliable than a comment claiming to know.

**How to apply:** Write a comment only when WHY is non-obvious: a hidden constraint, a subtle invariant, a workaround for a specific bug, behavior that would surprise a careful reader. If removing the comment wouldn't confuse a future reader, don't write it.

## Inventing acceptance criteria

**Rule:** Implement, test, and check only what the spec's acceptance criteria state. Never invent a requirement no spec contains, and never silently widen a criterion while implementing it.

**Why:** Specs eliminate guesswork: they are the explicit criteria the human verifies against. A criterion an agent invents was approved by no one, so it escapes human control and bloats the change beyond the spec.

**How to apply:** When the work needs a rule the spec lacks, stop and add it with `spex-create-capability-spec` (or `spex-create-engineering-spec`), get it approved, then implement. Load the governing specs with `spex spec-context` so you work from their actual text, not your memory of it.

## Skipping the approval gate

**Rule:** Each artifact is approved by a human before the next is generated: spec before contract, contract before plan, plan before code and tests. Never jump ahead — never write a contract for an unapproved spec or implement against an unapproved plan.

**Why:** The human owns every decision by approving the artifact that records it. Skipping a gate ships a decision no human reviewed, which is how an unreviewed change compounds into mess and erodes trust in the codebase.

**How to apply:** Generate one artifact, present it, wait for approval, then generate the next. When a downstream skill finds an upstream artifact missing, it stops and creates that artifact first rather than proceeding on an assumption.

## Marking a spec active prematurely

**Rule:** A capability spec becomes `active` ONLY when its acceptance criteria are fully implemented and verification passes — `spex check` exits 0 on the affected repositories and the tests pass. If the spec has moved ahead of the code, leave its unfinished acceptance criteria unchecked (`[ ]`) so `spex coverage` flags them. Never report "done" when a required step was skipped.

**Why:** Trust degrades exponentially. A single false `active` forces the human to re-verify everything you claim, forever after, and breaks the guarantee that an `active` spec is a satisfied one.

**How to apply:** If verification fails, keep the spec `planned`, leave the unimplemented acceptance criteria `[ ]`, and state exactly what passed and what didn't. The honest "blocked on X" is always more valuable than a false "active".

## Destructive actions as a shortcut

**Rule:** Never use destructive operations (`rm -rf`, force push, hard reset, branch delete, drop table) to make an obstacle go away. Never bypass safety prompts to "just get it done".

**Why:** Destructive actions are irreversible. The cost of pausing to confirm is low; the cost of an unwanted destruction can be hours of lost work or a production incident.

**How to apply:** When you encounter unexpected state (unfamiliar files, branches, lock files, configs), investigate before deleting. Resolve merge conflicts; don't discard. Find what holds a lock; don't delete it. When in doubt, ask before acting.

## Long timeouts hiding bugs

**Rule:** Keep test timeouts short and explicit. Don't extend a timeout to make a flaky test pass. Don't wrap a slow operation in `sleep()` to "give it time".

**Why:** Long timeouts conceal the actual failure mode. A test that needs 30 seconds isn't slow — it's broken in a way that happens to converge eventually. The bug is real; the timeout just delays your discovery.

**How to apply:** Set timeouts at 2-3× the realistic upper bound. When a timeout fires, debug what made the operation slow — don't extend the timeout. The exception is genuinely-slow operations (network calls to slow upstreams) where the timeout reflects reality.

## Over-eager reformatting

**Rule:** A bug fix doesn't need surrounding cleanup. A one-shot operation doesn't need a helper function. Don't refactor adjacent code that isn't part of the task.

**Why:** Bundled cleanup expands the diff, slows review, and increases the chance of an unrelated regression slipping in. Reviewers reading "fix off-by-one in sorting" don't expect the function next to it to be rewritten.

**How to apply:** Make the smallest change that satisfies the acceptance criteria. If you notice surrounding code that needs cleanup, note it — propose a separate spec or plan. Keep the change tight.

## Duplicating a fact instead of referencing

**Rule:** Every fact has exactly one authoritative source; everything else references it. Never copy an interface into markdown, a behavior into code comments or plans, a convention into agent rules or READMEs, or a term's definition into a second doc.

**Why:** A duplicated fact is already wrong, or soon will be. Agents copy context between files for convenience ("let me add this to the agent rules too"), and copies diverge faster than anyone notices.

**How to apply:** Keep the interface in `<workspace-root>/<project-name>-spex/contracts/`, the behavior in `<workspace-root>/<project-name>-spex/cs-specs/<spec-id>.yaml`, the convention in `<workspace-root>/<project-name>-spex/es-specs/<spec-id>.yaml`, the layout in `spex.yaml`, and terms in `docs/glossary.md`. Reference these sources; do not copy them.

## Changes without traceability

**Rule:** Every behavioral change MUST trace to a spec through a Spec ID annotation at the layer boundary: `@spec impl <spec-id>` and `@spec test <spec-id>` in code, `x-spec: <spec-id>` in OpenAPI. Annotate once at the boundary, never nested. Never leave a behavioral change orphaned, and never reference a Spec ID with no spec file.

**Why:** The Spec ID is the unit of traceability. `spex coverage` computes coverage from these annotations and exits non-zero when an active capability spec is missing a required layer. Without the annotation the spec appears unimplemented, the audit fails, and bottom-up tracing (test → code → contract → spec) is impossible.

**How to apply:** Annotate at the boundary as you implement, using the spec the work satisfies. Capability-spec traceability is required; engineering-spec traceability is optional. If the change has no spec, stop and create one with `spex-create-capability-spec` first.
