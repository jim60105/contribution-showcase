## Why

This project currently has no CI/CD automation. Every build, test, lint check, and release must be run manually, which is error-prone and slows down development. Adding GitHub Actions pipelines will automate quality gates on every push/PR and streamline the release process — producing cross-platform binaries and publishing to crates.io on tagged releases.

## What Changes

- Add a **CI workflow** (`build-test-audit.yml`) that runs on push to `master`/`dev` and on pull requests targeting those branches:
  - Checks formatting (`cargo fmt -- --check`)
  - Runs Clippy linting (`cargo clippy --all-targets -- -D warnings`)
  - Runs all tests (`cargo test`) across a matrix of OS targets (ubuntu, windows, macos)
  - Performs a security audit of Rust dependencies
- Add a **Release workflow** (`release.yml`) triggered by version tags (`v*`):
  - Creates a GitHub Release with changelog extraction from `CHANGELOG.md`
  - Builds release binaries for multiple platforms (Linux x86_64, Linux aarch64, Windows x86_64, Windows aarch64, macOS x86_64, macOS aarch64)
  - Uploads binaries as release assets
  - Publishes the crate to crates.io (stable releases only, skipping pre-releases)
  - Marks tags containing `-` (e.g., `v0.2.0-rc.1`) as pre-releases
- Add a `CHANGELOG.md` file to the project root to support release note extraction
- Prepare `Cargo.toml` with required metadata fields (`description`, `license`, `repository`, `homepage`, `keywords`, `categories`) to ensure crates.io publishability

## Capabilities

### New Capabilities
- `ci-test-pipeline`: Automated build, test, lint, and security audit pipeline triggered on push and pull requests
- `ci-release-pipeline`: Automated release workflow that creates GitHub releases with cross-platform binaries and publishes to crates.io on version tags

### Modified Capabilities
<!-- No existing spec-level requirements are changing. -->

## Impact

- **New files**: `.github/workflows/build-test-audit.yml`, `.github/workflows/release.yml`, `CHANGELOG.md`
- **Dependencies**: No new Rust dependencies. Relies on GitHub Actions ecosystem actions (`actions/checkout`, `dtolnay/rust-toolchain`, `actions/cache`, `actions-rust-lang/audit`, `softprops/action-gh-release`)
- **Secrets required**: `CARGO_REGISTRY_TOKEN` must be configured in GitHub repository secrets for crates.io publishing
- **Branch strategy**: CI triggers aligned with the project's Gitflow model (`master` and `dev` branches)
