# Install

## Supported platforms

Linux and macOS, both built and tested in CI. On Windows, run spex inside [WSL](https://learn.microsoft.com/windows/wsl/install) as on Linux; a native Windows build is not supported, because `script` checks run POSIX `sh` scripts directly and `init` links `CLAUDE.md` to `AGENTS.md` with a symlink.

## Prerequisites:

- **Rust**: [Install Rust](https://rust-lang.org/tools/install/)
- **Git**: [Install Git](https://git-scm.com/install/)

## Tools used by bundled rules

A project that adopts a bundled rule needs the external tools its checks run. `spex check` reports an error naming the missing tool.

## Install the `spex` binary from source.

```sh
git clone https://github.com/foreachpartners/main-spex
cd main-spex
cargo install --path . --force
```
