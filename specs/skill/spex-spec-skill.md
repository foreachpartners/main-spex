# `spex-spec` skill

Creates new specs, or extends and changes existing ones, by interviewing the user. Loaded when the user wants to extend or change existing specs or create new ones, from a PRD, an RFD, a feature idea, a business rule, or a technical capability the product needs. It turns a fuzzy idea or PRD into schema-valid specs by asking the user focused questions, then writes one file per capability, or updates the named existing spec in place.

## Acceptance criteria

### Paths and scope

- MUST read `<spex-dir>/schemas/spec.schema.json`, the authority for field rules with worked examples, before drafting.
- MUST write a new spec to `<spex-dir>/specs/<spec-id>.yaml`.
- MUST treat a specific capability the system must provide (an endpoint, integration, health check, performance target) as a spec, and a cross-cutting rule all code must obey, enforceable package-wide, as a rule.
- MUST capture exactly one coherent capability per spec, its specifics being the acceptance criteria.
- MUST identify, when the source covers more than one capability, each distinct one, propose the split as a list of specs (one `id` and one-line summary each), confirm it with the user before drafting, and write each confirmed capability as its own spec file.

### Step 0: establish create vs edit

- MUST treat the request as an edit when the user names an existing spec ID (e.g. `place-pizza-order`) or asks to update, extend, fix, or change a spec.
- MUST read `<spex-dir>/specs/<spec-id>.yaml` in full before doing anything else, for an edit, confirm with the user what should change (new or revised acceptance criteria, a scope change, a status change), and skip straight to Step 4, editing the existing file in place and preserving every untouched field.
- MUST treat the request otherwise as a new spec and continue with Step 1.

### Step 1: gather context

- MUST treat a source the user attached or pasted (a PRD, an RFD, or a feature description), as text or as a file path or link, as the primary input: read it in full and pre-fill every answer it supports before asking anything.
- MUST ask first what the user wants to spec when they provided no source and nothing to spec.
- MUST read `<spex-dir>/spex.yaml` for the project overview, and each package's `description` to learn its role and determine the spec's implementation `scope`.
- MUST run `spex specs --ai --output <tmp-path>`, then read `<tmp-path>` for related specs.
- MUST read the interface contracts listed per package under `contracts` in `spex.yaml`, to learn the existing interfaces the spec relates to.
- MUST run `spex rules --id writing-style --ai` for the writing conventions the spec's prose follows.
- MUST NOT invent a capability or proceed on assumptions.

### Step 2: identify the capabilities and clarify missing details

- MUST break the source into distinct capabilities, one spec per capability, and confirm the split with the user before drafting when there is more than one.
- MUST pre-fill every field it can from the source and the existing context, then ask the user only about what is missing or ambiguous (unresolved details or acceptance criteria it cannot infer).
- MUST confirm anything it inferred rather than assuming silently.

### Step 3: draft the fields

- MUST draft each spec following the field rules in `<spex-dir>/schemas/spec.schema.json`, using its `examples` as the model for the YAML.
- MUST give the `id` a name that conveys the capability so it reads without opening the spec, unique enough that a global search finds no other matches, preferring two or more words with a dash.
- MUST write `acceptance_criteria` as a list of `{id, must|must_not|should|should_not|may, is_done}` objects, with exactly one commitment field present whose value is the verifiable statement, ordered MUST, MUST NOT, SHOULD, SHOULD NOT, MAY.
- MUST give each criterion an `id`, a kebab-case slug unique within the spec that names the criterion's behavior, not its position (e.g. `auth-required`, not `ac-1`).
- MUST give every criterion it writes, in a new spec or as a new criterion added to an existing one, `is_done: false`, since nothing is implemented yet; it keeps the criterion pending until its impl annotation exists and keeps the spec from being `active`, and `spex-test` clears it once `spex coverage` shows the criterion fully traced.
- MUST capture, verbatim and as its own verifiable criterion, a concrete decision the user gave (a constant, default, threshold, rate, or rule, e.g. each pizza takes one minute to make, two workers are on shift, the estimate is rounded to whole minutes).
- MUST ask for a detail needed to make a criterion verifiable when it is missing (Step 2).
- MUST describe end behavior, verifiable from outside (what the system does or exposes), not the implementation steps taken to build it.
- MUST set `scope` to the packages that implement the capability, each matching a `package_short_name` in `spex.yaml`.
- MUST name, when `scope` lists more than one package, the package that owns each acceptance criterion at the start of its text, so the boundary between packages is unambiguous and no shared behavior is left unassigned, splitting a behavior that crosses the boundary into separate per-package criteria (e.g. the API owns producing a value and the frontend owns presenting it).
- MUST omit the package name from the acceptance criteria of a single-scoped spec, since it is already redundant with `scope`.
- MUST set `blocked_by` only for specs that must be in place before this one can be implemented and whose status is not `active`, and leave it absent when nothing genuinely blocks the work.
- MUST set `contract`, a required field, to `single` (the migration default; one `@spec contract <spec-id>` annotation for the whole spec, usual for a normal API spec), `for-each-ac` (a `@spec contract <spec-id>#<ac-id>` annotation per criterion, when each criterion exposes its own contract surface that must be traced individually), or `none` (no contract coverage, when the spec has no interface contract at all).
- MUST keep `description` short, simple, and concrete: a general summary of the capability and who or what is affected.
- MUST add `examples` only when a request/response, payload, or usage snippet clarifies the capability better than prose, each entry keyed with a short label (e.g. `request`, `response`).
- MUST NOT soften a value the user supplied into a vague directional statement, or invent a value the user did not supply.
- MUST NOT write an internal build step (installing a package, adding a dependency, creating a file, writing a migration) as a criterion.
- MUST NOT set `is_done: false` on a criterion the user did not ask it to add or change.
- MUST NOT list a shared foundation that every capability rides on (e.g. the API surface), or an already-shipped or established spec, in `blocked_by`.
- MUST NOT restate in `description` the specifics already covered by `acceptance_criteria`, or the step-by-step implementation, which belongs to the plan.

### Step 4: write, then offer edits

- MUST write each drafted or edited spec to `<spex-dir>/specs/<spec-id>.yaml`, the filename equal to the `<spec-id>`, using literal block scalars for a multi-paragraph `description`.
- MUST change, when editing an existing spec, only what the user asked for, leaving every other field, its ordering, and formatting untouched.
- MUST validate the result against `<spex-dir>/schemas/spec.schema.json`.
- MUST tell the user the path it wrote (or the diff it applied, for an edit), ask whether they want to change anything further, and apply any requested edits in place.

### Step 5: point to the next skill

- MUST offer to draft each spec's contract with `spex-contract <spec-id>`, noting that starting a new chat first saves context.
- MUST follow the `writing-style` rule read in Step 1.
