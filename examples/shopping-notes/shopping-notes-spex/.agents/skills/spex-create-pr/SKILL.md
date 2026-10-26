---
name: spex-create-pr
description: Create a pull request from the current branch, filled from the PR template and labeled with its risk. Use when the user wants to open, create or raise a PR.
---

Open a pull request whose description tells a reviewer why the change exists, what changed, and how carefully to review it.

## Process

### 1. Find the repos

Find each repo with changes to propose (`spex info` lists the packages and their paths). A change spanning several repos becomes one PR per repo: do steps 2 to 5 for each.

Stop and ask when a repo is on its default branch, or has uncommitted changes that belong in the PR.

### 2. Pick the template

Use the repo's own PR template when it has one, otherwise [`references/pull-request-template.md`](references/pull-request-template.md).

### 3. Label the risk

Rate the change from its diff as Low, Medium or High, using the levels in the Risk section of [`references/pull-request-template.md`](references/pull-request-template.md), whichever template step 2 picked. The reviewer can then tell whether it can merge once checks are green or needs careful review.

A change to an area the package's skill marks as sensitive is at least Medium. When unsure, take the higher level.

### 4. Fill the template

Write the body from the user's request, the commits, the diff and the specs they trace to.

- State why the change exists and the gist of what changed, in the sentences the template asks for.
- List a ticket only when the user, the branch name or a commit names it. When a change spans several repos, list the related PRs.
- Keep a template section only when it has real content. Remove every other optional section and every comment placeholder.
- When the change is visible in the UI, ask the user for the screenshots or recording.
- Fill the `## Risk` section with the level from step 3 and one line naming what in the change drives it, in the form of the template's examples. When the template has no Risk section, add one after its first section.
- Title: the commit message format of the package's `commit-message-format` rule when it lists one, otherwise one short imperative sentence.

The body is done when no placeholder or empty section is left, the Risk section is present, and every claim is backed by the commits or the user's words.

### 5. Confirm and create

Show the user each PR's repo, base, head, title and body, and wait for approval: pushing and opening a PR is visible to others. Then push the branch if needed and create the PR with `gh pr create --assignee @me`.

## Report

The URL of each PR created, and any repo skipped with the reason.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
