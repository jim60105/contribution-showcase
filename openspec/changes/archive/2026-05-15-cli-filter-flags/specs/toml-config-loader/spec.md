## MODIFIED Requirements

### Requirement: Filter Accessors

The system SHALL expose accessors for the `author`, `since`, `until`, `types`, and `exclude_hashes` filter fields after config loading and validation, returning `None` for unset optional filters. When CLI overrides have been applied to the config object before accessor calls, the overridden values SHALL be returned.

#### Scenario: All filters set

- **GIVEN** the merged config includes `author`, `since`, `until`, `types`, and `exclude_hashes`
- **WHEN** filter accessors are called
- **THEN** each returns the corresponding value

#### Scenario: No filters set

- **GIVEN** the merged config has no filter fields
- **WHEN** filter accessors are called
- **THEN** each returns `None`

#### Scenario: CLI types override config types

- **GIVEN** the TOML config sets `types = ["feat", "fix"]` and the CLI provides `--types docs,test`
- **WHEN** filter accessors are called after merging
- **THEN** `types` returns `Some(["docs", "test"])`

#### Scenario: CLI exclude-hashes override config exclude-hashes

- **GIVEN** the TOML config sets `exclude_hashes = ["aaa"]` and the CLI provides `--exclude-hashes bbb,ccc`
- **WHEN** filter accessors are called after merging
- **THEN** `exclude_hashes` returns `Some(["bbb", "ccc"])`
