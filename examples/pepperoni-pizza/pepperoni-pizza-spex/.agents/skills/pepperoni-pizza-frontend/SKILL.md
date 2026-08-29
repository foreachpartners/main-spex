---
name: pepperoni-pizza-frontend
description: Information, rules, commands and other pepperoni-pizza-frontend repository context. MUST be loaded before reading, writing, testing, or running any code in that repository.
---

# General

Run `spex info --repo frontend` for general repository information and applicable rules. React + TypeScript (Vite) web client where customers browse pizzas, place orders, and watch deliveries.

# Project structure

- `src/main.tsx` - entry point, mounts `App`
- `src/App.tsx` - single-page UI: menu, cart, order placement, live deliveries
- `src/api.ts` - API client for the backend
- `src/types.ts` - shared types (`Pizza`, `Delivery`, `DeliveryEstimate`)
- `src/*.test.ts(x)` - unit tests, colocated with the sources
- `src/test-setup.ts` - test environment setup for Vitest

# Commands

Run from the `pepperoni-pizza-frontend` repository root.

- `npm run dev` - run the dev server
- `npm run build` - type-check and build for production
- `npm test` - run the test suite
- `npm run test:watch` - run tests in watch mode

# Index

Read individual files for detailed explanations and code examples:

- `references/architecture.md` - how the single-component UI, API client, and state fit together
