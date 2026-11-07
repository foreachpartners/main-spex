---
name: <package-name>
description: Information, rules, commands and other <package-name> package context. Keep it up to date with important changes. MUST be loaded before reading, writing, testing, reviewing, or running any code in that package.
---

# General

A package skill is a short orientation for an agent
entering the package, not a changelog or feature catalog. Specs, contracts,
rules and code are the sources of truth for what the package does.

Edit a package skill only when the work did one of these:

- added or removed a top-level directory, bounded context, crate or entry point
  (one line, directory level)
- introduced a package-wide convention or architectural pattern that every later
  feature must follow, and no spec or rule states it
- hit a tooling quirk that cost real time and can't be discovered from the code

Change nothing by default. Never add: a feature, endpoint, field, error code, type, function or file;
anything a spec, contract, rule or the code already states; a how-to for one
feature.

Before writing a line, ask whether an agent working on an unrelated feature in
this package would be misled or slowed without it. If not, skip it. Edit
existing text in place instead of appending. When unsure, leave it out and
mention it in the report.

Run `spex info --package <package-short-name>` for general package information and applicable rules. <One-line description of what this package is and does>.

# Project structure

Paths are relative to the package `path` that `spex info --package <package-short-name>` prints. List only directories and entry points, one line each with its role. Group similar files into one entry; never list individual files besides entry points.

<replace with the real project structure>
- `src/main.ts` - entry point, boots the server
- `src/routes` - route definitions, one file per endpoint; add new endpoints here
- `src/components` - reusable UI components, one subfolder per component
- `src/lib` - shared logic reused across routes and components: auth, config, validation, error handling
- `tests/e2e` - end-to-end tests

# Commands

Run from the `<package-name>` package root.

<replace with project-specific commands>
- `npm dev` - serve the app
- `npm test` - run unit tests
- `npm lint` - run linters
- `npm format` - format files

# Rules

<replace with project-specific rules, if any>
After all changes run `npm format`, `npm lint` and `npm test`.

# Review

Before reviewing changes in this package, read `references/review.md`: it holds what earlier reviews of this package kept running into.

Add to it only when a review hit something that cost real time to resolve or made you ask the user, and the next review would hit it again. For example:

- a pattern that looks like a violation but is intended, with the reason
- a check or command that doesn't work as documented, with the workaround
- a decision the user already made that a reviewer would otherwise ask about again

Most reviews add nothing. Never add: a finding about the change under review; anything a spec, contract, rule or the code already states. Edit existing lines in place instead of appending, drop one that no longer holds, and when unsure leave it out and mention it in the report.

# Index

Read individual files for detailed explanations and code examples:

- `references/review.md` - what earlier reviews of this package kept running into

<add project-specific files below the review notes, if needed>

- `references/error-handling.md` - error handling patterns used in this package
- `references/testing.md` - testing strategy and conventions
- `scripts/migrate.sh` - runs the database migration
