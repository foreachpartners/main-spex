# FAQ

## Which layout should I choose?

`spex init` asks which layout the project uses:

- `multi-repo`: every package is its own git repository under one workspace root folder, with a separate spex repository. Choose it when access should be granted per repository, as explained below.
- `single-repo`: the project is one git repository. The spex files live in its `spex/` folder, and `AGENTS.md`, `CLAUDE.md`, `.agents` and `.claude` in its root.
- `monorepo`: one git repository holding several packages, each listed in `spex.yaml` as a package. Laid out like `single-repo`.

## Why a workspace of separate repositories?

A workspace keeps every package it manages as its own git repository under one shared folder, each named `<project-name>-<package-short-name>`:

```text
shopping-notes/             <- workspace root folder
  shopping-notes-spex/      <- specs, config, and contracts/
  shopping-notes-api/
  shopping-notes-frontend/
```

Each person — and each AI agent — gets access only to what they need. Because each package is its own git repository, access is granted per repository:

- A backend engineer gets access to `api`, not `frontend`.
- A frontend engineer gets access to `frontend`, not `api`.
- A contractor or AI agent scoped to one task gets only the packages that task touches.
- Everyone who needs to read or approve specs gets access to `spex`, without that granting access to any implementation.

A single repository or monorepo grants all-or-nothing access: anyone who can clone it sees every line of every service. Splitting the project into separate repositories under a workspace keeps least privilege the default, so a leak, a compromised credential, or an over-eager agent is limited to a single repository instead of the whole project.
