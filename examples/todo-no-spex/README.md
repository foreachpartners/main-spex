# todo-no-spex

An example workspace used to test the spex framework. It is a small todo
application split into the kind of sibling repositories spex expects to manage,
but **without** a `spex` initialized.

Use it as a clean starting point to try `spex init` and the rest of the flow
against a realistic, multi-repo project.

## Layout

```text
todo-no-spex/        <- workspace root folder
  api/               <- Rust todo API (axum, in-memory store)
  frontend/          <- React + TypeScript UI (Vite)
  contracts/         <- OpenAPI definition of the API
```

```sh
cd examples/todo-no-spex
mkdir spex && cd spex
spex init
```

## Running the example

### API

```sh
cd api
cargo run
# Serves http://localhost:3000
```

### Frontend

```sh
cd frontend
npm install
npm run dev
# Serves http://localhost:5173
```

The frontend talks to the API at `http://localhost:3000`, so start the API
first.
