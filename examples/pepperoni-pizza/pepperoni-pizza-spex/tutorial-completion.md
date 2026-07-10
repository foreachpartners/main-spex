# Tutorial Completion

This document records the result of the spex tutorial: the verification command
output, and an assessment of the capability spec, its implementation, and the
network-bind rule spec produced along the way.

## spex check

```
api
  no issues

frontend
  no issues

spex
  no issues

workspace
  no issues

pepperoni-pizza: 0 errors, 0 warnings
```

## spex coverage

```
spex coverage · pepperoni-pizza

Business specs (1)
  active      1

Active spec coverage (1)
  ✓  CS-DELIVERY-TIME

  1 fully covered · 0 gaps

pepperoni-pizza: no gaps
```

## Rating: CS-DELIVERY-TIME spec and implementation

**Requirement:** "show customers pizza delivery time based on number of pizzas
ordered, other ongoing orders and today workers" (details left to the author).

**Verdict: Strong (4 / 5).** A clean, well-decomposed spec with a faithful,
fully-tested implementation. One liberty taken with "today workers" keeps it
from a perfect score.

### What was done well

- **All three required inputs are modeled.** `api/src/main.rs` computes the
  estimate from the order's own pizzas plus the pizzas across all in-progress
  deliveries (`in_progress_pizzas`), divided across workers — covering "pizzas
  ordered", "other ongoing orders", and "workers" in one formula
  (`estimate_seconds`).
- **The spec is precise and testable.** `CS-DELIVERY-TIME.yaml` breaks the
  vague requirement into 10 `MUST` acceptance criteria, each pinning a concrete
  rule (1 min/pizza, 2 workers, round up to the whole minute, return whole
  seconds) and a clear owner (`api` vs `frontend`).
- **Two distinct customer touchpoints**, matching the spirit of "show
  customers": a live cart estimate (`POST /order-estimate`) and a counting-down
  `remainingSeconds` on each in-progress delivery — both surfaced in
  `frontend/src/App.tsx`.
- **Coverage and tests are real.** `cargo test` passes 12/12 and `npm test`
  passes 11/11; tests map back to acceptance criteria (e.g.
  `estimate_rounds_up_over_two_workers`, `list_deliveries_counts_down_remaining_time`),
  and `// @spec` annotations tie code and tests to the spec, so
  `spex coverage` reports the spec fully covered with no gaps.
- **Sensible edge handling:** `remaining_seconds` is floored at 0, deliveries
  drop once their estimate elapses, and unknown pizza IDs return `400`.

### Where it falls short

- **"Today workers" is hardcoded, not dynamic.** Worker count is a compile-time
  constant (`const WORKERS: i64 = 2`), and the spec froze this as AC2 ("using 2
  workers on shift"). The literal requirement implies staffing that varies by
  day; the current design cannot reflect a different number of workers without a
  rebuild. Defensible given the author was free to decide details, but it is the
  weakest interpretation of the requirement.
- **The estimate ignores in-flight progress.** A new order's estimate counts the
  _full_ pizza load of every in-progress delivery, even ones nearly complete, so
  the live estimate can overstate the true wait. Acceptable as a simplification,
  worth noting as future refinement.
- **Concurrency nuance:** `create_order` reads in-progress pizzas and pushes the
  new delivery under one lock (good), but the estimate is a point-in-time
  snapshot — fine for this in-memory toy, not a correctness guarantee under a
  real datastore.

## Rating: RS-CONFIGURABLE-NETWORK-BIND spec

**Requirement:** "the API must not hardcode network bind addresses or ports —
they must be read from configuration/environment".

**Verdict: Excellent (5 / 5).** This is a model rule spec: it states the rule,
explains _why_, enforces it automatically, and the implementation conforms.

### What was done well

- **Captures intent, not just a rule.** The description explains the cost of
  hardcoding (locks to one environment, leaks topology, forces rebuilds), so a
  reader understands the motivation rather than memorizing a ban.
- **Precise, layered acceptance criteria.** It requires reading bind config from
  the environment, sourcing it from the repo's config-prefix (cross-referencing
  `RS-NAMING-CONVENTION` for `PEPPERONI_PIZZA_API__BIND_ADDR`), forbids any
  embedded literal on the startup path, and adds a `SHOULD` to fail fast on
  missing/malformed config.
- **It is enforced automatically.** The `no-hardcoded-bind-literal` regexp check
  scans `src/**/*.rs` for socket-address literals (IPv4, `localhost`, IPv6 loop,
  `:port`) at `error` severity — turning the rule into a guardrail rather than a
  guideline. This is why `spex check` on `api` reports no issues.
- **Concrete avoid/prefer examples** make the expected pattern unambiguous.
- **The implementation matches the spec.** `api/src/main.rs` loads `bind_addr`
  via the `config` crate with the `PEPPERONI_PIZZA_API__` prefix, falls back to
  the committed `.env.dev` default (`PEPPERONI_PIZZA_API__BIND_ADDR=127.0.0.1:3000`),
  binds with no literal in source, and panics with a clear message when the key
  is absent — satisfying every criterion including the fail-fast `SHOULD`.

### Minor nitpicks

- The IPv6 portion of the regex (`\[::1?\]`) is written as character-class-ish
  and would only match `[::1]`/`[::]` forms; broader IPv6 literals could slip
  past. Low risk for this codebase, but the pattern is the spec's weakest link.
- The check covers only the server's `src` (correct scope), so a hardcoded URL
  elsewhere — e.g. `BASE_URL = "http://localhost:3000"` in
  `frontend/src/api.ts` — is intentionally out of scope of this API-focused
  spec; flagging it would belong to a separate frontend config spec.
