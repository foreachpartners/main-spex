# shopping-notes-no-spex

An example workspace used to test the spex framework. It is a small shopping
notes application that let's user note down shoping items to not forget to buy them when visiting a store 
split into the kind of sibling repositories spex expects to
manage, but **without** `spex` initialized.

Use it as a clean starting point to try `spex init` and the rest of the flow
against a realistic, multi-repo project.

## Layout

```text
shopping-notes-no-spex/      <- workspace root folder
  shopping-notes-api/        <- node fastify api
  shopping-notes-frontend/   <- react SPA
```

## State

- api: node fastify api with `GET /` returning `hello from api` and vitest tests
- frontend: react SPA with a home page and an `/api-test` page that fetches the api, vitest unit tests and playwright e2e tests
