# CI Release Pipeline — Delta Spec

> **Change:** enhance-ci-coverage-and-quality
> **Modifies:** `ci-release-pipeline`

---

## ADDED Requirements

### ADDED: Release Smoke Test for Linux Binaries

The release workflow SHALL include a smoke test step for Linux target binaries that executes AFTER the Package step and BEFORE the Upload Release Asset step. The smoke test validates that built binaries are runnable. Details are specified in the `ci-release-smoke-test` capability spec.

#### Scenario: Smoke test runs for x86_64 Linux binary

- **WHEN** the build matrix entry for `x86_64-unknown-linux-gnu` completes the Package step
- **THEN** the workflow SHALL execute a smoke test step that validates the built binary is runnable BEFORE proceeding to the Upload Release Asset step

#### Scenario: Smoke test runs for aarch64 Linux binary

- **WHEN** the build matrix entry for `aarch64-unknown-linux-gnu` completes the Package step
- **THEN** the workflow SHALL execute a smoke test step that validates the built binary is runnable BEFORE proceeding to the Upload Release Asset step

#### Scenario: Smoke test does not run for non-Linux targets

- **WHEN** the build matrix entry for a Windows or macOS target completes the Package step
- **THEN** the workflow SHALL NOT execute the smoke test step and SHALL proceed directly to the Upload Release Asset step

#### Scenario: Smoke test failure prevents asset upload

- **WHEN** the smoke test step fails for a Linux target binary
- **THEN** the workflow SHALL NOT proceed to the Upload Release Asset step for that matrix entry and the job SHALL fail

#### Scenario: Smoke test details are defined externally

- **WHEN** the smoke test step is implemented
- **THEN** the specific validation logic SHALL follow the `ci-release-smoke-test` capability spec
