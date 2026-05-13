## 1. Prerequisites

- [x] 1.1 Add `version` to the clap `#[command]` derive in `src/main.rs` to enable `--version` flag (required by release smoke test)

## 2. Rename CI Workflow and Add Coverage Job

- [x] 2.1 Rename `.github/workflows/build-test-audit.yml` to `.github/workflows/build-test-audit-coverage.yml` via `git mv` and update the workflow `name` field to "Build, Test, Audit & Coverage"
- [x] 2.2 Add a new `coverage` job to the CI workflow running on `ubuntu-latest` with `llvm-tools-preview` component, `cargo-llvm-cov`, and Cargo caching (use `coverage` prefix in cache keys)
- [x] 2.3 Add coverage generation step running `cargo llvm-cov --lcov --output-path lcov.info`
- [x] 2.4 Add Codecov coverage upload step using `codecov/codecov-action@v6` with `files: lcov.info`, `fail_ci_if_error: false`, and `CODECOV_TOKEN` secret

## 3. Add Smoke Test to Release Workflow

- [x] 3.1 Add a smoke test step to the release workflow build job, placed after Package and before Upload Release Asset, conditioned on `matrix.os == 'ubuntu-latest'`
- [x] 3.2 Implement x86_64 native smoke test (`timeout 30 ./${{ matrix.asset_name }} --version`) and aarch64 QEMU smoke test (install `qemu-user-static`, `binfmt-support`, `gcc-aarch64-linux-gnu`; run with `QEMU_LD_PREFIX`)
