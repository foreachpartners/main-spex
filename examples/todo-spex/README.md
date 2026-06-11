# todo-spex

Mirror of `todo-no-spex` **with** a `spex` initialized.

Use it as a clean starting point to try `spex check` and the rest of the flow
against a realistic, multi-repo project.

## Layout

```text
todo-no-spex/        <- workspace root folder
  api/               <- Rust todo API (axum, in-memory store)
  frontend/          <- React + TypeScript UI (Vite)
  contracts/         <- OpenAPI definition of the API
  spex/              <- spex framwork harness, specs, skills, scripts and workspace setup
```
