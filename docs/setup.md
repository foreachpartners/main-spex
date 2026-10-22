# Setup

Step-by-step setup of spex in your project. For the short version, see the [Quickstart](../README.md#quickstart).

## Prerequisites

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/) (required for building the spex CLI)
- **Git**: [Install Git](https://git-scm.com/install/)
- **Linux or macOS**: on Windows, use [WSL](https://learn.microsoft.com/windows/wsl/install); see [supported platforms](../specs/cli/install-cli.md)
- **AI coding agent**: for example Claude Code, Codex, Cursor, VS Code or Zed. The agent runs the spex skills.

## 1. Install spex

```sh
cargo install main-spex
spex --version
```

Running the install again upgrades the CLI. After an upgrade, run `spex update` in your project to update its skills, rules, and docs.

## 2. Choose a layout

`spex init` asks which layout the project uses. See the [FAQ](faq.md#which-layout-should-i-choose) for how to choose.

| Layout        | Project                                     | spex directory                      |
| ------------- | ------------------------------------------- | ----------------------------------- |
| `multi-repo`  | every package is its own git repository     | `<project-name>-spex`, its own repo |
| `single-repo` | the project is one git repository           | `spex/` in the repository           |
| `monorepo`    | one git repository holding several packages | `spex/` in the repository           |

## 3. Prepare the project root

The project root is the folder you run `spex init` from. Its name becomes the project name.

In `multi-repo`, put all packages (each a git repository) in the project root, each in a folder named `<project-name>-<package-short-name>`. Rename existing packages to match before continuing.

```text
shopping-notes/                    project root
  shopping-notes-api/              existing api package
  shopping-notes-frontend/         existing frontend package
  ...                              repeat for other packages
```

In `single-repo` and `monorepo`, the project root is the root of the git repository.

## 4. Run `spex init`

From the project root:

```sh
spex init
```

It asks you to confirm the project name (the project root folder name by default) and to pick the layout (`multi-repo` by default), then creates the spex directory.

In `multi-repo`, you get a new `shopping-notes-spex` folder next to the packages:

```text
shopping-notes/                    project root
  shopping-notes-api/
  shopping-notes-frontend/
  shopping-notes-spex/             spex directory and agent root
    spex.yaml
    shopping-notes.code-workspace
    AGENTS.md
    CLAUDE.md -> AGENTS.md
    .agents/skills/
    .claude/skills -> ../.agents/skills
    docs/  schemas/  rules/  specs/  plans/  contracts/
```

In `single-repo` and `monorepo`, you get a `spex/` folder, and `AGENTS.md`, `CLAUDE.md`, `.agents` and `.claude` in the repository root. Existing ones are kept.

```text
shopping-notes/                    project root and git repository
  spex/                            spex directory
  AGENTS.md
  CLAUDE.md -> AGENTS.md
  .agents/skills/
  .claude/skills -> ../.agents/skills
```

## 5. Open the workspace

Work across all packages at once and connect the AI coding agent. In `single-repo` and `monorepo`, open the repository and skip to the next step.

In `multi-repo`, the workspace file lists only the spex repository until `/spex-setup` adds the other packages:

- Cursor/VSCode: open the workspace from the file `shopping-notes-spex/shopping-notes.code-workspace`
- Cursor agentic: open the workspace file in editor mode first, then pick it in agentic mode
- Claude CLI: run `claude --add-dir ../shopping-notes-api ../shopping-notes-frontend` from the `shopping-notes-spex` repository
- Zed: add a project and pick all packages (folders)

## 6. Run `/spex-setup`

In the AI chat window, the agent should list the spex skills. If it does not, check that `.agents/skills/` exists in the agent root and that you opened the project from there.

Run the skill:

```text
/spex-setup
```

It finds your packages and proposes a `spex.yaml` entry for each one: description, contracts, and the rules that apply. After you confirm, it writes them, creates a skill for each package, and adds the packages to the workspace file. It then offers to write the project's PRD with `/spex-prd`, or to author your first spec or rule.

Review what the agent generated, fix what is wrong, and commit.

## 7. Check the result

```sh
spex info
```

It prints the project configuration, so you can see the packages that were added.

Then describe the product with `/spex-prd` (optional) and try your first feature with `/spex-spec`, or follow the [Tutorial](tutorial.md) to take the `shopping-notes` example from a plain project to a fully spec-driven one.
