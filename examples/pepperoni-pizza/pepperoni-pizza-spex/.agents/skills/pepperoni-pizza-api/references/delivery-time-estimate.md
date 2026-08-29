# Delivery time estimate

`estimate_seconds` in `src/main.rs` computes how long a delivery takes:

- `WORKERS` (2) pizzas can be made in parallel; each takes `MINUTES_PER_PIZZA` (1) minute.
- The estimate covers the new order's pizzas plus every pizza already `in_progress_pizzas` across current deliveries, so a busy kitchen quotes a longer wait.
- `(total * MINUTES_PER_PIZZA + WORKERS - 1) / WORKERS` divides the work across the workers and rounds up to the next whole minute, then the result is returned in seconds.

`POST /order-estimate` calls this without creating a delivery, to preview the wait before ordering. `POST /orders` calls the same function to set the new delivery's `estimated_seconds`.

## Countdown and expiry

Each `Delivery` records `created_at`. `remaining_seconds()` subtracts elapsed time from `estimated_seconds`, floored at zero, so the frontend can show a live countdown. `GET /deliveries` (`list_deliveries`) drops any delivery whose `delivery_window()` (its `estimated_seconds`) has fully elapsed before returning the list — a delivery disappears once it's arrived, there is no separate "completed" state.
