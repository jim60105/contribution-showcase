# Capability: toml-config-loader (delta)

## MODIFIED Requirements

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
