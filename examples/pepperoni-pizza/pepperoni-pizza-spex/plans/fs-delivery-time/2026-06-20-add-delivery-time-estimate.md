# Plan: Show customers an estimated delivery time for their order

- Specs: FS-DELIVERY-TIME
- Scope: api, frontend
- Rule-specs: RS-COMMIT-MESSAGE-FORMAT, RS-CONFIGURABLE-NETWORK-BIND, RS-DEPENDENCIES, RS-NAMING-CONVENTION, RS-SECRET-HANDLING, RS-SPEC-ID-TRACING, RS-TESTING, RS-WRITING-STYLE

## Step 1 — Compute, expose, and display the delivery-time estimate

Status: completed

### Task 1.1 — Estimate computation and endpoints (repo: api)

Status: completed

New constants in `src/main.rs`, encoding the workload model:

```rust
const WORKERS: i64 = 2;
const MINUTES_PER_PIZZA: i64 = 1;
```

New pure function in `src/main.rs`:

```rust
fn estimate_seconds(order_pizzas: i64, in_progress_pizzas: i64) -> i64;
```

Changed `Delivery` domain model in `src/main.rs`, adding the estimate and its countdown anchor:

```rust
struct Delivery {
    id: i64,
    items: Vec<String>,
    estimated_seconds: i64, // fixed at order creation
    remaining_seconds: i64, // recomputed on each read
    created_at: Instant,    // countdown anchor, not serialized
}
```

New `Delivery` methods in `src/main.rs`:

```rust
impl Delivery {
    fn delivery_window(&self) -> Duration;
    fn remaining_seconds(&self) -> i64;
}
```

New request type in `src/main.rs`, matching the `EstimateRequest` contract schema:

```rust
struct EstimateRequest {
    pizza_ids: Vec<i64>,
}
```

New handler in `src/main.rs`, returning the `DeliveryEstimate` contract schema (`{ "estimatedSeconds": i64 }`):

```rust
async fn estimate_order_delivery_time(
    State(state): State<AppState>,
    Json(input): Json<EstimateRequest>,
) -> Response; // POST /order-estimate
```

`create_order` and `list_deliveries` keep their existing signatures; their bodies populate and refresh the new `Delivery` fields (`estimated_seconds` at creation, `remaining_seconds` on every read, with elapsed deliveries dropped).

Applies: RS-SPEC-ID-TRACING (annotate `estimate_order_delivery_time`, the boundary), RS-TESTING (cover rounding, in-progress workload, validation, and the countdown, with no skipped or focused tests).

### Task 1.2 — Cart estimate and delivery countdown UI (repo: frontend)

Status: completed

Changed `Delivery` type in `src/types.ts`, adding the estimate fields:

```ts
export interface Delivery {
  id: number;
  items: string[];
  estimatedSeconds: number;
  remainingSeconds: number;
}
```

New `DeliveryEstimate` type in `src/types.ts`:

```ts
export interface DeliveryEstimate {
  estimatedSeconds: number;
}
```

New `pizzaApi` method in `src/api.ts`:

```ts
estimateOrderDeliveryTime: (pizzaIds: number[]) => Promise<DeliveryEstimate>;
```

`App.tsx` calls `estimateOrderDeliveryTime` whenever the cart changes to show the live estimate, and renders each polled delivery's `remainingSeconds` as its countdown.

Applies: RS-SPEC-ID-TRACING (annotate `estimateOrderDeliveryTime`, the boundary), RS-TESTING (cover the API client call and the cart/delivery UI states, with no skipped or focused tests).
