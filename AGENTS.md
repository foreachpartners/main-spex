# Main spex

The AI-first development framework that turns natural specifications into deterministic code — keeping AI running at full speed while ensuring slop-free verifiable results.

## Core Concepts

Main spex, or spex for short, is built on four simple pieces:

- **Spec** — one coherent capability the system must provide, with acceptance criteria proven by `spex coverage`. Specs are single source of truth for project - all behaviour, contracts, code, tests and other artifacts come from it and must adhere to it.
- **Rule** — one coherent cross-cutting convention (style, structure, security), with acceptance criteria proven by `spex check`. Each repository must adhere to a list of rules applicable to it defined in `spex.yaml`
- **Skill** — the instructions an AI agent follows to do the job consistently and correctly.
- **Traceability** — the `@spec` annotations stitching together spec, contracts, implementations, and tests, used to ensure coverage and to look up any of spec, contract, implementation, or test artifacts.

## Specification

`docs/specification.md` is the single source of truth for how the framework behaves, written as specs (description + acceptance criteria), and `docs/cli/` for the CLI's command behavior. Consult them before changing behavior, and keep them in sync when behavior changes. Also on changes update if needed `docs/tutorial.md` and `examples/shopping-notes`.

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
    spex_config.rs  SpexConfig, the raw spex.yaml as written in YAML (loaded 1 to 1)
    repo_config.rs  RepoConfig, the raw optional spex.repo.yaml at a repository root (coverage_ignore, ignored_errors), loaded 1 to 1
    spex_yaml_edit.rs  Text edits to spex.yaml that keep comments and field order (cli_version, ignored_errors blocks)
    repository.rs   Repository model built from the raw repository config, with absolute repository and contract paths
    spex.rs         Spex built from SpexConfig (spex dir, project name, resolved repositories, coverage_ignore), with each repository's spex.repo.yaml merged in on discovery, spec (adding the coverage_ignore by spec id) and rule loaders, repository selection by name, and scope validation (shared by commands)
  support/          Cross-topic helpers
    acceptance_criteria_lines.rs  Source line of each acceptance criterion (shared by spec and rule)
    ai_view.rs      The --ai view of a loaded yaml file (path beside id, dropped fields; shared by specs and rules)
    commitment.rs   RFC 2119 commitments in order (shared by spec and rule)
    fs.rs           Filesystem operations (file writes, dir creation, paths, relative paths, file URLs, embedded dir extraction, symlinks, exec bits)
    repo_files.rs   Iterator over a workspace repository's files (gitignore, dotfiles, skipped names)
    style.rs        Terminal hyperlinks, version line, dim/red styles, terminal width and visible-width measuring (shared by commands)
    testing.rs      Shared unit-test builders for specs, rules, repositories and config (test builds only)
    text.rs         Text helpers (word wrapping, pluralize)
    yaml_dir.rs     Yaml file walking and loading of a directory (YamlFile: parsed item, path, raw text)
  commands/         Subcommands, one per CLI command; a command with more than one file gets a folder whose mod.rs is an index only (info and jq are single files)
    init/ check_schema/ coverage/ rules/ specs/   Command implementation plus its tests (init also holds scaffold, the fresh copy of bundled templates whose copy steps update reuses; coverage also holds annotation_index, the workspace scan of annotations; spec_coverage, per-spec and per-criterion coverage; report, the per-status and per-scope grouping of spec coverage and the exit code; render, the report text)
    update/         update command: orchestration and changelog (update), conditional rewrite of bundled templates (sync_templates)
    check/          check command: orchestration (check), finding (its typed source and location), scoped file, baseline matching and fill, repo selection, report, spec-id annotation tracing, fixtures (shared test builders for scoped files, checks and scripts; test builds only)
      conformance/  Structural checks on specs/rules themselves (ids, ac order, property order, schema, script refs, status, blocked_by, scope, coverage_ignore); findings are rebased to spex-relative locations once, in conformance.rs (its mod.rs is an index only)
      runners/      One runner per check type (ast_grep, glob, regexp, script) plus the shared forbidden/required hit reporting (mod)
docs/                          Framework reference docs (single source of truth); only those symlinked from templates/docs/ are bundled into projects by init/update
  glossary.md              Terms used across code, messages, schemas, and docs
  framework-principles.md  Non-negotiable foundation spex is built on
  conventions.md           Engineering values spex follows but didn't invent
  specification.md         Framework behavior, minus the CLI
  cli/                     Single source of truth for CLI behavior: README.md (overview and shared behavior), install.md, development.md, one file per command
  tutorial.md              Hands-on walkthrough of adopting spex
  faq.md                   Frequently asked questions
templates/                     Assets scaffolded into projects
  schemas/                     JSON Schemas for spex.yaml, specs, rules, and plans
  skills/                      Agent skills
  rules/                       Reusable rule templates, each with its script folder (same name, beside the rule file) when it has scripts
  docs/                        Framework docs shipped to projects (symlinks into docs/)
  AGENTS.md                    Seeds the project's AGENTS.md (and CLAUDE.md symlink) on init; update syncs its managed section
  README.md                    Contributor notes on scaffolding rules; never scaffolded
  spex.yaml                    Workspace config seeded by init (project name filled in)
  project-name.code-workspace  Editor workspace file seeded by init
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
- **Single source of truth**: every piece of data, logic, or business rule must be mastered in exactly one place. Any other system that needs this information must reference, link to, or generate from this primary source rather than duplicating it.
- **KISS** (Keep It Simple, Stupid): Prefer the simplest solution that works. Avoid unnecessary abstraction, comments, indirection, and premature generalization.
- **DRY** (Don't Repeat Yourself): Extract shared logic into a single source of truth instead of duplicating it. Reuse existing code, types, and systems before adding new ones.
- **Self-Documenting Code**: Express intent through clear names for types, functions, and variables instead of comments. Never add a comment — not for WHAT the code does, not for WHY it exists, not to note an extraction, refactor, or design rationale. Write no comments at all unless the user explicitly asks for one in that exact request.
- **Ask to clarify**: When a request is ambiguous, underspecified, or could be reasonably interpreted in more than one way, ask for clarification before proceeding instead of guessing.
- **Systemic Fix Over Quick Fix**: Always implement the proper, maintainable solution. Never apply a workaround, hack, band-aid, or temporary fix, unless the user explicitly asks for one in that request. If the proper fix requires refactoring, architectural changes, or expanding the scope of the current task, stop immediately, present the proper solution, and ask the user for explicit permission before proceeding.
- **Change only what the task requires**: don't refactor, rename, or reformat adjacent code that isn't part of it unless triggered by the "Systemic Fix Over Quick Fix" rule.
- **Stop on major findings**: When investigation reveals a major problem — a bug, a spec/code mismatch, a failing check, a design flaw — related to the task or not, stop. Report it with evidence and ask the user how to proceed. Never fix it silently, work around it, or ignore it. Report minor problems at the end of the task.
- **Propose major improvements**: When, at any step, a substantially simpler or better approach than the requested one emerges, stop, propose it, and wait for the user's decision before continuing. Propose a behavior change as a spec change first, since the spec is the source of truth. Report minor improvements at the end of the task.
- **Glossary Terms**: Use the terms defined in `docs/glossary.md` consistently across code, messages, schemas, and docs.
- **Use changelog**: On version change fill `CHANGELOG.md`
- **Keep CLI commands in sync with template**: On CLI command or option changes, sync the CLI commands list in `templates/AGENTS.md`
- **Migrations for agent**: When a change requires migration steps in a spex project, fill `templates/skills/spex-update/references/migration_guide.md` and update `templates/spex.yaml`
- **Specification is single source of truth**: `docs/specification.md` defines how the framework behaves and `docs/cli/` how the CLI behaves. Treat both as authoritative, and update them whenever behavior changes.
- **Bump the CLI version**: Whenever you change the CLI's behavior, bump `version` in `Cargo.toml`. Scaffolded `spex.yaml` records this as `cli_version`, and spex warns the user to run `spex update` when the installed CLI differs from the version that wrote their config.
- **Never use a destructive command**: (`rm -rf`, force push, hard reset, branch delete) to route around an obstacle — investigate first, ask if unsure.
