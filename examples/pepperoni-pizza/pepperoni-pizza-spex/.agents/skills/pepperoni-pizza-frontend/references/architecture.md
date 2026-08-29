# Architecture

Single-page app with no routing and no global state library.

- `App.tsx` is the entire UI: menu, cart, and live deliveries are just
  `useState` values in one component, rendered as three `<section>`s. There
  are no child components — adding one is a deliberate structural change, not
  the existing pattern.
- All server communication goes through `api.ts` (`pizzaApi`); components
  never call `fetch` directly. `types.ts` holds the shared shapes
  (`Pizza`, `Delivery`, `DeliveryEstimate`) mirrored from the API's JSON.
- Server state is kept fresh with `useEffect` + polling/refetch-on-change,
  not a data-fetching library — there is no caching layer, so every effect
  owns its own fetch and cleanup.
