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

Rate the change from its diff as Low, Medium or High, using the levels in the Risk section of [`references/pull-request-template.md`](references/pull-request-template.md), whichever template step 2 picked.

A change to an area the package's skill marks as sensitive is at least Medium. When unsure, take the higher level.

### 4. Fill the template

Write the body from the user's request, the commits, the diff and the specs they trace to.

- Put the level from step 3 in the Risk section. When the template has no Risk section, add one after its first section.
- List a ticket only when the user, the branch name or a commit names it. When a change spans several repos, list the related PRs.
- For a UI change, capture it before and after yourself: screenshots when a still image shows the change, a recording when it is a multi-step flow. When the setup cannot capture them, skip them and report it.
- Title: one short imperative sentence.

The body is done when no placeholder or empty section is left and every claim is backed by the commits or the user's words.

### 5. Create

Push the branch if needed and create the PR with `gh pr create --assignee @me`.

## Report

The URL and title of each PR created, and any screenshot or recording that could not be captured.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
