## Why

The current CI pipeline (build-test-audit.yml) lacks code coverage measurement and Codecov integration. The reference project (subx-cli) includes cargo-llvm-cov for LCOV generation and Codecov coverage uploads, and release artifact smoke testing — none of which exist in our pipeline. This gap means there's no visibility into test coverage trends and no pre-upload validation of release binaries.

## What Changes

- Add a **code coverage** job to the CI workflow using `cargo-llvm-cov` to generate LCOV reports and upload them to Codecov
- Add a **smoke test** step to the release workflow that validates built Linux binaries (x86_64 natively, aarch64 via qemu) before uploading release assets
- Add `version` to clap CLI derive to enable `--version` flag (required by release smoke test)
- Rename workflow file from `build-test-audit.yml` to `build-test-audit-coverage.yml` to reflect the added coverage job

## Capabilities

### New Capabilities
- `ci-code-coverage`: Code coverage generation with cargo-llvm-cov and Codecov upload
- `ci-release-smoke-test`: Pre-upload smoke testing of Linux release binaries

### Modified Capabilities
- `ci-test-pipeline`: Rename workflow file
- `ci-release-pipeline`: Add smoke test step for Linux binaries before release asset upload

## Impact

- `.github/workflows/build-test-audit.yml` → renamed to `build-test-audit-coverage.yml`, coverage job added
- `.github/workflows/release.yml` → smoke test step added for Linux matrix entries
- `src/main.rs` → add `version` to clap `#[command]` derive to enable `--version` flag
- New secrets required: `CODECOV_TOKEN` (repository secret for Codecov uploads)
- New CI dependencies: `cargo-llvm-cov`, `qemu-user-static` (for aarch64 smoke test)
