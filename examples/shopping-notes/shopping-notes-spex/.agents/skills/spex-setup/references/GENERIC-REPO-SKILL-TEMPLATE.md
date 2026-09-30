---
name: <repository-name>
description: Information, rules, commands and other <repository-name> repository context. MUST be loaded before reading, writing, testing, or running any code in that repository.
---

# General

Run `spex info --repo <repository-short-name>` for general repository information and applicable rules. <One-line description of what this repository is and does>.

# Project structure

- `src/main.ts` - entry point, boots the server
- `src/routes` - route definitions, one file per endpoint; add new endpoints here
- `src/components` - reusable UI components, one subfolder per component
- `src/lib` - shared logic reused across routes and components: auth, config, validation, error handling
- `tests/e2e` - end-to-end tests

# Commands

Run from the `<repository-name>` repository root.

- `npm dev` - serve the app
- `npm test` - run unit tests
- `npm lint` - run linters
- `npm format` - format files

# Rules

After all changes run `npm format`, `npm lint` and `npm test`.

# Index

Read individual files for detailed explanations and code examples:

- `references/error-handling.md` - error handling patterns used in this repository
- `references/testing.md` - testing strategy and conventions
- `scripts/migrate.sh` - runs the database migration
