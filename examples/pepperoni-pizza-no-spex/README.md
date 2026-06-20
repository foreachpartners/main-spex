# pepperoni-pizza-no-spex

An example workspace used to test the spex framework. It is a small pepperoni
pizza shop application split into the kind of sibling repositories spex expects
to manage, but **without** a `spex` initialized.

Use it as a clean starting point to try `spex init` and the rest of the flow
against a realistic, multi-repo project.

## What it does

A single screen with three parts:

1. **Menu** — five pizzas fetched from the API, each with an `Add` button.
2. **Cart** — pizzas you added. Once it has at least one pizza, an `Order`
   button appears.
3. **Delivering** — ordering empties the cart and starts a delivery. The API
   drops each delivery 5 seconds after it starts, and the frontend polls every
   2 seconds so the delivering view clears itself.

## Layout

```text
pepperoni-pizza-no-spex/   <- workspace root folder
  api/               <- Rust pizza order API (axum, in-memory store)
  frontend/          <- React + TypeScript UI (Vite)
  contracts/         <- OpenAPI definition of the API
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
