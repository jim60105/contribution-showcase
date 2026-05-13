# Delta Spec: CI Release Smoke Test

> **Capability**: ci-release-smoke-test (NEW)
> **Change**: enhance-ci-coverage-and-quality
> **Status**: Draft

## Overview

Adds smoke tests to the release workflow that validate compiled Linux binaries before uploading them as release assets. This ensures that only functional binaries are published, catching linking errors, missing dependencies, or corrupted builds early.

## ADDED Requirements

### REQ-SMOKE-X86: Smoke Test for Linux x86_64 Binary

The release workflow SHALL execute a smoke test for the `x86_64-unknown-linux-gnu` binary by running `./<asset_name> --version` with a 30-second timeout. The test SHALL run natively on the `ubuntu-latest` runner. A non-zero exit code or timeout SHALL fail the workflow before the upload step.

#### Scenario: x86_64 binary passes smoke test

- **WHEN** the release workflow builds the `x86_64-unknown-linux-gnu` target on the `ubuntu-latest` runner
- **THEN** the workflow SHALL execute `timeout 30 ./<asset_name> --version` natively
- **AND** the step SHALL succeed with exit code 0

#### Scenario: x86_64 binary fails smoke test

- **WHEN** the `x86_64-unknown-linux-gnu` binary exits with a non-zero code or exceeds the 30-second timeout
- **THEN** the workflow SHALL fail
- **AND** the binary SHALL NOT be uploaded as a release asset

---

### REQ-SMOKE-AARCH64: Smoke Test for Linux aarch64 Binary

The release workflow SHALL execute a smoke test for the `aarch64-unknown-linux-gnu` binary using `qemu-user-static` with `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu`. The test SHALL install `qemu-user-static`, `binfmt-support`, and `gcc-aarch64-linux-gnu` packages. The command SHALL be `timeout 30 qemu-aarch64-static ./<asset_name> --version`.

#### Scenario: aarch64 binary passes smoke test under QEMU

- **WHEN** the release workflow builds the `aarch64-unknown-linux-gnu` target on the `ubuntu-latest` runner
- **THEN** the workflow SHALL install `qemu-user-static`, `binfmt-support`, and `gcc-aarch64-linux-gnu` via `apt-get`
- **AND** the workflow SHALL execute `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu timeout 30 qemu-aarch64-static ./<asset_name> --version`
- **AND** the step SHALL succeed with exit code 0

#### Scenario: aarch64 binary fails smoke test under QEMU

- **WHEN** the `aarch64-unknown-linux-gnu` binary exits with a non-zero code or exceeds the 30-second timeout under QEMU emulation
- **THEN** the workflow SHALL fail
- **AND** the binary SHALL NOT be uploaded as a release asset

---

### REQ-SMOKE-PLACEMENT: Smoke Test Placement

The smoke test step SHALL execute AFTER the Package step and BEFORE the Upload Release Asset step, ensuring only validated binaries are uploaded.

#### Scenario: Smoke test runs between packaging and upload

- **WHEN** the release workflow reaches the smoke test step
- **THEN** the Package step SHALL have already completed successfully
- **AND** the Upload Release Asset step SHALL NOT execute until the smoke test passes

#### Scenario: Failed smoke test prevents upload

- **WHEN** the smoke test step fails
- **THEN** the Upload Release Asset step SHALL be skipped
- **AND** the workflow SHALL report a failure status

---

### REQ-SMOKE-SCOPE: Smoke Test Scope

Smoke tests SHALL only run for Linux targets (`matrix.os == 'ubuntu-latest'`). Windows and macOS binaries are NOT smoke-tested.

#### Scenario: Smoke test runs on Linux runner

- **WHEN** the matrix job has `matrix.os == 'ubuntu-latest'`
- **THEN** the smoke test step SHALL execute

#### Scenario: Smoke test is skipped on non-Linux runners

- **WHEN** the matrix job has `matrix.os` set to a Windows or macOS runner (e.g., `windows-latest`, `macos-latest`)
- **THEN** the smoke test step SHALL be skipped via its `if` condition
- **AND** the workflow SHALL proceed directly to the Upload Release Asset step

## Reference Implementation

```yaml
- name: Smoke test
  if: matrix.os == 'ubuntu-latest'
  shell: bash
  run: |
    set -euo pipefail
    if [[ "${{ matrix.target }}" == aarch64-* ]]; then
      sudo apt-get update
      sudo apt-get install -y qemu-user-static binfmt-support gcc-aarch64-linux-gnu
      QEMU_LD_PREFIX=/usr/aarch64-linux-gnu \
        timeout 30 qemu-aarch64-static ./${{ matrix.asset_name }} --version
    else
      timeout 30 ./${{ matrix.asset_name }} --version
    fi
```
