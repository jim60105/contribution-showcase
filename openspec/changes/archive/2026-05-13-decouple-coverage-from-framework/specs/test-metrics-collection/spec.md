## MODIFIED Requirements

### Requirement: Framework Detection

The system SHALL detect the test framework for each project by checking for the presence of configuration files in this order: `pyproject.toml` containing "pytest" → `Cargo.toml` → `package.json` containing "vitest" or "jest" → `deno.json` or `deno.jsonc` containing "vitest" → `deno.json` or `deno.jsonc` (without vitest, detected as "deno test"). If no test framework is detected, the project's framework SHALL be "none".

Framework detection determines the test file discovery patterns and test case counting strategy only. It does NOT gate coverage command execution or coverage result parsing. A project with `framework == "none"` SHALL still have coverage command execution and coverage result parsing performed; only test file/case counting is skipped.

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

#### Scenario: Deno project with vitest
- **GIVEN** a project directory containing `deno.json` with "vitest" as a dependency
- **WHEN** framework detection runs
- **THEN** the framework is identified as "vitest"

#### Scenario: Deno project without vitest
- **GIVEN** a project directory containing `deno.json` without "vitest"
- **WHEN** framework detection runs
- **THEN** the framework is identified as "deno test"

#### Scenario: Unknown framework does not block coverage
- **GIVEN** a project directory without any recognized test configuration but with `coverage_command` configured
- **WHEN** test metrics collection runs
- **THEN** the framework is "none", test counts are 0, but the coverage command is still executed and coverage results are still parsed

### Requirement: Coverage Report Discovery

The system SHALL attempt to read existing coverage reports without executing tests:
- Python: parse `coverage.xml` (Cobertura XML, `line-rate` attribute)
- Node.js: parse `coverage/coverage-summary.json` (Istanbul, `total.lines.pct` field)

`.coverage` (SQLite) and `target/llvm-cov/` are intentionally excluded due to
format complexity and instability.

Projects with any framework (including "none") SHALL have coverage discovery attempted. `coverage_percent` is `None` only when no supported coverage file is found.

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

#### Scenario: Coverage discovered for unknown framework project
- **GIVEN** a project with `framework == "none"` and `coverage_result_path = "coverage/coverage-summary.json"` pointing to a valid Istanbul JSON with `total.lines.pct: 91.2`
- **WHEN** coverage discovery runs
- **THEN** `coverage_percent` is `91.2`

### Requirement: Per-Project Coverage Command Execution

The system SHALL support optional `coverage_command` and `coverage_result_path` fields in the project configuration. When `coverage_command` is set, the system SHALL execute it via `sh -c` in the project directory before reading coverage results, regardless of the detected framework. Framework detection SHALL NOT gate execution of `coverage_command`. All configured coverage commands SHALL be spawned in parallel, and the system waits for all to complete before collecting results.

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

#### Scenario: Coverage command runs for unknown framework
- **GIVEN** a project with no recognized test framework configuration but with `coverage_command = "deno task test:coverage"` configured
- **WHEN** test metrics collection runs
- **THEN** the coverage command is executed in the project directory and coverage results are parsed normally

#### Scenario: Coverage command fails for unknown framework project
- **GIVEN** a project with `framework == "none"` and a `coverage_command` that exits non-zero and produces no output file
- **WHEN** test metrics collection runs
- **THEN** `coverage_percent` is `None`, `test_file_count` is 0, and `test_case_count` is 0
