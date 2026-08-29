# Live delivery polling

`App.tsx` keeps the "Delivering" section in sync with the backend without a push channel:

- On mount, and every 2 seconds after (`setInterval`), it calls `pizzaApi.listDeliveries()` and replaces `deliveries` state with the response.
- The backend (`GET /deliveries`) drops a delivery once its estimated delivery time has elapsed, so a delivery simply stops appearing in a later poll — the frontend never marks one "arrived" itself.
- The interval is cleared in the effect's cleanup, so only one poll loop ever runs per mount.

A second effect refetches the delivery-time estimate (`pizzaApi.estimateOrderDeliveryTime`) whenever `cart` changes, resetting it to `null` once the cart empties so the UI never shows a stale estimate for an empty cart.
