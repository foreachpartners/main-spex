# Plan: Show customers an estimated delivery time for their order

- Specs: BIZ-DELIVERY-TIME
- Scope: api, frontend
- Tech-specs: TECH-COMMIT-MESSAGE-FORMAT, TECH-NAMING-CONVENTION, TECH-SECRET-HANDLING, TECH-SPEC-ID-ANNOTATIONS, TECH-WRITING-STYLE

## Overview

The contract is already drafted in `spex/contracts/api.openapi.yaml`, so both repositories implement against a settled interface and run in parallel.

`api` (Rust/axum, `src/main.rs`): add a single estimation helper that turns a pizza count plus the current in-progress workload into whole seconds, using 1 minute per pizza and 2 workers each making one pizza at a time, rounding up to the next whole minute and returning it in seconds — `ceil((order_pizzas + in_progress_pizzas) / 2) * 60`. Add `estimatedSeconds` (the fixed total) to the `Delivery` struct, computed at creation from the new order's pizzas plus the pizzas already in in-progress deliveries, so both `create_order` and `list_deliveries` return it; a delivery stays in progress until its `estimatedSeconds` elapses, then drops from the listing. Add `remainingSeconds`, recomputed live in `list_deliveries` as `estimatedSeconds` minus the elapsed seconds (never negative), so the delivering view counts down on each poll. Add the `POST /order-estimate` handler that computes the live estimate from the requested pizzas plus the current in-progress pizzas. Follow the existing `create_order` handler by example for state access, validation, and error shape. Extend `src/tests.rs` following the existing `#[tokio::test]` cases.

`frontend` (React + TypeScript, `src/`): add `estimatedSeconds` and `remainingSeconds` to the `Delivery` type and a `DeliveryEstimate` type, add an `estimateOrderDeliveryTime` call to `pizzaApi` in `api.ts` (POST `/order-estimate`), show the live estimate in the cart as pizzas are added (refreshing on cart change), and show each in-progress delivery's `remainingSeconds` as its delivery time (it counts down as the app polls). Follow `api.ts`, `App.tsx`, and the existing Vitest suites by example.

## Contract changes

Already drafted in `spex/contracts/api.openapi.yaml`; no contract edits in this plan. The implementation consumes:

- `POST /order-estimate` — body `EstimateRequest` `{ pizzaIds: number[] }`; `200` returns `DeliveryEstimate` `{ estimatedSeconds: integer }`; `400` returns `Error` `{ message }`. Annotated `x-spec: BIZ-DELIVERY-TIME`.
- `Delivery` schema gains required `estimatedSeconds: integer` (whole minutes rounded up, returned in seconds; fixed at creation) and required `remainingSeconds: integer` (seconds left, recomputed each request, counting down), returned by `POST /orders` (`201`) and `GET /deliveries` (`200`).

## Steps

### Step 1 — Implement the delivery-time estimate

Status: completed

#### Task 1.1 — Estimate logic, delivery field, and estimate endpoint (repo: api)

Status: completed

Edit `api/src/main.rs` and `api/src/tests.rs`.

- Add constants for the rule: 2 workers, 1 minute per pizza.
- Add an `estimate_seconds` helper computing `ceil((order_pizzas + in_progress_pizzas) / workers)` whole minutes, then returning it in seconds (`* 60`, `i64`).
- Add `estimated_seconds: i64` and `remaining_seconds: i64` to `Delivery` and apply `#[serde(rename_all = "camelCase")]` so they serialize as `estimatedSeconds`/`remainingSeconds`. In `create_order`, compute `estimated_seconds` from the new order's pizza count plus the count of pizzas across current in-progress deliveries (set `remaining_seconds` equal to it), before pushing the new delivery. In `list_deliveries`, drop a delivery once its `estimated_seconds` has elapsed, and recompute `remaining_seconds` as `estimated_seconds` minus the elapsed seconds (never negative) on each returned delivery.
- Add the `estimate_order_delivery_time` handler for `POST /order-estimate`: accept `EstimateRequest { pizza_ids }`, validate unknown pizzas with the existing `error` helper and shape, sum the requested pizzas plus current in-progress pizzas, and return `{ estimatedSeconds }`. Register the route in `main`.
- Annotate the boundary per TECH-SPEC-ID-ANNOTATIONS: `// @spec impl BIZ-DELIVERY-TIME` once on the new handler (do not nest onto the helper). Do not re-annotate `create_order` internals.
- Add tests in `api/src/tests.rs` covering the cases below; annotate the new test(s) `// @spec test BIZ-DELIVERY-TIME`.
- Applies: TECH-SPEC-ID-ANNOTATIONS (boundary annotation), TECH-NAMING-CONVENTION, TECH-SECRET-HANDLING (no secrets introduced), TECH-WRITING-STYLE (test names and comments).
- Validate: `cargo test` in `api`.

#### Task 1.2 — Cart estimate and delivery time in the UI (repo: frontend)

Status: completed

Edit `frontend/src/types.ts`, `frontend/src/api.ts`, `frontend/src/App.tsx`, `frontend/src/api.test.ts`, and `frontend/src/App.test.tsx`.

- Add `estimatedSeconds: number` to the `Delivery` interface and a `DeliveryEstimate` interface in `types.ts`.
- Add `estimateOrderDeliveryTime(pizzaIds: number[])` to `pizzaApi` in `api.ts`, POSTing `{ pizzaIds }` to `/order-estimate` and returning `DeliveryEstimate`, following the existing `createOrder` call by example.
- In `App.tsx`, fetch and show the live estimate in the cart whenever the cart changes (fetch only when the cart is non-empty), and render each in-progress delivery's `remainingSeconds` as its delivery time in the Delivering section.
- Annotate per TECH-SPEC-ID-ANNOTATIONS: `// @spec impl BIZ-DELIVERY-TIME` once at the new `pizzaApi` method boundary; annotate the new tests `// @spec test BIZ-DELIVERY-TIME`.
- Extend `api.test.ts` and `App.test.tsx` with the frontend cases below, following the existing Vitest patterns (mock `fetch`; mock `pizzaApi`).
- Applies: TECH-SPEC-ID-ANNOTATIONS, TECH-NAMING-CONVENTION, TECH-SECRET-HANDLING, TECH-WRITING-STYLE.
- Validate: `npm test` in `frontend`.

## Test cases

Each case maps to an acceptance criterion and proves it.

- [BIZ-DELIVERY-TIME / AC1, AC2, AC4, AC5] api estimate rounds up over two workers: given an order of 3 pizzas and no in-progress deliveries, when estimating, then the result is `120` whole seconds (`ceil(3/2)` minutes × 60), using 1 minute per pizza and 2 workers.
- [BIZ-DELIVERY-TIME / AC3] api estimate counts in-progress workload: given 1 pizza in the order and a delivery already in progress holding 2 pizzas, when estimating, then the result is `120` seconds (`ceil((1+2)/2)` minutes × 60).
- [BIZ-DELIVERY-TIME / AC6] `POST /order-estimate` exposes the pre-order estimate: given a valid `pizzaIds` body, when called, then `200` returns `{ estimatedSeconds }` for those pizzas plus current in-progress pizzas; an unknown pizza id returns `400` with an error message.
- [BIZ-DELIVERY-TIME / AC7] deliveries carry the estimate: given a created order, then the `201` `Delivery` includes `estimatedSeconds`; and `GET /deliveries` returns `estimatedSeconds` on each in-progress delivery.
- [BIZ-DELIVERY-TIME / AC8] in-progress deliveries count down: given a delivery created with a 120-second estimate, when 30 seconds have elapsed and `GET /deliveries` is called, then it returns `estimatedSeconds` `120` and `remainingSeconds` `90`.
- [BIZ-DELIVERY-TIME / AC6, AC9] frontend shows the cart estimate: given pizzas added to the cart, when `estimateOrderDeliveryTime` resolves, then the cart displays the returned estimated seconds; `api.test.ts` asserts `pizzaApi.estimateOrderDeliveryTime` POSTs `{ pizzaIds }` to `/order-estimate`.
- [BIZ-DELIVERY-TIME / AC10] frontend shows the remaining delivery time: given an in-progress delivery with `remainingSeconds`, when rendered, then the Delivering section shows its remaining delivery time in seconds.
