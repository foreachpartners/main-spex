---
name: pepperoni-pizza-frontend
description: Information, rules, commands and other pepperoni-pizza-frontend repository context. MUST be loaded before reading, writing, testing, or running any code in that repository.
---

# General

Run `spex info --repo frontend` for general repository information and applicable rule specs.

# Paths

- `src/App.tsx` - single-page UI: menu, cart, order placement, live deliveries
- `src/api.ts` - API client for the backend
- `src/types.ts` - shared types (`Pizza`, `Delivery`, `DeliveryEstimate`)
- `src/*.test.ts(x)` - unit tests, colocated with the sources

# Commands

Run from the `pepperoni-pizza-frontend` repository root.

- `npm run dev` - run the dev server
- `npm run build` - type-check and build for production
- `npm test` - run the test suite
- `npm run test:watch` - run tests in watch mode
