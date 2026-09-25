# shopping-notes

An example workspace used to test the spex framework. It is a small shopping
notes application that lets a user note down shopping items so they don't
forget to buy them when visiting a store, split into the kind of sibling
repositories spex expects to manage, **with** `spex` fully set up: specs,
contracts, implementation, and tests for every acceptance criterion.

Use it as a reference for what a completed [Tutorial](../../docs/tutorial.md)
run looks like, starting from `shopping-notes-no-spex`.

## Layout

```text
shopping-notes/               <- workspace root folder
  shopping-notes-api/         <- node fastify api
  shopping-notes-frontend/    <- react SPA
  shopping-notes-spex/        <- specs, config, and contracts/
```

## State

- api: node fastify api with a healthcheck endpoint and shopping list capabilities (add, view, mark bought, delete, clear), vitest tests
- frontend: react SPA with a home page, a not found page, and shopping list capabilities (add, view, mark bought, delete, clear), vitest unit tests and playwright e2e tests
- spex: full spec, contract, and traceability coverage — see `shopping-notes-spex/tutorial-completion.md`
