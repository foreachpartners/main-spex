---
name: spex-update
description: Updates a spex project's CLI version and applies the migration steps that upgrade requires. Use when the user wants to update spex, upgrade to a new spex version, or run `spex update`.
---

# Process

**You MUST follow each process step without skipping.**

## Step 1 — Verify a clean baseline

Run `spex check` and `spex coverage`. If either reports a failure, stop and ask the user whether to fix the existing issues first or update anyway. A failure to load `spex.yaml` because it lacks a field the installed CLI requires is expected after upgrading the binary: go to Step 2, since Step 3 applies the migration steps that add the field before Step 4 runs.

## Step 2 — Record the current version

Read `cli_version` from `<spex-dir>/spex.yaml`. This is the previous version — record it before Step 4 overwrites it.

## Step 3 — Migrate

Run `spex update --migration-guide <previous version>`. It prints the installed binary's migration steps for every version after the previous version, oldest first, and neither reads `spex.yaml` nor changes any file.

Apply the printed steps in the printed order, before Step 4: a breaking change can leave the project in a state `spex update` does not handle. Empty output needs no migration; a listed step that doesn't apply to this project (e.g. it names a feature the project doesn't use) needs no action.

## Step 4 — Update

Run `spex update`. It bumps `cli_version` in `spex.yaml` and rewrites every bundled schema, doc, rule (with its scripts) and skill.

## Step 5 — Verify

Run `spex check` and `spex coverage` again. Fix any issues they report before finishing.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
