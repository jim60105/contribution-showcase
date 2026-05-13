# Design: Enhance CI Coverage and Quality

## Context

The current CI pipeline (`.github/workflows/build-test-audit.yml`) provides format checking (rustfmt), linting (clippy), a test matrix across ubuntu/windows/macos, and a security audit job — but has **no code coverage measurement**. The reference project [subx-cli](https://github.com/jim60105/subx-cli) already integrates `cargo-llvm-cov` with Codecov for coverage reporting, providing a proven pattern to adopt.

The release pipeline (`.github/workflows/release.yml`) builds binaries for six targets (three architectures × two platforms) but performs **no pre-upload smoke test** to validate that built binaries are actually runnable before publishing them as release assets.

## Goals / Non-Goals

### Goals

- Add code coverage measurement with Codecov integration via `cargo-llvm-cov` (LCOV output)
- Validate release binaries with a smoke test (`--version` flag) before uploading assets
- Rename `build-test-audit.yml` → `build-test-audit-coverage.yml` to reflect the expanded scope

### Non-Goals

- **Coverage threshold enforcement** — The project is early stage; enforcing minimums can be added later once a baseline is established
- **Windows/macOS coverage** — Linux-only coverage is sufficient to start; all logic is platform-independent
- **Custom quality check scripts** — The reference project uses a `scripts/` directory for helpers; we will use direct commands in workflow steps to keep things simple
- **cargo-nextest** — Not needed; our 152 tests run in ~2 seconds with plain `cargo test`, and Codecov coverage upload does not require JUnit XML

## Decisions

### Decision 1: `cargo-llvm-cov` for Coverage

Use `cargo-llvm-cov` to generate LCOV coverage reports uploaded to Codecov.

- **Why**: Same tool as the reference project. Generates LCOV format natively for Codecov. Works directly with `cargo test` via `cargo llvm-cov --lcov`.
- **Alternative considered**: `cargo-tarpaulin` — less accurate instrumentation, does not support LLVM-level branch coverage, and would diverge from the reference project's toolchain.

### Decision 2: Linux-Only Coverage Job

Run the coverage job only on `ubuntu-latest`, not across the full OS matrix.

- **Why**: The reference project runs coverage on ubuntu + windows. For this simpler project, ubuntu-only is sufficient to start. All application logic is platform-independent with no OS-specific code paths.
- **Alternative considered**: Multi-OS coverage — adds CI time and complexity without significant benefit for pure-Rust CLI code that has no platform-conditional compilation.

### Decision 3: Smoke Test Using `--version` Flag

Add a smoke-test step in the release workflow that runs each built binary with `--version` to validate it is executable.

- **Why**: Simple, fast validation that the binary loads and runs correctly on the target platform. The reference project uses the same pattern. For `aarch64-unknown-linux-gnu` targets, `qemu-user-static` provides cross-architecture execution.
- **Alternative considered**: Full integration tests in the release pipeline — too complex and slow for a release gate; comprehensive testing belongs in the CI workflow, not the release workflow.

### Decision 4: Workflow File Rename

Rename `build-test-audit.yml` → `build-test-audit-coverage.yml`.

- **Why**: Reflects the added coverage responsibility. Matches the reference project's naming convention where workflow names describe all major jobs.
- **Alternative considered**: Keep the current name — would not reflect the workflow's actual content, making it misleading to contributors.

### Decision 5: Enable CLI `--version` Flag

Add `version` to the clap `#[command]` derive macro in `src/main.rs`.

- **Why**: The release smoke test runs `./binary --version`. Without the `version` attribute, clap will not recognize `--version` and the smoke test will fail every release.
- **Alternative considered**: Use `--help` for smoke test — works but `--version` is more conventional and produces cleaner output.

### Decision 6: Separate Coverage Cache Keys

Use distinct cache keys for the coverage job (e.g., `coverage-target` prefix) to avoid cross-contamination with normal test builds.

- **Why**: Coverage-instrumented builds produce different artifacts. Sharing the `target` cache between test and coverage jobs may cause inefficient rebuilds or stale instrumented binaries.

## Risks / Trade-offs

- **[Risk] `CODECOV_TOKEN` secret not configured** — Coverage upload will silently fail if the repository secret is missing. Mitigated by setting `fail_ci_if_error: false` on the Codecov upload step so CI remains green while the secret is pending configuration.
- **[Risk] `cargo-llvm-cov` installation adds ~30s to the coverage job** — Acceptable overhead given the value of coverage data. The tool is installed via `taiki-e/install-action` which caches the binary.
- **[Risk] QEMU smoke test on `aarch64` may be slow** — The reference project uses a 30-second timeout for QEMU-based smoke tests, which is sufficient for a `--version` invocation.
- **[Trade-off] Linux-only coverage means no Windows/macOS coverage gaps detected** — Acceptable for platform-independent code with no `#[cfg(target_os)]` conditionals. Can be revisited if platform-specific code is introduced.
