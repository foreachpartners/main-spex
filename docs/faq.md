# FAQ

## Why a workspace of separate repositories?

A workspace keeps every repository it manages as its own git repository under one shared folder, each named `<project-name>-<repository-short-name>`:

```text
pepperoni-pizza/             <- workspace root folder
  pepperoni-pizza-spex/      <- specs, config, and contracts/
  pepperoni-pizza-api/
  pepperoni-pizza-frontend/
```

Each person — and each AI agent — gets access only to what they need. Because each repository is independent, access is granted per repository:

- A backend engineer gets access to `api`, not `frontend`.
- A frontend engineer gets access to `frontend`, not `api`.
- A contractor or AI agent scoped to one task gets only the repositories that task touches.
- Everyone who needs to read or approve specs gets access to `spex`, without that granting access to any implementation.

A monorepo grants all-or-nothing access: anyone who can clone it sees every line of every service. Splitting the project into separate repositories under a workspace keeps least privilege the default, so a leak, a compromised credential, or an over-eager agent is limited to a single repository instead of the whole project.
