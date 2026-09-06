# Changelog

Notable changes to spex, following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions match `Cargo.toml` (`cli_version` in `spex.yaml`). `spex update` prints the entries newer than the version it's updating from.

## [0.7.83] - 2026-09-06

- `commit-message-format` rule: replace the `spec`/`rule` scope restriction with free-form scope plus `@spec`/`@rule` ID tags on spec/rule-related commits.

## [0.7.82] - 2026-09-06

- `spex-spec` skill: for a multi-repository spec, name each repository in its acceptance criteria; omit the repository name for a single-scoped spec since it's already redundant with `scope`.
