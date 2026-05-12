# TOML Config Loader — Delta

## ADDED Requirements

### Requirement: Exclude Hashes Config Field

Under `[filters]`, the system SHALL accept an optional `exclude_hashes` key whose value is a list of full 40-character hexadecimal commit hashes. When `exclude_hashes` is absent or empty, no commits are excluded by hash.

#### Scenario: Config with one excluded hash

- **GIVEN** a TOML config containing `exclude_hashes = ["dd33ee63950bb49a284de835528343561f1a70d5"]` under `[filters]`
- **WHEN** the config is loaded
- **THEN** `filters.exclude_hashes` contains the single hash `"dd33ee63950bb49a284de835528343561f1a70d5"`

#### Scenario: Config without exclude_hashes

- **GIVEN** a TOML config that does not contain an `exclude_hashes` key under `[filters]`
- **WHEN** the config is loaded
- **THEN** `filters.exclude_hashes` is `None`
