# FAQ

## Why a workspace of separate repositories?

A workspace keeps every repository it manages (`spex`, `api`, `frontend`, …) as its own git repository under one shared folder:

```text
pizza-shop/        <- workspace root folder
  spex/
  contracts/
  api/
  frontend/
```

There are two reasons for this layout.

### Access control separation

Each person — and each AI agent — gets access only to what they need. Because each repository is independent, access is granted per repository:

- A backend engineer gets access to `api`, not `frontend`.
- A frontend engineer gets access to `frontend`, not `api`.
- An API designer gets access to `contracts`, without touching the services that implement it.
- A contractor or AI agent scoped to one task gets only the repositories that task touches.
- Everyone who needs to read or approve specs gets access to `spex`, without that granting access to any implementation.

A monorepo grants all-or-nothing access: anyone who can clone it sees every line of every service. Splitting the project into separate repositories under a workspace keeps least privilege the default, so a leak, a compromised credential, or an over-eager agent is limited to a single repository instead of the whole project.

### One spec, traceable across many repositories

Keeping specs in their own `spex` repository lets a single Spec ID govern a contract and its implementations across more than one repository. `BIZ-PLACE-PIZZA-ORDER` can be referenced from `contracts` (the OpenAPI definition of `placeOrder`), from `api` (the endpoint), and from `frontend` (the order form), and all trace back to the same approved spec:

```text
spex/        BIZ-PLACE-PIZZA-ORDER              <- single source of truth
  contracts/   openapi.yaml               references BIZ-PLACE-PIZZA-ORDER
  api/         implementation             references BIZ-PLACE-PIZZA-ORDER
  frontend/    implementation             references BIZ-PLACE-PIZZA-ORDER
```

If the spec lived inside one implementation repository, a requirement that spans services would have no natural home and traceability would break at the repo boundary. With `spex` as a shared, versioned source of truth, every repo links to the same reviewed spec by ID, and you can check out `spex` at the commit the code was built against to see exactly what was approved.
