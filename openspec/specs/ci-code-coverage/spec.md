# CI Code Coverage Specification

**Capability Type:** CI/CD Pipeline

## Purpose

Defines the code coverage job within the CI workflow. This job uses `cargo-llvm-cov` with `cargo test` to generate LCOV coverage reports, then uploads them to Codecov for tracking and reporting.

## Requirements

### Requirement: Coverage Tool Installation

The coverage job SHALL install `cargo-llvm-cov` using `taiki-e/install-action@v2`. The Rust toolchain SHALL include the `llvm-tools-preview` component.

#### Scenario: Coverage tool is installed during the job

- **WHEN** the coverage job runs
- **THEN** `cargo-llvm-cov` SHALL be installed via `taiki-e/install-action@v2`

#### Scenario: Rust toolchain includes llvm-tools-preview

- **WHEN** the Rust toolchain is installed for the coverage job
- **THEN** the `llvm-tools-preview` component SHALL be included in the toolchain configuration

### Requirement: LCOV Report Generation

The coverage job SHALL generate an LCOV coverage report by running `cargo llvm-cov --lcov --output-path lcov.info`. The report SHALL be written to `lcov.info` in the workspace root.

#### Scenario: LCOV report is generated

- **WHEN** the coverage generation step executes
- **THEN** it SHALL run the command `cargo llvm-cov --lcov --output-path lcov.info`
- **AND** the file `lcov.info` SHALL be produced in the workspace root

### Requirement: Codecov Coverage Upload

The coverage job SHALL upload the LCOV report to Codecov using `codecov/codecov-action@v6` with `files: lcov.info` and `fail_ci_if_error: false`. The upload SHALL use `CODECOV_TOKEN` from repository secrets.

#### Scenario: Coverage report is uploaded to Codecov

- **WHEN** the LCOV report has been generated
- **THEN** it SHALL be uploaded using `codecov/codecov-action@v6`
- **AND** the action SHALL be configured with `files: lcov.info`
- **AND** the action SHALL be configured with `fail_ci_if_error: false`

#### Scenario: Codecov token is provided from secrets

- **WHEN** the Codecov upload action runs
- **THEN** it SHALL use `CODECOV_TOKEN` from repository secrets for authentication

### Requirement: Coverage Job Runner

The coverage job SHALL run on `ubuntu-latest` only. It does NOT use a multi-OS matrix.

#### Scenario: Coverage job runs on a single OS

- **WHEN** the coverage job is configured
- **THEN** `runs-on` SHALL be set to `ubuntu-latest`
- **AND** no matrix strategy SHALL be defined for the operating system

### Requirement: Coverage Job Caching

The coverage job SHALL cache the Cargo registry, index, and build target directories using `actions/cache@v5` with keys incorporating the `Cargo.lock` hash. Cache keys SHALL use a `coverage` prefix to distinguish from normal test build caches.

#### Scenario: Cargo artifacts are cached with coverage-specific keys

- **WHEN** the coverage job runs
- **THEN** it SHALL use `actions/cache@v5` to cache the Cargo registry, index, and build target directories
- **AND** the cache key SHALL incorporate a hash of `Cargo.lock` with a `coverage` prefix

#### Scenario: Cache is restored on subsequent runs

- **WHEN** a cache entry exists matching the current `Cargo.lock` hash
- **THEN** the cached Cargo registry, index, and target directories SHALL be restored before building
