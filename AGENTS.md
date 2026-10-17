# Main spex

The AI-first development framework that turns specifications into predictable code — keeping AI running at full speed while ensuring slop-free, verified results.

## Core Concepts

Main spex, or spex for short, is built on four simple pieces:

- **Spec** — one coherent capability the system must provide, with acceptance criteria proven by `spex coverage`. Specs are single source of truth for project - all behaviour, contracts, code, tests and other artifacts come from it and must adhere to it.
- **Rule** — one coherent cross-cutting convention (style, structure, security), with acceptance criteria proven by `spex check`. Each package must adhere to a list of rules applicable to it defined in `spex.yaml`
- **Skill** — the instructions an AI agent follows to do the job consistently and correctly.
- **Traceability** — the `@spec` annotations stitching together spec, contracts, implementations, and tests, used to ensure coverage and to look up any of spec, contract, implementation, or test artifacts.

## Specification

`specs/` is the single source of truth for how the framework, the CLI, and the skills behave, written as specs (description + acceptance criteria). Consult them before changing behavior, and keep them in sync when behavior changes. Also on changes update if needed `docs/tutorial.md` and `examples/shopping-notes`.

## Quick commands

```sh
just lint       # cargo clippy, warnings denied
just fmt        # Format everything (fmt-docs + cargo fmt)
just test       # cargo test
just fmt-docs   # dprint fmt (md/yaml/json)
just check-docs # dprint check (md/yaml/json)
just check-fmt  # Check formatting of everything (check-docs + cargo fmt --check)
just install    # Reinstall cli
just help       # spex --help
```

## Project structure

```
.github/workflows/  CI: tests and format checks on pull requests
src/                CLI source
  main.rs           Entry point, arg parsing
  error.rs          The shared eros Result alias, the one-line description of an error for users, the report printed for a failed command, and the wording shared by several commands
  templates.rs      Bundled templates directory and the names of its folders and files (constants only)
  spec/             Spec topic (mod.rs is an index only)
    id.rs           SpecId and AcId value objects and their id grammar
    spec_config.rs  SpecConfig, the raw spec file as written in YAML (loaded 1 to 1)
    spec.rs         Spec model built from SpecConfig, with its absolute path (status, acceptance criteria as {id, statement, is_done, line})
  rule/             Rule topic (mod.rs is an index only)
    id.rs           RuleId value object and its id grammar
    rule_config.rs  RuleConfig, the raw rule file as written in YAML (loaded 1 to 1)
    rule.rs         Rule model built from RuleConfig, with its absolute path
  tracing/          Traceability topic
    annotations.rs  @spec annotation grammar and parsing (shared by check and coverage)
  config/           Configuration topic (mod.rs is an index only)
    layout.rs       Layout (multi-repo, single-repo, monorepo): its spex dir name, agent root and choices
    spex_config.rs  SpexConfig, the raw spex.yaml as written in YAML (loaded 1 to 1)
    package_config.rs  PackageConfig, the raw optional spex.package.yaml at a package root (exclude, coverage_ignore, check_ignore), loaded 1 to 1
    spex_yaml_edit.rs  Text edits to spex.yaml that keep comments and field order (cli_version, check_ignore blocks)
    package_yaml_edit.rs  Text edit to spex.package.yaml that keeps comments and field order (check_ignore block)
    package.rs      Package model built from PackageEntry (the entry in spex.yaml), with absolute package and contract paths, its own exclude (compiled to the matcher the file walk uses), check_ignore and coverage_ignore
    spex.rs         Spex built from SpexConfig (spex dir, project name, resolved packages), with each package's spex.package.yaml merged into its own package on discovery, spec (attaching each package's coverage_ignore to the spec under the package's short name, by spec id) and rule loaders, package selection by name, and resolving package names to short names (shared by commands)
  support/          Cross-topic helpers
    acceptance_criteria_lines.rs  Source line of each acceptance criterion (shared by spec and rule)
    ai_view.rs      The --ai view of a loaded yaml file (path beside id, dropped fields; shared by specs and rules)
    commitment.rs   RFC 2119 commitments in order (shared by spec and rule)
    fs.rs           Filesystem operations (file writes, dir creation, paths, path normalizing, relative paths, file URLs, embedded dir extraction, symlinks, exec bits)
    glob.rs         Glob set building from patterns (shared by a check's include and exclude and by a package's exclude)
    managed_section.rs  The `# Project` marker split of AGENTS.md and skills: replacing the managed section, keeping existing content below the marker (shared by init and update)
    package_files.rs   Iterator over a package's files (gitignore, dotfiles, skipped names, the package's exclude) and the Exclude matcher
    style.rs        Terminal hyperlinks, version line, dim/red styles, terminal width and visible-width measuring (shared by commands)
    testing.rs      Shared unit-test builders for specs, rules, packages and config (test builds only)
    text.rs         Text helpers (word wrapping, pluralize, markdown level two sections)
    yaml_dir.rs     Yaml file walking and loading of a directory (YamlFile: parsed item, path, raw text)
  commands/         Subcommands, one per CLI command; a command with more than one file gets a folder whose mod.rs is an index only (info and jq are single files)
    init/ check_schema/ coverage/ rules/ specs/   Command implementation plus its tests (init also holds scaffold, the fresh copy of bundled templates whose copy steps update reuses, and agent_files, the agent root's existing `.claude`, `AGENTS.md` and `CLAUDE.md`: conflict checks, adoption, links; coverage also holds annotation_index, the project scan of annotations; spec_coverage, per-spec and per-criterion coverage; report, the per-status and per-package grouping of spec coverage and the exit code; render, the report text)
    update/         update command: orchestration and changelog (update), the bundled migration guide since a version (migration_guide), conditional rewrite of bundled templates (sync_templates)
    check/          check command: orchestration (check), finding (its typed source and location), scoped file, baseline matching and fill, package selection, report, spec-id annotation tracing, fixtures (shared test builders for scoped files, checks and scripts; test builds only)
      conformance/  Structural checks on specs/rules themselves (ids, ac order, property order, schema, script refs, status, blocked_by, packages, coverage_ignore); findings are rebased to spex-relative locations once, in conformance.rs (its mod.rs is an index only)
      runners/      One runner per check type (ast_grep, glob, regexp, script) plus the shared forbidden/required hit reporting (mod)
specs/                         Single source of truth for behavior, one md file per spec (description + acceptance criteria), named `<name>-<group>`
  framework/               Artifact chain, traceability, managed section
  project/                 Layout
  config/                  spex.yaml, spex.package.yaml, CLI version
  spec/                    Spec file, spec annotations, coverage states
  rule/                    Rule file, rule checks
  cli/                     Shared behavior (command set, project discovery, exit codes), install, publishing, one `spex-<command>-cli-cmd` file per command
  skill/                   Bundled skills (one `<skill-name>-skill` file per skill), package skill
docs/                          Framework reference docs; only those symlinked from templates/docs/ are bundled into projects by init/update
  glossary.md              Terms used across code, messages, schemas, and docs
  prd.md                   Product requirements: problem, goal, success, capabilities, principles, values, non-goals
  tutorial.md              Hands-on walkthrough of adopting spex
  faq.md                   Frequently asked questions
templates/                     Assets scaffolded into projects
  schemas/                     JSON Schemas for spex.yaml, specs, rules, and plans
  skills/                      Agent skills
  rules/                       Reusable rule templates, each with its script folder (same name, beside the rule file) when it has scripts
  docs/                        Framework docs shipped to projects (symlinks into docs/)
  AGENTS.md                    Seeds the project's AGENTS.md (and CLAUDE.md symlink) on init; update syncs its managed section
  README.md                    Contributor notes on scaffolding rules; never scaffolded
  spex.yaml                    Project config seeded by init for every layout, listing only the spex package (project name, layout, agent root, package name filled in)
  project-name.code-workspace  Editor workspace file seeded by init (multi-repo only)
examples/                      Sample example projects with/without spex(shopping-notes, shopping-notes-no-spex)
tests/                         Integration tests that build and run the spex binary
  cli/                         The one integration test binary; main.rs is an index only
    common.rs                  Helpers shared across commands (running spex, copying fixture projects)
    <command>.rs               One file per command with all its tests and helpers
    check/                     One file per check type (regexp, glob, ast_grep, script), declared as modules of check.rs
  fixtures/                    Projects the integration tests copy and run spex against
  changelog_format.rs          CHANGELOG.md format test, not tied to a command
```

Unit tests live next to the code they cover as `*_test.rs` files. End-to-end
tests that drive the compiled binary live in `tests/cli/`, split by the command
they belong to.

Keep this section in sync when the structure changes.

## Rules

- **Simplicity**: All technical assets, tools, and processes must be simple and intuitive for any teammate or AI to quickly understand, execute, and safely change.
- **Single source of truth**: every business rule, behavior, contract, and configuration value must be mastered in exactly one place. Any other system that needs this information must reference, link to, or generate from this primary source rather than duplicating it. Repeating wording or an idea for readability is not duplication.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables instead of comments. Never add a comment — not for WHAT the code does, not for WHY it exists, not to note an extraction, refactor, or design rationale. Write no comments at all unless the user explicitly asks for one in that exact request.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Systemic Fix Over Quick Fix**: Always implement the proper, maintainable solution. Never apply a workaround, hack, band-aid, or temporary fix, unless the user explicitly asks for one in that request. If the proper fix requires refactoring, architectural changes, or expanding the scope of the current task, stop immediately, present the proper solution, and ask the user for explicit permission before proceeding.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless triggered by the "Systemic Fix Over Quick Fix" rule.
- **Stop on major findings**: When investigation reveals a major problem — a bug, a spec/code mismatch, a failing check, a design flaw — related to the task or not, stop. Report it with evidence and ask the user how to proceed. Never fix it silently, work around it, or ignore it. Report minor problems at the end of the task.
- **Propose major improvements**: When, at any step, a substantially simpler or better approach than the requested one emerges, stop, propose it, and wait for the user's decision before continuing. Propose a behavior change as a spec change first, since the spec is the source of truth. Report minor improvements at the end of the task.
- **Glossary Terms**: Use the terms defined in `docs/glossary.md` consistently across code, messages, schemas, and docs.
- **Use changelog**: On version change fill `CHANGELOG.md`; a change that doesn't bump the version gets no entry
- **Keep CLI commands in sync with template**: On CLI command or option changes, sync the CLI commands list in `templates/AGENTS.md`
- **Migrations for agent**: When a change requires migration steps in a spex project, fill `templates/skills/spex-update/references/migration_guide.md` and update `templates/spex.yaml`
- **Specification is single source of truth**: `specs/` defines how the framework, the CLI, and the skills behave. Treat it as authoritative, and update it whenever behavior changes.
- **Bump the CLI version**: Bump `version` in `Cargo.toml` only for a public-facing change (one a user of the CLI or of a spex project can observe: commands, options, output, file formats and schemas, the templates `spex update` writes) or a major one (breaking, or needing migration steps). Never for internal refactors, tests, CI, examples, typos, or wording-only edits. Scaffolded `spex.yaml` records this as `cli_version`, and spex warns the user to run `spex update` when the installed CLI differs from the version that wrote their config.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.
