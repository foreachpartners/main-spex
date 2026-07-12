---
name: spex-plan
description: Create a new implementation plan for one or more functional specs, or edit an existing plan. Use when the user wants to plan, break down, prepare, update, or amend implementation for a FS spec, or change a named existing plan. Reads the spec, contract, the affected repositories, and the rs-specs that govern them, inspects the current code, then writes or updates a step-by-step implementation plan that AI agents can do in parallel.
---

# Create or Edit an Implementation Plan

An implementation plan turns one or more approved functional specs into an ordered set of steps whose tasks AI agents execute in parallel. A plan captures the shape of a change — new or changed method signatures, database/schema changes, domain models, handler/endpoint signatures — never its logic. Your job is to read the specs and contracts, resolve the governing rs-specs, inspect the current code, then draft or update the plan.

## Paths

All paths are relative to the workspace root (the folder that holds every workspace repository). Replace `<workspace-root>` with the real folder name and `<spec-id>` with the lead spec's `id`.

Each spec's plans live at `<workspace-root>/<project-name>-spex/plans/<spec-id>/<date>-<slug>.md`. The schema at `<workspace-root>/<project-name>-spex/schemas/plan-filename.schema.json` is authoritative for the filename rules.

## Step 1 — Read the specs and contracts

Determine which specs to plan:

- If the user named one or more spec IDs, plan those; with several, pick the lead spec (the one the others depend on, or the first named) — its `id` names the plan directory.
- If the user said "the one I just made" or gave no ID but a spec was created earlier in this session, plan that one and confirm.
- If the user instead named an existing plan (a filename, a path, or "the plan for `<spec-id>`"), this is an edit: read it in full and treat its steps as the base, changing only what the user asked for and preserving every untouched step/task and its `Status:` line. If a spec ID has more than one plan, list them and ask which to edit.
- If no spec is identified, ask which to plan. Do not invent a spec.
- Each target spec must already exist at `<workspace-root>/<project-name>-spex/fs-specs/<spec-id>.yaml`; if one is missing, stop and create it first with `spex-functional-spec`.

Read every target spec in full. Capture its `title`, `description`, `acceptance_criteria`, `scope` (the in-scope repositories), and `blocked_by`; confirm with the user how to handle any unresolved blocker.

Read the contract(s) the spec touches in the `spex` repository's `contracts/` folder (listed per repository under `contracts` in `spex.yaml`). If an acceptance criterion crosses a repository boundary and no contract covers it, stop and draft one first with `spex-contract`; a spec with no cross-repository interface needs none.

## Step 2 — Resolve the governing rs-specs

From `<workspace-root>/<project-name>-spex/spex.yaml` (repository names are arbitrary — only `spex` is fixed — read each repository's `description` to learn its role):

- Collect the union of every targeted spec's `scope` — the affected repositories.
- For each affected repository, read its `rs_specs` list (repository-wide specs are already expanded into it). The deduplicated union is the set of governing rs-specs.
- Read each one in full from `<workspace-root>/<project-name>-spex/rs-specs/` so the plan's tasks respect naming, annotation, and testing conventions. Note which repository each came from, so a task only carries the rs-specs that apply there.

## Step 3 — Inspect the current code

For each affected repository, resolve its `path` in `spex.yaml` and read what the spec will change:

- The domain models, repositories/data-access, and handlers/endpoints nearest to the change.
- The database schema or migrations, if the spec touches persisted data.
- One existing feature to copy the structure from (the "by example" pattern).

Ground every interface you plan in what already exists — extend it, don't guess at it.

## Step 4 — Draft the steps and tasks

Break the work into ordered steps. Within a step, split into tasks only when there is more than one independent unit:

- Every task in one step MUST be safe to run in parallel: it must not depend on another task's output in the same step, and must not edit the same files as another task in that step.
- Put work that depends on an earlier result in a later step — a schema or contract change comes before the code that consumes it.
- Give each task a disjoint write scope (its own files) and state which repository it runs in.
- Name which single place carries each spec annotation per RS-SPEC-ID-TRACING — the boundary (a handler, or the first file in a relation chain) — never the nested helpers it calls.
- Keep steps as few as the dependencies allow; a small spec may be a single step with a single task.

Describe only the shape of each change, never its logic:

- A new method: its full signature as code, labeled with its type and file.
- A changed method: the same, showing before and after.
- A table added or changed: a Markdown table, one row per column, with type and constraints.
- A domain model added or changed: a fenced code block with its fields and types, no logic.
- A new or changed handler/endpoint: its route and request/response types, not its body.

Include only changes big enough to matter for review. Omit renames, minor internal helpers, and anything a reviewer would not need to see to approve the shape of the change.

Give every step and every task its own `Status:` line, `not-started` or `completed`; a fresh plan is all `not-started`. Follow the Example below for the structure.

Write the file to `<workspace-root>/<project-name>-spex/plans/<spec-id>/<date>-<slug>.md` for a new plan (creating the spec's plan directory if needed), or update the existing file in place for an edit — never rename or duplicate it. `<date>` is today's real date (`YYYY-MM-DD`); `<slug>` is a short lowercase hyphenated summary; the filename stem must validate against `plan-filename.schema.json` and stay within 50 characters.

Tell the user the path you wrote (or the change you applied), walk through the step boundaries and which tasks run in parallel, then apply any requested edits in place.

## Example

Plan for `FS-RESET-PASSWORD` (scope: `api`, `frontend`):

````md
# Plan: Let users reset a forgotten password

- Specs: FS-RESET-PASSWORD
- Scope: api, frontend
- Rule-specs: RS-NAMING-CONVENTION, RS-SECRET-HANDLING, RS-SPEC-ID-TRACING

## Step 1 — Store reset tokens and expose the reset endpoints

Status: not-started

### Task 1.1 — Reset-token storage and endpoints (repo: api)

Status: not-started

New table `password_reset_tokens`:

| Column     | Type        | Constraints              |
| ---------- | ----------- | ------------------------ |
| token      | text        | PK                       |
| user_id    | uuid        | FK -> users.id, not null |
| expires_at | timestamptz | not null                 |
| used_at    | timestamptz | nullable                 |

New domain model in `src/api/models.rs`:

```rust
struct PasswordResetToken {
    token: String,
    user_id: UserId,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
}
```

New `PasswordResetRepository` methods in `src/api/password_reset_repo.rs`:

```rust
async fn create(&self, user_id: UserId, ttl: Duration) -> Result<PasswordResetToken, ApiError>;
async fn find_valid(&self, token: &str) -> Result<Option<PasswordResetToken>, ApiError>;
async fn mark_used(&self, token: &str) -> Result<(), ApiError>;
```

Changed `UserRepository` method in `src/api/user_repo.rs`, so a miss never errors and the handler can stay silent about unknown emails:

```rust
// before
async fn find_by_email(&self, email: &str) -> Result<User, ApiError>;
// after
async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApiError>;
```

New handlers in `src/api/handlers/password_reset.rs`:

```rust
async fn request_reset(body: RequestResetBody) -> Result<StatusCode, ApiError>; // POST /password-resets
async fn confirm_reset(body: ConfirmResetBody) -> Result<StatusCode, ApiError>; // POST /password-resets/confirm
```

Applies: RS-SECRET-HANDLING (token generation, never logged), RS-SPEC-ID-TRACING (annotate `request_reset` and `confirm_reset`, the boundary).

### Task 1.2 — Reset request and confirm UI (repo: frontend)

Status: not-started

New `pizzaApi` methods in `src/api.ts`:

```ts
requestPasswordReset(email: string): Promise<void>;
confirmPasswordReset(token: string, newPassword: string): Promise<void>;
```

New `ResetPasswordForm` component props in `src/components/ResetPasswordForm.tsx`:

```ts
interface ResetPasswordFormProps {
  token: string;
  onSubmit: (newPassword: string) => Promise<void>;
}
```

Applies: RS-NAMING-CONVENTION, RS-SPEC-ID-TRACING (annotate the new `pizzaApi` methods, the boundary).

## Step 2 — Sign out other sessions on reset

Status: not-started

Changed `SessionRepository` in `src/api/session_repo.rs` (repo: api):

```rust
// added
async fn delete_all_for_user_except(&self, user_id: UserId, keep: SessionId) -> Result<(), ApiError>;
```

Called from `confirm_reset` (Task 1.1), which already carries the spec annotation — RS-SPEC-ID-TRACING forbids re-annotating this repository method (no nested annotations).
````

After writing the plan, offer to execute it: run `spex-implement <spec-id>`. Note that it is better to start a new chat first to save context.

Follow the project's writing style: imperative and directive, clarity over brevity, brevity over all else. No filler, hedging, motivational, or apologetic phrasing.
