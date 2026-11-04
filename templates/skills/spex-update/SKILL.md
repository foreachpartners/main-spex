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

Run `spex update --migration-guide <previous version>`. It prints the installed binary's migration steps for every version after the previous version, newest first, and neither reads `spex.yaml` nor changes any file.

Read all the printed steps, then apply them from the oldest version to the newest, the reverse of the printed order, before Step 4: a breaking change can leave the project in a state `spex update` does not handle. The output `no migration needed` needs no migration; a listed step that doesn't apply to this project (e.g. it names a feature the project doesn't use) needs no action.

## Step 4 — Update

Run `spex update`. It bumps `cli_version` in `spex.yaml` and rewrites every bundled schema, doc and skill, and every rule (with its scripts) the project holds that is also in the rules library, unless it sets `sync_with_upstream: false`. It never adds a rule.

## Step 5 — Offer new rules

The changelog `spex update` printed announces each rule added to the rules library. If it announced none, go to Step 6.

Read each new rule with `spex lib rules --id <rule-id>` and suggest to the user which packages should apply it: only those it genuinely applies to, given the package's stack and role (for example `openapi-conventions` only for a package that exposes a REST API), with a one-line reason each. Ask once for all new rules. Take every rule the user wants for at least one package with `spex lib rules --id <rule-id>... --copy <spex-dir>/rules`, and add its `id` to `rules` of every package the user chooses in `spex.yaml`. Leave the other new rules in the library.

Tell the user which packages apply each new rule and which new rules were left out.

## Step 6 — Verify

Run `spex check` and `spex coverage` again. Fix any issues they report before finishing.

`spex check` warns on each rule no package lists. Ask the user before removing one, as the `spex-rule` skill does in its "Remove a rule" section.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
