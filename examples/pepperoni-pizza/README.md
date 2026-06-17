# pepperoni-pizza-spex

Mirror of `pepperoni-pizza-no-spex` **with** a `spex` initialized.

Use it as a clean starting point to try `spex check` and the rest of the flow
against a realistic, multi-repo project.

## Layout

```text
pepperoni-pizza/      <- workspace root folder
  api/               <- Rust pizza order API (axum, in-memory store)
  frontend/          <- React + TypeScript UI (Vite)
  contracts/         <- OpenAPI definition of the API
  spex/              <- spex framework harness, specs, skills, scripts and workspace setup
```
