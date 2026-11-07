---
name: shopping-notes-frontend
description: Information, rules, commands and other shopping-notes-frontend package context. MUST be loaded before reading, writing, testing, reviewing, or running any code in that package.
---

# General

Run `spex info --package frontend` for general package information and applicable rules. React SPA that lets users create and manage shopping notes against the API.

# Project structure

Paths are relative to the package `path` that `spex info --package frontend` prints. List only directories and entry points, one line each with its role. Group similar files into one entry; never list individual files besides entry points.

- `src/main.tsx` - entry point, mounts the React app
- `src/app.tsx` - root app component and routing
- `src/pages` - page components, one file per route (`home.tsx`, `api-test.tsx`), each with a colocated `*.test.tsx`
- `e2e/` - Playwright end-to-end tests

# Commands

Run from the `shopping-notes-frontend` package root.

- `npm start` - serve the app (http://localhost:5173)
- `npm test` - run unit tests then e2e tests
- `npm run test:unit` - run unit tests only (vitest)
- `npm run test:e2e` - run e2e tests only (playwright; requires `npx playwright install chromium` once)
- `npm run build` - type-check and build for production

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
