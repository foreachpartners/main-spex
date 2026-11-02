---
name: shopping-notes-api
description: Information, rules, commands and other shopping-notes-api package context. MUST be loaded before reading, writing, testing, or running any code in that package.
---

# General

Run `spex info --package api` for general package information and applicable rules. Fastify REST API backend for the shopping notes application.

# Project structure

Paths are relative to the package `path` that `spex info --package api` prints. List only directories and entry points, one line each with its role. Group similar files into one entry; never list individual files besides entry points.

- `src/index.ts` - entry point, starts the Fastify server
- `src/app.ts` - builds and configures the Fastify app instance (routes, plugins)
- `src` - also holds `ItemStore`, the JSON-file-backed shopping item store used by the `/items` routes, and the colocated `*.test.ts` tests
- `data/` - default JSON file store location (git-ignored, created on first write); override with the `SHOPPING_NOTES_API_DATA_FILE` env var or the `dataFile` option to `createApp()`

# Commands

Run from the `shopping-notes-api` package root.

- `npm start` - serve the app (http://localhost:3000)
- `npm test` - run unit tests (vitest)
- `npm run build` - type-check and compile with tsc
