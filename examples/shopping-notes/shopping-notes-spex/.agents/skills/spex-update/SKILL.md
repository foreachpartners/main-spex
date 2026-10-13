---
name: spex-update
description: Updates a spex project's CLI version and applies the migration steps that upgrade requires. Use when the user wants to update spex, upgrade to a new spex version, or run `spex update`.
---

# Process

**You MUST follow each process step without skipping.**

## Step 1 — Verify a clean baseline

Run `spex check` and `spex coverage`. If either reports a failure, stop and ask the user whether to fix the existing issues first or update anyway. A failure to load `spex.yaml` because it lacks a field the installed CLI requires is expected after upgrading the binary: go to Step 2, since Step 3 still runs and Step 4 adds the field.

## Step 2 — Record the current version

Read `cli_version` from `<spex-dir>/spex.yaml`. This is the previous version — record it before Step 3 overwrites it.

## Step 3 — Update

Run `spex update`. It bumps `cli_version` in `spex.yaml` and rewrites every bundled schema, doc, rule (with its scripts) and skill — including this skill's `references/migration_guide.md`, which now holds the latest steps.

Read `cli_version` from `spex.yaml` again. This is the current version.

## Step 4 — Migrate

Re-read `references/migration_guide.md` — Step 3 just refreshed it. Apply, in order, every version section strictly greater than the previous version and up to and including the current version. A version missing from the guide needs no migration; a listed step that doesn't apply to this project (e.g. it names a feature the project doesn't use) needs no action.

## Step 5 — Verify

Run `spex check` and `spex coverage` again. Fix any issues they report before finishing.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
