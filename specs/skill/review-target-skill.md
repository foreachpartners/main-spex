# Review target

How the review skills ([`spex-review`](spex-review-skill.md), [`spex-review-rule`](spex-review-rule-skill.md), [`spex-review-spec`](spex-review-spec-skill.md)) pin the diff they review, so a bad ref or an empty diff fails before the review starts.

## Acceptance criteria

- MUST review the committed diff when the user specified a fixed point: a commit SHA, branch name, tag, `main`, `HEAD~5`, etc.
- MUST capture the committed diff with `git diff <fixed-point>...HEAD` (three-dot, merge-base diff), list its commits with `git log <fixed-point>..HEAD --oneline`, and confirm the ref resolves with `git rev-parse <fixed-point>`.
- MUST otherwise review the uncommitted changes, staged and unstaged, against `HEAD`.
- MUST capture the uncommitted diff with `git diff HEAD`, and name its commit list "working tree (uncommitted)".
- MUST review the packages listed in the spec's `packages`.
- MUST fail at pinning, rather than partway through the review, on a bad ref or an empty diff.
- MUST fail when the working tree is clean and no fixed point is given, since there is nothing to review.
