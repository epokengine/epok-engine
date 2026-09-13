# Development and release branches

Epok uses `develop` for continuous integration and `release` for published
versions. Feature and fix branches merge into `develop`. A release is promoted
only with a pull request from `develop` to `release`.

## Version rule

`VERSION` is the authoritative SemVer version and must match the `[package]`
version in `Cargo.toml`. A pull request into `release` passes only when its
version is greater than both the version already on `release` and every `vX.Y.Z`
tag reachable from `release`.

After the pull request merges, GitHub Actions creates the annotated tag
`vX.Y.Z`. Git tags therefore describe merged release commits; they are not put
on unmerged pull requests.

Before opening a release pull request:

1. Update `VERSION` and `Cargo.toml` to the same new SemVer value.
2. Update `Cargo.lock` by running `cargo check` or `cargo update -p epok-editor`.
3. Run `make check` and the relevant integration tests.
4. Open a pull request from `develop` into `release`.

## Local build state

Builds launched with `make build`, `make release`, `make check`, or
`make build-psx` update `.epok/build-state.json` after they succeed. The file is
local and ignored by Git. It contains the current version, the `develop` base
branch and commit, and the number of successful builds made from that base.

Run `make build-state` to inspect it or `make reset-build-state` to start the
local counter again. Direct `cargo` commands are intentionally not counted.
