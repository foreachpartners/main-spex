# Architecture

Single Axum service, no database — everything lives in process memory for the
lifetime of the server.

- `AppState` is the one shared state struct, cloned into every handler via
  Axum's `State` extractor: a static `menu` (`Arc<Vec<Pizza>>`), a mutable
  `deliveries` list (`Arc<Mutex<Vec<Delivery>>>`), and an `AtomicI64` id
  counter. There is no persistence layer — restarting the process clears all
  deliveries.
- Handlers are plain async functions registered directly on the `Router` in
  `main` — no separate router/service/repository layers. Request parsing,
  validation, business logic, and response shaping all happen inline in each
  handler.
- Config is loaded once at startup (`.env` overrides `.env.dev`, real env vars
  win over both) into a single `Config` struct; there is no runtime
  reconfiguration.
- Concurrency is handled with a single `Mutex` around the deliveries `Vec`;
  handlers lock it for the duration of their read/write and drop the guard
  before returning a response.
