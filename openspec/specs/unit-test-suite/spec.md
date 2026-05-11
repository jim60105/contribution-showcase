# Unit Test Suite

## Purpose

Provides a comprehensive unit test suite using Rust's built-in test framework
to verify correctness of core logic — commit parsing, configuration handling,
data aggregation, and safety-critical escaping — without requiring external
test dependencies or a live Git repository.

## Requirements

### Requirement: Built-In Test Framework Only

The system SHALL use Rust's built-in `#[cfg(test)]` modules and `#[test]`
attribute with standard assertion macros (`assert!`, `assert_eq!`,
`assert_ne!`). No external test framework or crate SHALL be added.

#### Scenario: Test compilation
- **GIVEN** the project's `Cargo.toml`
- **WHEN** reviewing dev-dependencies
- **THEN** no external test framework crates are listed

### Requirement: Test Execution

All tests SHALL be runnable via `cargo test` with no additional flags or
setup required.

#### Scenario: Running the test suite
- **GIVEN** a clean checkout of the repository
- **WHEN** `cargo test` is executed
- **THEN** all unit tests pass

### Requirement: Test Module Placement

Tests SHALL be co-located as `#[cfg(test)]` modules inside `collector.rs`,
`config.rs`, and `main.rs`.

#### Scenario: Test discoverability
- **GIVEN** the source files `collector.rs`, `config.rs`, and `main.rs`
- **WHEN** searching for `#[cfg(test)]` blocks
- **THEN** each of the three files contains a test module

### Requirement: Coverage Areas

The test suite SHALL cover the following categories of logic:

- Conventional commit message parsing (type, scope, description extraction)
- Commit type label mapping
- Type breakdown deduplication
- Timeline aggregation (weekly bucketing)
- Date-based and type-based filtering
- Git shortstat output parsing (insertions, deletions)
- Configuration file loading and defaults
- Repository path resolution
- Branch name validation
- Date string validation
- HTML-safe JSON escaping (`<`, `>`, `&`, U+2028, U+2029)

#### Scenario: Conventional commit parsing
- **GIVEN** a commit message `feat(auth): add login endpoint`
- **WHEN** the parser processes the message
- **THEN** the type is `feat`, scope is `auth`, and description is
  `add login endpoint`

#### Scenario: JSON escaping of dangerous characters
- **GIVEN** a string containing `<script>`
- **WHEN** the HTML-safe JSON escaping function processes it
- **THEN** the output contains `\u003cscript\u003e`

#### Scenario: Shortstat line parsing
- **GIVEN** a shortstat line ` 3 files changed, 42 insertions(+), 7 deletions(-)`
- **WHEN** the parser extracts statistics
- **THEN** insertions is `42` and deletions is `7`

### Requirement: Mock Data Only

Tests SHALL NOT invoke real Git commands or depend on a live repository.
All test inputs SHALL use inline mock data or fixtures.

#### Scenario: No git subprocess in tests
- **GIVEN** the unit test suite
- **WHEN** `cargo test` is executed
- **THEN** no `git` child process is spawned
