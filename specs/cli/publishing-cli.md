# Publishing

`spex` is published on [crates.io](https://crates.io/crates/main-spex) as the crate `main-spex`, which installs the `spex` binary.

## Acceptance criteria

- MUST publish only when a `v*` tag is pushed: `.github/workflows/release.yml` runs `cargo publish`.
- MUST bump `version` in `Cargo.toml` and add the matching `CHANGELOG.md` entry for each release; `cargo test` updates `Cargo.lock` and checks that the latest changelog entry matches `version`.
- MUST merge the release to `dev` and wait for CI to pass before tagging, since the release workflow does not run the tests.
- MUST tag the merged commit `vX.Y.Z` and push the tag.
- MUST fail the release workflow without publishing when the tag does not match `version` in `Cargo.toml`.
- MUST yank a bad release with `cargo yank --version X.Y.Z` so new installs skip it, then publish a fixed version; yanking does not delete the version.
- MUST NOT publish on merge to `dev`.
- MUST NOT republish or delete a published version: it can never be done.
- SHOULD make `cargo install main-spex` install a new version within a minute or two of the tag.

## Examples

```sh
git tag vX.Y.Z
git push origin vX.Y.Z
```
