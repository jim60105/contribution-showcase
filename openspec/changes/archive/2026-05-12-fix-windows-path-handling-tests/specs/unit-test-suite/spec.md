# Unit Test Suite — Delta

## MODIFIED Requirements

### Requirement: Cross-Platform Path Fixtures

Unit tests that embed filesystem paths in TOML fixtures SHALL remain valid on Linux, macOS, and Windows.

#### Scenario: Windows path in TOML fixture

- **GIVEN** a test writes a TOML path field from a runtime filesystem path on Windows
- **WHEN** the fixture is parsed
- **THEN** parsing SHALL succeed without invalid backslash escape errors

### Requirement: Cross-Platform Path Assertions

Path assertions SHALL be independent of path separator style.

#### Scenario: Source file discovery assertion on Windows

- **GIVEN** discovered path `src\main.rs` on Windows
- **WHEN** the assertion evaluates the expected location
- **THEN** it SHALL pass using path-separator-agnostic matching
