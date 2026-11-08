---
name: spex-grill-me
description: Grill the user relentlessly about a plan, decision, or idea. Use when the user wants to stress-test their thinking, or uses any 'grill' trigger phrases.
---

Interview me relentlessly about every aspect of this until we reach a shared understanding. Walk down each branch of the decision tree, resolving dependencies between decisions one-by-one. For each question, provide your recommended answer.

Ask the questions one at a time, waiting for feedback on each question before continuing. Asking multiple questions at once is bewildering.

If a _fact_ can be found by exploring the environment (filesystem, tools, etc.), look it up rather than asking me. The _decisions_, though, are mine — put each one to me and wait for my answer.

When the subject touches something that already exists in the project (a capability, endpoint, behavior, data, or package), look up what the project's specs and contracts already say about it before you ask:

- Find the related specs with `spex specs --fields id title description --output <tmp-path>` (narrow it with `--search <keyword>...`), then read each in full with `spex specs --id <spec-id>...`.
- Read the contracts of the affected packages: `spex info --package <package-name>` lists them under `contracts`, each with its `path`.

Specs are the source of truth, so grill against them. Treat what a spec or contract already settles as a fact: state it and move on instead of asking. Put to me, with your recommendation, each place where my plan contradicts an acceptance criterion or a contract, where they leave a gap, or where a spec or contract would have to change first. Never change a spec or contract while grilling.

Do not act on it until I confirm we have reached a shared understanding.

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
