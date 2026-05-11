# Rust CLI Scaffold

## Purpose

Provide a well-structured Rust CLI entry point for the contribution-showcase tool, using `clap` 4 for argument parsing and `anyhow` for error handling. The CLI accepts flags that control configuration, output destination, and commit filtering, with CLI flags always taking precedence over values loaded from the TOML config file.

## Requirements

### Requirement: CLI Argument Parsing

The system SHALL parse command-line arguments using `clap` 4, accepting `--config`, `--output`, `--author`, `--since`, and `--until` flags.

#### Scenario: All flags provided
- **GIVEN** the binary is invoked with `--config custom.toml --output out.html --author "Alice" --since 2024-01-01 --until 2024-12-31`
- **WHEN** argument parsing completes
- **THEN** each flag value is available to the application with the exact values supplied

#### Scenario: No flags provided
- **GIVEN** the binary is invoked with no arguments
- **WHEN** argument parsing completes
- **THEN** all optional flags are `None` and the application proceeds with defaults

#### Scenario: Unknown flag provided
- **GIVEN** the binary is invoked with `--unknown-flag`
- **WHEN** `clap` attempts to parse arguments
- **THEN** the process exits with a non-zero status and an error message is printed to stderr

### Requirement: Default Config File Path

The system SHALL use `showcase.toml` as the default config file path when `--config` is not specified.

#### Scenario: Config flag omitted
- **GIVEN** the binary is invoked without `--config`
- **WHEN** config loading begins
- **THEN** the application attempts to load `showcase.toml` from the current working directory

#### Scenario: Config flag provided
- **GIVEN** the binary is invoked with `--config path/to/my-config.toml`
- **WHEN** config loading begins
- **THEN** the application loads the config from `path/to/my-config.toml`

### Requirement: CLI Flags Override Config Values

The system SHALL give CLI flag values precedence over values loaded from the TOML config file.

#### Scenario: Output path overridden by CLI
- **GIVEN** the config file sets `output = "config-output.html"`
- **WHEN** the binary is invoked with `--output cli-output.html`
- **THEN** the effective output path is `cli-output.html`

#### Scenario: Author filter overridden by CLI
- **GIVEN** the config file sets `author = "ConfigAuthor"`
- **WHEN** the binary is invoked with `--author "CliAuthor"`
- **THEN** the effective author filter is `"CliAuthor"`

#### Scenario: Config value used when CLI flag absent
- **GIVEN** the config file sets `since = "2024-06-01"`
- **WHEN** the binary is invoked without `--since`
- **THEN** the effective since date is `2024-06-01`

### Requirement: Error Output to Stderr

The system SHALL write all error and diagnostic messages to stderr, never to stdout.

#### Scenario: Config file not found
- **GIVEN** the specified config file does not exist
- **WHEN** the application attempts to load it
- **THEN** an error message is printed to stderr and the process exits with a non-zero status

#### Scenario: Invalid CLI argument value
- **GIVEN** the binary is invoked with `--since not-a-date`
- **WHEN** argument parsing or validation runs
- **THEN** an error message is printed to stderr and the process exits with a non-zero status

### Requirement: HTML Output to File

The system SHALL write generated HTML output to the file specified by the effective output path, not to stdout.

#### Scenario: Successful generation
- **GIVEN** all configuration and inputs are valid
- **WHEN** the application completes HTML generation
- **THEN** the HTML content is written to the effective output file path

### Requirement: Average Daily Lines in Data Model

The `Summary` struct SHALL include an `avg_daily_lines` field of type `f64`
that is serialised to JSON.

#### Scenario: ShowcaseData serialisation
- **GIVEN** a `ShowcaseData` instance with `summary.avg_daily_lines` set to `150.0`
- **WHEN** the data is serialised to JSON
- **THEN** the output contains `"avg_daily_lines": 150.0` within the summary object

### Requirement: Anyhow Error Handling

The system SHALL use `anyhow` for error propagation, providing contextual error messages throughout the call chain.

#### Scenario: Nested error context
- **GIVEN** a TOML parse error occurs inside config loading
- **WHEN** the error propagates to `main`
- **THEN** the displayed error includes context from each layer (e.g., "Failed to load config: invalid TOML: …")
