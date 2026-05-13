# CI Test Pipeline Specification

### Requirement: CI Workflow Trigger Conditions

The CI workflow SHALL trigger on push events to the `master` and `dev` branches and on pull request events targeting those branches. The workflow MUST skip execution when only Markdown files are changed.

#### Scenario: Push to master triggers workflow

- **WHEN** a commit is pushed to the `master` branch containing non-Markdown file changes
- **THEN** the CI workflow SHALL be triggered and all pipeline jobs SHALL execute

#### Scenario: Push to dev triggers workflow

- **WHEN** a commit is pushed to the `dev` branch containing non-Markdown file changes
- **THEN** the CI workflow SHALL be triggered and all pipeline jobs SHALL execute

#### Scenario: Pull request targeting master triggers workflow

- **WHEN** a pull request is opened, synchronized, or reopened targeting the `master` branch with non-Markdown file changes
- **THEN** the CI workflow SHALL be triggered and all pipeline jobs SHALL execute

#### Scenario: Pull request targeting dev triggers workflow

- **WHEN** a pull request is opened, synchronized, or reopened targeting the `dev` branch with non-Markdown file changes
- **THEN** the CI workflow SHALL be triggered and all pipeline jobs SHALL execute

#### Scenario: Markdown-only changes are skipped

- **WHEN** a push or pull request event contains changes exclusively to files matching `**/*.md`
- **THEN** the CI workflow SHALL NOT execute any jobs via the `paths-ignore` configuration

#### Scenario: Mixed changes including Markdown are not skipped

- **WHEN** a push or pull request event contains changes to both Markdown files and non-Markdown files (e.g., `.rs`, `.toml`)
- **THEN** the CI workflow SHALL be triggered and all pipeline jobs SHALL execute

### Requirement: Formatting Check

The CI pipeline SHALL run a Rust formatting check using `cargo fmt -- --check` to enforce consistent code style. The job MUST fail if any source file is not properly formatted.

#### Scenario: All files are properly formatted

- **WHEN** the formatting check step executes `cargo fmt -- --check`
- **AND** all Rust source files conform to `rustfmt` formatting rules
- **THEN** the step SHALL exit with code 0 and the job SHALL pass

#### Scenario: Formatting violations are detected

- **WHEN** the formatting check step executes `cargo fmt -- --check`
- **AND** one or more Rust source files do not conform to `rustfmt` formatting rules
- **THEN** the step SHALL exit with a non-zero code and the job SHALL fail

### Requirement: Clippy Linting

The CI pipeline SHALL run Clippy linting using `cargo clippy --all-targets -- -D warnings` to enforce lint-free code. The job MUST treat all Clippy warnings as errors.

#### Scenario: No Clippy warnings present

- **WHEN** the linting step executes `cargo clippy --all-targets -- -D warnings`
- **AND** the codebase produces no Clippy warnings
- **THEN** the step SHALL exit with code 0 and the job SHALL pass

#### Scenario: Clippy warnings are present

- **WHEN** the linting step executes `cargo clippy --all-targets -- -D warnings`
- **AND** the codebase produces one or more Clippy warnings
- **THEN** the warnings SHALL be promoted to errors via the `-D warnings` flag and the job SHALL fail

### Requirement: Cross-Platform Test Execution

The CI pipeline SHALL execute `cargo test` across a matrix of operating systems including `ubuntu-latest`, `windows-latest`, and `macos-latest`, all using the `stable` Rust toolchain. Each matrix combination MUST run independently.

#### Scenario: Tests pass on Ubuntu

- **WHEN** the test job executes `cargo test` on the `ubuntu-latest` runner with the `stable` Rust toolchain
- **THEN** all tests SHALL pass and the job SHALL report success for the Ubuntu matrix entry

#### Scenario: Tests pass on Windows

- **WHEN** the test job executes `cargo test` on the `windows-latest` runner with the `stable` Rust toolchain
- **THEN** all tests SHALL pass and the job SHALL report success for the Windows matrix entry

#### Scenario: Tests pass on macOS

- **WHEN** the test job executes `cargo test` on the `macos-latest` runner with the `stable` Rust toolchain
- **THEN** all tests SHALL pass and the job SHALL report success for the macOS matrix entry

#### Scenario: Test failure on one OS does not skip others

- **WHEN** the test job fails on one operating system in the matrix
- **THEN** the remaining matrix jobs SHALL continue to execute independently and report their own results

### Requirement: Cargo Caching

The CI pipeline SHALL cache Cargo artifacts to accelerate subsequent workflow runs. The cache MUST include the Cargo registry, registry index, and the build target directory.

#### Scenario: Cache is populated on first run

- **WHEN** the CI workflow runs for the first time without an existing cache
- **THEN** the pipeline SHALL build all dependencies from source and save the Cargo registry (`~/.cargo/registry`), registry index (`~/.cargo/git`), and build target (`target/`) directories to the cache

#### Scenario: Cache is restored on subsequent runs

- **WHEN** the CI workflow runs and a matching cache exists from a previous run
- **THEN** the pipeline SHALL restore the cached Cargo registry, registry index, and build target directories before building, reducing build times

#### Scenario: Cache key includes dependency lockfile

- **WHEN** the cache key is computed
- **THEN** it SHALL incorporate the hash of `Cargo.lock` so that dependency changes invalidate the cache

### Requirement: Security Audit

The CI pipeline SHALL perform a security audit of Rust dependencies using `cargo audit` or an equivalent auditing action. The job MUST fail if any known vulnerability is found in the dependency tree.

#### Scenario: No known vulnerabilities in dependencies

- **WHEN** the security audit step executes against the project's dependency tree
- **AND** no advisories match any dependency version in `Cargo.lock`
- **THEN** the audit step SHALL exit with code 0 and the job SHALL pass

#### Scenario: Known vulnerability detected in a dependency

- **WHEN** the security audit step executes against the project's dependency tree
- **AND** one or more dependencies have known vulnerabilities listed in the RustSec Advisory Database
- **THEN** the audit step SHALL exit with a non-zero code and the job SHALL fail

### Requirement: Workflow File Location

The CI workflow definition MUST be stored at `.github/workflows/build-test-audit-coverage.yml` in the repository root, following GitHub Actions conventions.

#### Scenario: Workflow file exists at correct path

- **WHEN** the repository is inspected for CI configuration
- **THEN** the file `.github/workflows/build-test-audit-coverage.yml` SHALL exist and contain a valid GitHub Actions workflow definition

#### Scenario: Workflow file is valid YAML

- **WHEN** GitHub Actions parses `.github/workflows/build-test-audit-coverage.yml`
- **THEN** the file SHALL be accepted as a syntactically valid workflow definition without parse errors
