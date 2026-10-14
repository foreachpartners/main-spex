# Publishing

`spex` is published on [crates.io](https://crates.io/crates/main-spex) as the crate `main-spex`, which installs the `spex` binary. Pushing a `v*` tag runs `.github/workflows/release.yml`, which runs `cargo publish`; merging to `dev` publishes nothing. A published version can never be republished or deleted.

## Each release

1. Bump `version` in `Cargo.toml` and add the matching `CHANGELOG.md` entry. `cargo test` updates `Cargo.lock` and checks that the latest changelog entry matches `version`.
2. Merge to `dev` and wait for CI to pass; the release workflow does not run the tests.
3. Tag the merged commit and push the tag:
   ```sh
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```
4. The workflow fails without publishing if the tag does not match `version` in `Cargo.toml`. Otherwise `cargo install main-spex` installs the new version within a minute or two.

## A bad release

Run `cargo yank --version X.Y.Z` so new installs skip it, then publish a fixed version. Yanking does not delete the version.
