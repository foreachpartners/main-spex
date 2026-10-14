# Install

How a user gets the `spex` binary, and the platforms and tools it needs.

## Acceptance criteria

- MUST support Linux and macOS, both built and tested in CI.
- MUST support Windows only through [WSL](https://learn.microsoft.com/windows/wsl/install), run as on Linux.
- MUST require [Rust](https://rust-lang.org/tools/install/) and [Git](https://git-scm.com/install/) as prerequisites.
- MUST install the latest release from [crates.io](https://crates.io/crates/main-spex) with `cargo install main-spex`; running it again upgrades.
- MUST report an error naming the missing tool when `spex check` runs a bundled rule's check whose external tool is not installed; a project that adopts a bundled rule needs the external tools its checks run.
- MUST allow installing unreleased changes from a checkout with `cargo install --path . --force`.
- MUST NOT support a native Windows build, because `script` checks run POSIX `sh` scripts directly and `init` links `CLAUDE.md` to `AGENTS.md` with a symlink.

## Examples

```sh
cargo install main-spex
```

```sh
git clone https://github.com/foreachpartners/main-spex
cd main-spex
cargo install --path . --force
```
