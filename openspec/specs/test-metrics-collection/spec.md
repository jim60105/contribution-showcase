# Test Metrics Collection

## Purpose

Collects test framework detection, test file/case counts, and coverage
report data for each configured project. Results feed the Test Metrics
slide in the HTML report.

## Requirements

### Requirement: Framework Detection

The system SHALL detect the test framework for each project by checking for the presence of configuration files in this order: `pyproject.toml` containing "pytest" → `Cargo.toml` → `package.json` containing "vitest" or "jest". If no test framework is detected, the project's framework SHALL be "none".

#### Scenario: Python project with pytest
- **GIVEN** a project directory containing `pyproject.toml` with "pytest" as a dependency
- **WHEN** framework detection runs
- **THEN** the framework is identified as "pytest"

#### Scenario: Rust project
- **GIVEN** a project directory containing `Cargo.toml`
- **WHEN** framework detection runs
- **THEN** the framework is identified as "cargo test"

#### Scenario: Node.js project with vitest
- **GIVEN** a project directory containing `package.json` with "vitest" as a dependency
- **WHEN** framework detection runs
- **THEN** the framework is identified as "vitest"

#### Scenario: No test framework detected
- **GIVEN** a project directory without any recognized test configuration
- **WHEN** framework detection runs
- **THEN** the framework is "none" and test counts are 0

### Requirement: Test File Discovery

The system SHALL count files matching test patterns: `test_*`, `*_test.*`, `*.test.*`, `*.spec.*`. The search SHALL exclude `node_modules/`, `target/`, `.venv/`, `__pycache__/`, and other build artifact directories.

#### Scenario: Python project with test files
- **GIVEN** a project with files `tests/test_auth.py`, `tests/test_api.py`, `src/utils.py`
- **WHEN** test file discovery runs
- **THEN** `test_file_count` is 2

### Requirement: Test Case Counting

The system SHALL count test cases by pattern-matching source lines within discovered test files:
- Python: lines matching `def test_` or `async def test_`
- JavaScript/TypeScript: lines matching `it(` or `test(`
- Rust: lines matching `#[test]`, `#[tokio::test`, or `#[rstest]`

Counting is approximate (regex-based, not AST-based).

#### Scenario: Python test file with 5 test functions
- **GIVEN** a test file containing 5 lines matching `def test_*`
- **WHEN** test case counting runs
- **THEN** the file contributes 5 to `test_case_count`

### Requirement: Coverage Report Discovery

The system SHALL attempt to read existing coverage reports without executing tests:
- Python: parse `coverage.xml` (Cobertura XML, `line-rate` attribute)
- Node.js: parse `coverage/coverage-summary.json` (Istanbul, `total.lines.pct` field)

`.coverage` (SQLite) and `target/llvm-cov/` are intentionally excluded due to
format complexity and instability. Rust projects and projects without a
supported report format get `coverage_percent` of `None`.

If no coverage report file is found, `coverage_percent` SHALL be `None`.

#### Scenario: Python project with coverage.xml
- **GIVEN** a project directory containing a `coverage.xml` file with `line-rate="0.85"`
- **WHEN** coverage discovery runs
- **THEN** `coverage_percent` is `85.0`

#### Scenario: Node.js project with Istanbul summary
- **GIVEN** a project directory containing `coverage/coverage-summary.json` with `total.lines.pct: 72.5`
- **WHEN** coverage discovery runs
- **THEN** `coverage_percent` is `72.5`

#### Scenario: No coverage report
- **GIVEN** a project directory with no coverage report files
- **WHEN** coverage discovery runs
- **THEN** `coverage_percent` is `None`

### Requirement: Per-Project Coverage Command Execution

The system SHALL support optional `coverage_command` and `coverage_result_path` fields in the project configuration. When `coverage_command` is set, the system SHALL execute it via `sh -c` in the project directory before reading coverage results. All configured coverage commands SHALL be spawned in parallel, and the system waits for all to complete before collecting results.

When `coverage_result_path` is set, the system SHALL use that explicit path instead of auto-discovery. The path is relative to the project root.

#### Scenario: Coverage command configured
- **GIVEN** a project with `coverage_command = "uv run pytest --cov=src --cov-report=xml:coverage.xml -q"`
- **WHEN** test metrics collection runs
- **THEN** the command is executed in the project directory before reading coverage data

#### Scenario: Parallel coverage execution
- **GIVEN** 5 projects each with a `coverage_command` configured
- **WHEN** test metrics collection runs
- **THEN** all 5 coverage commands are spawned concurrently

#### Scenario: Explicit coverage result path
- **GIVEN** a project with `coverage_result_path = "coverage.xml"`
- **WHEN** coverage discovery runs
- **THEN** the system reads from `{project_path}/coverage.xml` instead of auto-discovering
