# spex tutorial

A simple tutorial where you learn spex on a simple project from scratch.

You will create a `shopping-notes` application that lets a user note down shopping items so they don't forget to buy them when visiting a store. It will have a simple frontend (react SPA) and api (node) packages.

By the end you will have recreated a project similar to `examples/shopping-notes` from the simple empty api and frontend
`examples/shopping-notes-no-spex` starting point.

## Install spex

Install [Rust](https://rust-lang.org/tools/install/)

```sh
cargo install main-spex
```

## Copy the example project

Copy `examples/shopping-notes-no-spex` somewhere on your machine and rename the
copy to `shopping-notes`. This folder becomes your **project root**, containing the
`shopping-notes-api` and `shopping-notes-frontend` packages side by side.

### Get familiar with project

Get familiar with our base project, run api and frontend, run tests.

### Setup git repos

Ask AI to init git repositories for api and frontend and commit existing files.

## Set up spex and workspace

Run `spex init` in terminal from the `shopping-notes` project root and pick the `multi-repo` layout when asked. It generates `shopping-notes-spex` next to the packages.

### Get familiar with spex

Get familiar with generated base `shopping-notes-spex` repository, we will cover each part in details later.

### Setup spex git repo

Ask AI to init git in `shopping-notes-spex` repository and commit existing files.

### Setup workspace

Open the workspace to work across all packages at once and integrate with an AI coding agent:

- Cursor/VSCode: open workspace from file `shopping-notes-spex/shopping-notes.code-workspace`
- Cursor agentic: open workspace file in editor mode first, then pick it in agentic mode.
- Claude CLI: run `claude --add-dir ../shopping-notes-api ../shopping-notes-frontend` from `shopping-notes-spex` repo
- Zed: add project, pick all packages (folders)

The workspace file lists only `shopping-notes-spex` for now, the `/spex-setup` skill below adds the other packages to it.

### Skills

If you did everything correctly you should be able to run spex AI skills using you coding agent or cli.

**Agent Skill** (skill) — a modular, reusable capability that packages specialized instructions an AI agent loads on demand when a request matches the skill's description. Defined as a folder with a `SKILL.md` entry point, following the [Agent Skills](https://agentskills.io/) spec. In spex, a skill is the instructions an AI agent follows to do the job, taking all applicable rules and implementing a spec.

Skill consists of metadata and body, metadata is always loaded in when you start new agent chat for all skills,
body is loaded on demand, which helps to save tokens and not pollute context, therefore improving output results:

Example `spex-setup/SKILL.md`:

```
description: skill metadata description, always loaded, example: `completes spex setup, use it when user wants to finish spex setup`
---

skill body wih instructions, loaded only when user invokes skill `/spex-setup` or metadata matches user request,
for example if user says `let's setup spex` - it will match skill metadata description and AI agent will read skill body
```

The skill that we will run `shopping-notes-spex/.agents/skills/spex-setup/SKILL.md` is instruction for AI to
update `spex.yaml` (which is main spex configuration file for your project) and:

- add the packages (`shopping-notes-api`, `shopping-notes-frontend`) to `spex.yaml` and the workspace file
- add descriptions for your project and each package
- pick rules from bundled with spex that applicable to each package and remove unused
- generate main skills for each package with metadata that requires them to be loaded when AI works on any package. Skills will describe each package and we will update them from time to time to help AI understand how we want it to work there, see `shopping-notes-api/.agents/skills/shopping-notes-api/SKILL.md`

Run your first skill `/spex-setup` in AI chat window or agent CLI to complete setup. Check what agent generated, review it, fix if needed and commit.

## Rules

Run `spex check` in terminal from anywhere in the project, you will see `dependencies-rules/npm-floating-version` errors in api.

**Rule** — describes _how the code and artifacts are built_: one coherent cross-cutting convention or rule (style, structure, security, traceability, documentation), with its individual rules captured as acceptance criteria proven by `spex check`. Not tied to a single feature — it applies across whole packages.

Each package has rules that it needs to follow, applicable rules for each package are defined in `spex.yaml` and one of the rules for api is
reported `dependencies-rules`. You can open `shopping-notes-spex/rules/dependencies-rules.yaml` and check it description and acceptance_criteria package must follow with deterministic `checks`. Our error comes from `npm-floating-version` which requires `package.json dependencies must pin an exact version, not a range`. We can fix it by opening `/shopping-notes-api/package.json` and removing `^`.

Rules are a way to ensure AI generates code in a way we want. Rules applicable to a package are loaded in AI context when it works on this package and if possible they also have deterministic checks which we can validate by running `spex check`.

Commit your fixes, run `spex check` again and ensure there is no more errors.

## Specs

### Api healthcheck

**Spec** (specification) is a single structured requirement, verifiable and precise enough for engineers and AI agents to follow directly, describing _a capability the system must provide_.

Specs are short, fast to generate and review, ironing out details and creating a single source of truth for how our application works. It's like a waterfall from spec down to implementation, we can always point agent to spec and say `follow it`, `check implementation against it` or do it ourselves. Specs helps us to quickly reason about application behaviour, removes uncertainty of how things should be and help us to move fast.

Let's create a simple specs for api, our technical lead wants a healthcheck for api so that other tools, services and frontend could check if api is working fine.

Let's invoke a skill for spec creation for this task, ask AI `/spex-spec healthcheck endpoint for api, id api-healthcheck`.

Agent will ask you details, feel free to pick what you think right for example, for real project you may want to collaborate with project manager, technical lead, domain experts or other involved parties on things that you can't decide yourself.

Review created spec and edit it if needed.

Run `spex coverage` - a command that shows you a current state of project and ensures that each spec has contract if applicable, implementation and tests for each acceptance criterion. For now you will see only our spec `api-healthcheck` as planned.

Commit the change.

#### Contract

Now let's implement `api-healthcheck` spec, we start with OpenAPI contract.

**Contract** — derived from a spec, if applicable. The interface to build against: OpenAPI, `.proto`, GraphQL SDL, AsyncAPI, or a Figma mockup for UI. AI implements from spec and contract.

We implement contract first because it's fast to generate and easy to review it, narrowing down how AI should implement our spec. Without contract we would need to review full implementation and do larger refactoring in case AI implements it not in a way we want it. With contracts we are saving time and tokens. Also it's easy to refactor implementation not changing it public surface, but requires a breaking change otherwise.

Let's run a skill to create a contract for our spec `/spex-contract api-healthcheck in shopping-notes-spex/contracts/api.openapi.yaml and switch spec status to in-progress`

Review generated contract, edit if needed and commit changes.

Run `spex coverage`, you will see that our spec now `in-progress`, meaning we are working on it and it has all acceptance criteria (ac) not implemented.

#### Traceability

Try to remove `x-spec: "@spec contract api-healthcheck"` in `api.openapi.yaml` and run `spex coverage`, you will see `missing: contract` gap.

**Traceability** (tracing, trace) — the `@spec` annotations stitching together spec, contracts, implementations, and tests, used to ensure coverage and to look up any of spec, contract, implementation, or test from any other, in either direction.

Tracing is like a thread going through from spec to its artifacts in format of comment `@spec <type> <spec-id>` like `@spec contract api-healthcheck`. Trace helps AI and us to quickly find code and test related to spec and the other way around, saving tokens, focusing context for better output, and speeding up work, while deterministic `spex coverage` check ensures nothing is missing, spec has contract and **each** spec acceptance criterion has implementation and test. You may know that AI sometimes forget to implement or test some small part of our request, one ac, with `spex coverage` this problem is no longer exists.

Put back trace for contract and let's continue with implementation.

#### Implement

Like with everything else, we have a skill for this, run `/spex-implement api-healthcheck`. Skill helps to automate manual steps, pulls all necessary context for the task - spec, applicable rules, package skill `shopping-notes-api/SKILL.md` and auto-updates documentation in this skill if needed.

#### Review

While our change is small and we can manually review it, in case of bigger changes we would want agent to review implementation, let's try it out, run `/spex-review api-healthcheck api package implementation`.

Review skill will spawn two subagents in parallel where first will review if implementation follows rules, second will review if implementation follows spec. Review skill itself delegates to `spex-review-rule` and `spex-review-spec` skills for each subagent. It's like a constructor where you can combine pieces to achieve desired process, we will see it later using `spex-assembly-line` skill.

#### Test

Now it's time to add tests for our implementation, run `/spex-test api-healthcheck`. We run tests last to ensure AI is focused on implementation review and not distracted by test code, which is often larger than the implementation itself. As you may know, saving context is important because the larger the context, the more often AI hallucinates and produces worse results — at the time of writing, keeping context under ~200_000 tokens is recommended.

Now our first spec is completed, running `spex check` should pass and `spex coverage` should show our spec active with no gaps.

Commit changes in api and spex.

### Frontend not found page

Now our project manager want use to have not found page in frontend in case user navigates to non-existing page.

Let's create a spec for it `/spex-spec frontend not found page with link back to main page, id fe-not-found-page, contract figma`. Commit changes.

Now let's create design mockup contract for our page, as we have a bad taste let's ask our designer, here is what we got: `https://www.figma.com/design/MSDzHDhkLsANf0jvFbAUTR/Not-found-page---shopping-notes?node-id=1-2`

Let's write it down `/spex-contract fe-not-found-page, change status to in-progress, place in shopping-notes-frontend/contracts/figma.md, figma https://www.figma.com/design/MSDzHDhkLsANf0jvFbAUTR/Not-found-page---shopping-notes?node-id=1-2`. As our contract is not shared and belongs to frontend we place it there. Commit changes.

If you remember next we would need to do implementation, review and testing. There is a orchestrator skill that will do all of this for you `/spex-assembly-line fe-not-found-page` by spawning agent for each stage and running loop review and fix until clean or a 3-pass cap. The only thing that will be left for us is to review and commit changes.

## Shopping notes application

You learned all the basics of spex and now can proceed on your own. Here is the document for your task:

### Product Requirements

Shopping notes application that lets a user note down shopping items so they don't forget to buy them when visiting a store.

- User can add an item to the shopping list.
- User can view the list of items.
- User can easily mark an item as bought.
- User can delete an item from the list.
- User can clear all items from the list.

How do you split the work is up to you, but I suggest you to split frontend and api specs and also make each capability a separate spec, then create all specs, all contracts and implement each or all at once.

## Generate a completion report

Ask your AI agent summarize the result:

```text
Create a new file `shopping-notes-spex/tutorial-completion.md` that documents the result of this tutorial. In it:

1. Run `spex check` and paste its full output under a "spex check" heading.
2. Run `spex coverage` and paste its full output under a "spex coverage" heading.
3. Rate created specs, rules, implementation, tests, tracability and overall project
```

Commit it, if needed publish repository as private and share results with your manager to confirm you completed tutorial.

## 🎉 Congratulations!

You did it! Now you can use spex in real project. And don't forget to contribute good rules and skills and other improvements back to spex for everyone to use.
