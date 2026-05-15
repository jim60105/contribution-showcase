# TOML Config Loader

## Purpose

Load, parse, and validate a TOML configuration file that defines project entries and filtering options for the contribution-showcase tool. The loader resolves relative paths against the config file's parent directory, validates date and branch inputs to prevent malformed or malicious values, and provides safe accessors for all filter options.

## Requirements

### Requirement: Config File Loading and TOML Parsing

The system SHALL read a TOML config file and deserialize it into a typed configuration structure containing `[[projects]]` entries (each with `name`, `path`, `description`, optional `branch`, and optional `url`) and top-level filter/output options.

#### Scenario: Valid config with multiple projects
- **GIVEN** a TOML file containing two `[[projects]]` entries with `name`, `path`, and `description`
- **WHEN** the config is loaded
- **THEN** the parsed structure contains two project entries with the correct field values

#### Scenario: Project with optional branch
- **GIVEN** a `[[projects]]` entry includes `branch = "develop"`
- **WHEN** the config is loaded
- **THEN** the project's branch field is `Some("develop")`

#### Scenario: Project without branch
- **GIVEN** a `[[projects]]` entry omits the `branch` field
- **WHEN** the config is loaded
- **THEN** the project's branch field is `None`

#### Scenario: Project with optional URL
- **GIVEN** a `[[projects]]` entry includes `url = "https://github.com/example/repo"`
- **WHEN** the config is loaded
- **THEN** the project's URL field is `Some("https://github.com/example/repo")`

#### Scenario: Project without URL
- **GIVEN** a `[[projects]]` entry omits the `url` field
- **WHEN** the config is loaded
- **THEN** the project's URL field is `None`

#### Scenario: Malformed TOML
- **GIVEN** the config file contains invalid TOML syntax
- **WHEN** the loader attempts to parse it
- **THEN** an error is returned describing the parse failure

#### Scenario: Missing required project field
- **GIVEN** a `[[projects]]` entry omits the required `path` field
- **WHEN** the loader attempts to parse it
- **THEN** an error is returned indicating the missing field

### Requirement: Default Output Path

The system SHALL default the output path to `dist/index.html` when no `output` key is present in the config file and no `--output` CLI flag is provided.

#### Scenario: Output omitted from config and CLI
- **GIVEN** the config file does not contain an `output` key and `--output` is not passed
- **WHEN** the effective output path is resolved
- **THEN** the output path is `dist/index.html`

#### Scenario: Output specified in config
- **GIVEN** the config file sets `output = "report.html"`
- **WHEN** the effective output path is resolved and no CLI override exists
- **THEN** the output path is `report.html`

### Requirement: Config-Relative Path Resolution

The system SHALL resolve relative paths in the config file against the config file's parent directory, not the current working directory. Absolute paths SHALL be preserved as-is.

#### Scenario: Relative project path
- **GIVEN** the config file is at `/opt/configs/showcase.toml` and a project entry has `path = "../repos/my-project"`
- **WHEN** path resolution runs
- **THEN** the resolved path is `/opt/configs/../repos/my-project` (canonicalized to `/opt/repos/my-project`)

#### Scenario: Absolute project path
- **GIVEN** a project entry has `path = "/srv/repos/my-project"`
- **WHEN** path resolution runs
- **THEN** the resolved path remains `/srv/repos/my-project`

#### Scenario: Rooted project path on Windows semantics
- **GIVEN** a project entry has a rooted path like `/opt/repos/foo` on Windows semantics
- **WHEN** path resolution runs
- **THEN** the loader treats it as already anchored
- **AND** the resolved path remains `/opt/repos/foo`

#### Scenario: Relative output path
- **GIVEN** the config file is at `/opt/configs/showcase.toml` and `output = "build/out.html"`
- **WHEN** path resolution runs
- **THEN** the resolved output path is relative to `/opt/configs/`

### Requirement: Date Validation

The system SHALL parse `since` and `until` values as `chrono::NaiveDate` using strict `YYYY-MM-DD` format and SHALL enforce that `since <= until` when both are present. Validation runs after CLI overrides are merged.

#### Scenario: Valid date range
- **GIVEN** `since = "2024-01-01"` and `until = "2024-12-31"`
- **WHEN** validation runs
- **THEN** both dates are accepted

#### Scenario: Invalid date format
- **GIVEN** `since = "01/01/2024"`
- **WHEN** validation runs
- **THEN** an error is returned indicating the date format is invalid

#### Scenario: Since after until
- **GIVEN** `since = "2024-12-31"` and `until = "2024-01-01"`
- **WHEN** validation runs
- **THEN** an error is returned indicating that `since` must not be after `until`

#### Scenario: Only since provided
- **GIVEN** `since = "2024-06-01"` and `until` is not set
- **WHEN** validation runs
- **THEN** the since date is accepted without error

#### Scenario: Only until provided
- **GIVEN** `until = "2024-06-01"` and `since` is not set
- **WHEN** validation runs
- **THEN** the until date is accepted without error

### Requirement: Branch Name Validation

The system SHALL reject branch names that start with `-`, or contain `..`, `^`, `~`, or whitespace characters, to prevent argument injection and revspec traversal. Validation runs after CLI overrides are merged.

#### Scenario: Valid branch name
- **GIVEN** a project entry has `branch = "feature/new-ui"`
- **WHEN** validation runs
- **THEN** the branch name is accepted

#### Scenario: Branch starting with dash
- **GIVEN** a project entry has `branch = "-malicious"`
- **WHEN** validation runs
- **THEN** an error is returned indicating the branch name is invalid

#### Scenario: Branch containing double dot
- **GIVEN** a project entry has `branch = "main..develop"`
- **WHEN** validation runs
- **THEN** an error is returned indicating the branch name is invalid

#### Scenario: Branch containing caret
- **GIVEN** a project entry has `branch = "main^2"`
- **WHEN** validation runs
- **THEN** an error is returned indicating the branch name is invalid

#### Scenario: Branch containing tilde
- **GIVEN** a project entry has `branch = "main~1"`
- **WHEN** validation runs
- **THEN** an error is returned indicating the branch name is invalid

#### Scenario: Branch containing spaces
- **GIVEN** a project entry has `branch = "my branch"`
- **WHEN** validation runs
- **THEN** an error is returned indicating the branch name is invalid

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
