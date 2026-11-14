---
name: spex-review-spec
description: Review the current implementation of a spec — is what is implemented of the spec's acceptance criteria correct? Use when the user wants to review implementation of a spec.
---

Spec-axis review of the current branch against a spec — is what is implemented of the spec's acceptance criteria correct?

## Process

### 1. Resolve the spec

Use the spec the user provided. Otherwise resolve it from the current work: the feature branch diff against its base branch (`git diff <base-branch>...HEAD`) and the uncommitted changes (`git diff HEAD`). Take the specs changed in them and the specs named by `@spec` annotations in the changed code. Ask the user when no spec resolves.

### 2. Fetch the spec

Run `spex specs --id <spec-id-a> <spec-id-b>` for the resolved spec(s). Read the PRD in full, at the path `spex info` prints as `prd`, for the product's goal behind the spec.

### 3. Find the implementation

Search the packages listed in the spec's `packages` for code annotated `@spec impl <spec-id>`. Review it as it stands on the current branch.

### 4. Check against the contract

Skip when the spec has no contract. Otherwise find the contract annotated `@spec contract <spec-id>` in the contracts of the spec's `packages`. Check only what is implemented.

- Design reference (Figma frame or mockup): open the linked frame or mockup page and capture it. Run the implementation and capture the same screen at the design's size, in every state the design and the acceptance criteria cover. Compare the two and report differences in layout, content, and states. Ignore rendering noise such as font smoothing or 1px shifts.
- API contract (OpenAPI): compare each operation, status code, field, and error with the code.
- Use whatever Figma access, browser, and screenshot tooling the agent has. When there is none or the app cannot run, skip the comparison and list it in one "Not verified: design" line. Do not judge a design from code alone.

Report each difference as a finding. Quote the acceptance criterion and the contract element or frame note it breaks.

### 5. Review

Against the implementation, contract and the spec contents, report the acceptance criteria whose implementation looks wrong. Quote the acceptance criterion for each finding.

- A spec may be only partly implemented. An acceptance criterion with no implementation yet or implemented partially (no `@spec impl <spec-id>#<ac-id>`) is not a finding, as long as what is implemented is correct. List these in one "Not yet done" line.
- Tests come in later steps as the last verification layer, so missing tests are not a finding.

End with a one-line summary: total findings, and the worst issue (if any).

---

# Project

Everything above `Project` is overridden on `spex update`, any project specific data should be added here
