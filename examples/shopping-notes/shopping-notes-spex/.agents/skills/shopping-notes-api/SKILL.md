---
name: shopping-notes-api
description: Information, rules, commands and other shopping-notes-api package context. MUST be loaded before reading, writing, testing, reviewing, or running any code in that package.
---

# General

Run `spex info --package api` for general package information and applicable rules. Fastify REST API backend for the shopping notes application.

# Project structure

Paths are relative to the package `path` that `spex info --package api` prints. List only directories and entry points, one line each with its role. Group similar files into one entry; never list individual files besides entry points.

- `src/index.ts` - entry point, starts the Fastify server
- `src/app.ts` - builds and configures the Fastify app instance (routes, plugins)
- `src` - also holds `ItemStore`, the JSON-file-backed shopping item store used by the `/items` routes, and the colocated `*.test.ts` tests
- `data/` - default JSON file store location (git-ignored, created on first write); override with the `SNT_API_DATA_FILE` env var or the `dataFile` option to `createApp()`

# Commands

Run from the `shopping-notes-api` package root.

- `npm start` - serve the app (http://localhost:3000)
- `npm test` - run unit tests (vitest)
- `npm run build` - type-check and compile with tsc

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
