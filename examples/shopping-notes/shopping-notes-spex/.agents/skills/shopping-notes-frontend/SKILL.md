---
name: shopping-notes-frontend
description: Information, rules, commands and other shopping-notes-frontend package context. MUST be loaded before reading, writing, testing, or running any code in that package.
---

# General

Run `spex info --package frontend` for general package information and applicable rules. React SPA that lets users create and manage shopping notes against the API.

# Project structure

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
