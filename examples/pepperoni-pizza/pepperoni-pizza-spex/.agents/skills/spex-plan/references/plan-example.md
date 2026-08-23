# Plan: Let users reset a forgotten password

- Specs: RESET-PASSWORD
- Repositories scope: api, frontend
- Rules: NAMING-CONVENTION, SECRET-HANDLING, SPEC-ID-TRACING

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

## Step 2 — Sign out other sessions on reset

Status: not-started

Changed `SessionRepository` in `src/api/session_repo.rs` (repo: api):

```rust
// added
async fn delete_all_for_user_except(&self, user_id: UserId, keep: SessionId) -> Result<(), ApiError>;
```

Called from `confirm_reset` (Task 1.1), which already carries the spec annotation — SPEC-ID-TRACING forbids re-annotating this repository method (no nested annotations).
