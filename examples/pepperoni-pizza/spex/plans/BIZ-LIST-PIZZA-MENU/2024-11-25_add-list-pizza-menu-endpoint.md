# Plan: Let customers list the pizza menu

- Specs: BIZ-LIST-PIZZA-MENU
- Scope: contracts, api, frontend
- Tech-specs: TECH-COMMIT-MESSAGE-FORMAT, TECH-WRITING-STYLE, TECH-NAMING-CONVENTION, TECH-SECRET-HANDLING, TECH-SPEC-ID-ANNOTATIONS

## Overview

The system currently exposes only `/orders`. This work adds a read-only pizza menu across the three repositories, following the existing orders pattern by example.

- **contracts**: Add a `GET /pizzas` operation and a `Pizza` schema to `openapi.yaml`, mirroring the shape and conventions of the existing `listOrders` operation and `Order` schema.
- **api**: Add a `Pizza` struct, a seeded in-memory menu in `AppState`, a `list_pizzas` handler, and a `/pizzas` route in `main.rs` — copying the structure of `Order` and `list_orders`. Add a test that exercises the handler.
- **frontend**: Add a `Pizza` type to `types.ts`, a `pizzasApi.list` call to `api.ts` mirroring `ordersApi.list`, and render the fetched menu in `App.tsx`.

## Contract changes

- `GET /pizzas` (operationId `listPizzas`): returns `200` with `application/json` body `array` of `Pizza`. No parameters, no error responses (an empty menu returns `200` with `[]`).
- New schema `Pizza` (object), required `id`, `name`, `description`, `price`:
  - `id`: string, stable menu key (slug), `readOnly` (example `margherita`).
  - `name`: string, `minLength: 1` (example `Margherita`).
  - `description`: string (example `Tomato, mozzarella, basil`).
  - `price`: number, currency amount, `minimum: 0` (example `9.5`).
- Annotate the operation with `x-spec: "@spec contract BIZ-LIST-PIZZA-MENU"` per TECH-SPEC-ID-ANNOTATIONS.

## Steps

### Step 1 — Add the pizza menu contract

Status: completed

#### Task 1.1 — Add `GET /pizzas` and `Pizza` schema (repo: contracts)

Status: completed

Edit `contracts/openapi.yaml`. Add the `/pizzas` path with the `listPizzas` operation and the `Pizza` schema under `components/schemas`, mirroring the existing `listOrders` / `Order` definitions. Put `x-spec: "@spec contract BIZ-LIST-PIZZA-MENU"` on the operation. Apply TECH-NAMING-CONVENTION (operationId and schema naming consistent with existing entries), TECH-SPEC-ID-ANNOTATIONS, and TECH-WRITING-STYLE for the `summary`/`description` text. Annotate spec: BIZ-LIST-PIZZA-MENU.

### Step 2 — Implement and consume the contract (parallel)

Status: completed

Both tasks depend only on Step 1 and edit disjoint repositories, so separate agents run them in parallel.

#### Task 2.1 — Implement `list_pizzas` in the api (repo: api)

Status: completed

Edit `api/src/main.rs`. Add a `Pizza` struct (`Serialize`) matching the contract, seed a fixed menu into `AppState` (a stable, consistently ordered `Vec<Pizza>`), add an `async fn list_pizzas` handler copying `list_orders`, and register `.route("/pizzas", get(list_pizzas))`. Place `// @spec impl BIZ-LIST-PIZZA-MENU` at the handler boundary per TECH-SPEC-ID-ANNOTATIONS. Add a `#[cfg(test)] mod tests` with a test that calls the handler (or the menu builder) and asserts the returned menu and its ordering; mark it `// @spec test BIZ-LIST-PIZZA-MENU`. Run `cargo test`. Apply TECH-NAMING-CONVENTION, TECH-SECRET-HANDLING (no secrets, no `println!` of sensitive data), TECH-SPEC-ID-ANNOTATIONS. Annotate spec: BIZ-LIST-PIZZA-MENU.

#### Task 2.2 — Display the pizza menu in the frontend (repo: frontend)

Status: completed

Edit `frontend/src/types.ts` (add `Pizza` interface), `frontend/src/api.ts` (add `pizzasApi` with `list: () => request<Pizza[]>("/pizzas")`, mirroring `ordersApi.list`, with `// @spec impl BIZ-LIST-PIZZA-MENU` at the `pizzasApi` boundary), and `frontend/src/App.tsx` (fetch the menu on mount and render each pizza's name, description, and price; show the menu, and an empty-state when the menu is empty). Run `npm run build` to typecheck. Apply TECH-NAMING-CONVENTION, TECH-SECRET-HANDLING, TECH-SPEC-ID-ANNOTATIONS. Annotate spec: BIZ-LIST-PIZZA-MENU.

## Test cases

Each case maps to an acceptance criterion and proves it.

- [BIZ-LIST-PIZZA-MENU / AC1] Full menu returned: given a seeded menu of N pizzas, when `GET /pizzas`, then the response is `200` with all N pizzas.
- [BIZ-LIST-PIZZA-MENU / AC2] Pizza fields present: given the menu, when `GET /pizzas`, then every item has a non-empty `id`, `name`, `description`, and a numeric `price`.
- [BIZ-LIST-PIZZA-MENU / AC3] Empty menu is not an error: given an empty menu, when `GET /pizzas`, then the response is `200` with body `[]` (not an error status).
- [BIZ-LIST-PIZZA-MENU / AC4] Consistent ordering: given the seeded menu, when `GET /pizzas` is called twice, then both responses list the pizzas in the same order.
