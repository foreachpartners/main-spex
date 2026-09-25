---
name: shopping-notes-api
description: Information, rules, commands and other shopping-notes-api repository context. MUST be loaded before reading, writing, testing, or running any code in that repository.
---

# General

Run `spex info --repo api` for general repository information and applicable rules. Fastify REST API backend for the shopping notes application.

# Project structure

- `src/index.ts` - entry point, starts the Fastify server
- `src/app.ts` - builds and configures the Fastify app instance (routes, plugins)
- `src/app.test.ts` - tests for the app
- `src/item-store.ts` - `ItemStore` class, the JSON-file-backed shopping item store used by the `/items` routes
- `data/items.json` - default JSON file store location (git-ignored, created on first write); override with the `SHOPPING_NOTES_API_DATA_FILE` env var or the `dataFile` option to `createApp()`

# Commands

Run from the `shopping-notes-api` repository root.

- `npm start` - serve the app (http://localhost:3000)
- `npm test` - run unit tests (vitest)
- `npm run build` - type-check and compile with tsc
