# spex tutorial

A hands-on walkthrough from a plain multi-repo project to a spec-driven one. You
install the CLI, set up a spex workspace on the `pepperoni-pizza` example, fix
check and coverage findings, and run the functional and rule spec workflows
end to end.

By the end you will have recreated a project similar to `examples/pepperoni-pizza` from the
`examples/pepperoni-pizza-no-spex` starting point.

## Step 1 — Install spex

Follow the installation process in [cli.md](cli.md#install).

## Step 2 — Copy the example project

Copy `examples/pepperoni-pizza-no-spex` somewhere on your machine and rename the
copy to `pepperoni-pizza`. This folder is your **workspace root**, containing the
`api` and `frontend` repositories side by side.

## Step 2.1 — Rename folders to follow RS-NAMING-CONVENTION

Repository directory naming is <project-name>-<repository-short-name>

- `api` becomes `pepperoni-pizza-api`
- `frontend` becomes `pepperoni-pizza-frontend`

## Step 3 — Set up spex and workspace

Run `spex init` in terminal.

Open the workspace to work across all repositories at once and integrate with an AI coding agent:

- Cursor/VSCode: open workspace from file `pepperoni-pizza-spex/pepperoni-pizza.code-workspace`
- Cursor agentic: open workspace file in editor mode first, then pick it in agentic mode.
- Claude CLI: run `claude --add-dir ../pepperoni-pizza-api ../pepperoni-pizza-frontend ..` from `pepperoni-pizza-spex` repo
- Zed: add project, pick all workspace repos (folders)

Ask AI to init git in each repository and commit with `chore: initial commit`.

Commit as you complete each step so your progress is easy to review and roll back.

Run your first skill `/spex-setup` in AI chat window to complete setup.

## Step 4 — Fix everything `spex check` reports

Run `spex check` in terminal from anywhere in the workspace and work
through the findings until it exits cleanly.

## Step 5 — Functional spec workflow

Drive a real feature from PRD to verified code. Start with the PRD-level
statement of what the product should do:

> show customers pizza delivery time based on number of pizzas ordered, other ongoing orders and today workers

### 5.1 Create the functional spec

There is no person who requested this feature with whom you can clarify details, so decide yourself how it should behave for the tutorial.

Hand the PRD to the skill — it breaks the PRD into one or more specs:

```text
/spex-functional-spec show customers pizza delivery time based on number of pizzas ordered, other ongoing orders and today workers
```

The skill interviews you, proposes the split when the PRD covers more than one capability, then writes a schema-valid spec per capability. A spec captures one capability, not one repository. Review the spec and confirm its `status` is `planned`.

### 5.2 Derive the contract

```text
/spex-contract <spec-id>
```

The agent reads the spec, `api` OpenAPI contract and updates it to match changes.

### 5.3 Plan

```text
/spex-plan <spec-id>
```

The agent reads the spec, contract, code and proposes an implementation plan.

Tip: you can combine skills in the future `/spex-functional-spec /spex-contract /spex-plan`

### 5.4 Implement

```text
/spex-implement <spec-id>
```

The agent executes the plan tasks for each step in parallel.

### 5.5 Verify with check and coverage

```sh
spex check
spex coverage
```

Coverage should now show your new spec as `✓`.

### 5.6 Review

```text
/spex-review <spec-id>
```

## Step 6 — Rule spec workflow

Rule specs capture cross-cutting conventions that apply repo-wide. Create one
for the API:

```text
/spex-rule-spec the API must not hardcode network bind addresses or ports — they must be read from configuration/environment
```

The skill interviews you, proposes a `regexp` check that flags binding to a
hardcoded address like `127.0.0.1:3000`, and writes a schema-valid
`pepperoni-pizza-spex/rs-specs/rs-*.yaml`. Make sure the new spec is in the `api` repository's
`rule_spec_ids` list in `spex.yaml` so the check runs against it.

Then run `spex check` — it should flag the hardcoded bind address in `api`. You can fix it directly for
small change or create a plan if compliance with the new rule spec requires many changes.

## Step 7 — Generate a completion report

Have your AI agent summarize the result:

```text
Create a new file `pepperoni-pizza-spex/tutorial-completion.md` that documents the result of this tutorial. In it:

1. Run `spex check` and paste its full output under a "spex check" heading.
2. Run `spex coverage` and paste its full output under a "spex coverage" heading.
3. Rate created functional spec and implementation for this requirement: "show customers pizza delivery time based on number of pizzas ordered, other ongoing orders and today workers". User was free to decide on details.
3. Rate created rule spec for this requirement: "the API must not hardcode network bind addresses or ports — they must be read from configuration/environment"
```

Review and commit it.

## 🎉 Congratulations!

You did it! You took a plain multi-repo project and turned it into a fully
spec-driven workspace — installing the CLI, wiring up Git per repository, and
driving real features from spec to verified code.
