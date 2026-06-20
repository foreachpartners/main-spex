# spex tutorial

A hands-on walkthrough from a plain multi-repo project to a spec-driven one. You
install the CLI, set up a spex workspace on the `pepperoni-pizza` example, fix
check and coverage findings, and run the business and technical spec workflows
end to end.

By the end you will have recreated similar to `examples/pepperoni-pizza` project from the
`examples/pepperoni-pizza-no-spex` starting point.

## Step 1 — Install spex

Follow the installation process in [cli.md](cli.md#install).

## Step 2 — Copy the example project

Copy `examples/pepperoni-pizza-no-spex` somewhere on your machine and rename the
copy to `pepperoni-pizza`. This folder is your **workspace root**, containing the
`api`, `frontend`, and `contracts` repositories side by side.

## Step 3 — Set up spex

Follow the **Quickstart Setup** in [README.md](../README.md#setup): run `spex init`,
open the workspace in your editor, and run the `/spex-setup` skill.

## Step 4 — Put each repository under Git

spex traces one Spec ID across many repositories, so each workspace repository is
its own Git repository. Initialize Git and make a baseline commit for each repository.

Commit as you complete each step so your progress is easy to review and roll back.

## Step 5 — Fix everything `spex check` reports

Run [`spex check`](cli.md#spex-check) from anywhere in the workspace and work
through the findings until it exits cleanly.

## Step 6 — Business spec workflow

Drive a real feature from PRD to verified code. Start with the PRD-level
statement of what the product should do:

> Customers can see the full pizza menu (id, name, description, price) before
> ordering.

### 6.1 Create the business spec(s)

Hand the PRD to the skill — it breaks the PRD into one or more capabilities and
writes a spec for each:

```text
/spex-create-biz-spec Customers can see the full pizza menu (id, name, description, price) before ordering
```

The skill interviews you, proposes the split when the PRD covers more than one
capability, then writes a schema-valid spec per capability. A spec captures one
capability, not one repository: this PRD is the single capability "customers can
see the menu," so it produces one spec under `spex/biz-specs/` — even though its
`scope` spans `contracts`, `api`, and `frontend`. The agent chooses the `BIZ-`
Spec ID; note it down, since later steps reference it. Review the spec and confirm
its `status` is `active`.

### 6.2 Plan and execute

```text
/spex-create-plan list pizza menu
/spex-execute-plan list pizza menu
```

The agent adds the `GET /pizzas` operation to the OpenAPI contract, implements
the handler in `api`, wires it into the frontend, and adds tests — each tagged
with the matching `@spec` annotation — running `spex check` as it goes.

### 6.3 Verify with check and coverage

```sh
spex check
spex coverage
```

Coverage should now show your new spec as `✓`.

## Step 7 — Technical spec workflow

Technical specs capture cross-cutting conventions that apply repo-wide. Create one
for the API:

```text
/spex-create-tech-spec the API must not hardcode network bind addresses or ports — they must be read from configuration/environment
```

The skill interviews you, proposes a `regexp` check that flags binding to a
hardcoded address like `127.0.0.1:3000`, and writes a schema-valid
`spex/tech-specs/TECH-*.yaml`. Make sure the new spec is in the `api` repository's
`tech_specs` list in `spex.yaml` so the check runs against it. Then run `spex
check` — it flags the hardcoded bind address in `api` — resolve the finding by
reading the address from the environment, and commit.

## Step 8 — Generate a completion report

Have your AI agent summarize the result:

```text
Create a new file `spex/tutorial-completion.md` that documents the result of this tutorial. In it:

1. Run `spex check` and paste its full output under a "spex check" heading.
2. Run `spex coverage` and paste its full output under a "spex coverage" heading.
3. Under a "Rating" heading, rate both results out of 10 and briefly justify each score: are all checks passing, and is every active business spec fully covered (impl, test, contract)?
4. Under a "Next steps" heading, list any remaining findings or coverage gaps and what to do about them.

Keep it concise and factual — only report what the commands actually printed.
```

Review and commit it:

## 🎉 Congratulations!

You did it! You took a plain multi-repo project and turned it into a fully
spec-driven workspace — installing the CLI, wiring up Git per repository, and
driving real features from spec to verified code.
