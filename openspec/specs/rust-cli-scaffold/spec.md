# Rust CLI Scaffold

## Purpose

Provide a well-structured Rust CLI entry point for the contribution-showcase tool, using `clap` 4 with subcommands (`generate` and `init`) and `anyhow` for error handling. The `generate` subcommand accepts flags that control configuration, output destination, and commit filtering, with CLI flags always taking precedence over values loaded from the TOML config file. The `init` subcommand bootstraps a template configuration file.

## Requirements

### Requirement: CLI Argument Parsing

The system SHALL use clap 4 subcommands. The binary SHALL accept two subcommands: `generate` and `init`. The `generate` subcommand SHALL accept `--config`, `--output`, `--author`, `--since`, and `--until` flags. The `init` subcommand SHALL accept an optional `--output` flag (default: `showcase.toml`). Invoking the binary with no subcommand SHALL print help text and exit with a zero status.

#### Scenario: Generate with all flags provided

- **GIVEN** the binary is invoked with `generate --config custom.toml --output out.html --author "Alice" --since 2024-01-01 --until 2024-12-31`
- **WHEN** argument parsing completes
- **THEN** each flag value is available to the generate subcommand with the exact values supplied

#### Scenario: Init subcommand recognized

- **GIVEN** the binary is invoked with `init`
- **WHEN** argument parsing completes
- **THEN** the init subcommand is selected and the `--output` flag defaults to `showcase.toml`

#### Scenario: No subcommand prints help

- **GIVEN** the binary is invoked with no arguments
- **WHEN** argument parsing completes
- **THEN** help text is printed to stdout and the process exits with status zero

#### Scenario: Unknown subcommand error

- **GIVEN** the binary is invoked with an unrecognized subcommand (e.g. `foobar`)
- **WHEN** argument parsing runs
- **THEN** an error message is written to stderr and the process exits with a non-zero status

### Requirement: Default Config File Path

The system SHALL use `showcase.toml` as the default config file path when `--config` is not specified under the `generate` subcommand.

#### Scenario: Config flag omitted under generate

- **GIVEN** the binary is invoked with `generate` and no `--config` flag
- **WHEN** the config path is resolved
- **THEN** the effective config path is `showcase.toml`

#### Scenario: Config flag provided under generate

- **GIVEN** the binary is invoked with `generate --config custom.toml`
- **WHEN** the config path is resolved
- **THEN** the effective config path is `custom.toml`

### Requirement: CLI Flags Override Config Values

Under the `generate` subcommand, CLI flag values SHALL take precedence over values loaded from the TOML config file.

#### Scenario: Output path overridden under generate

- **GIVEN** the TOML config file sets `path = "config-output.html"` under the `[output]` section
- **WHEN** the binary is invoked with `generate --output cli-output.html`
- **THEN** the effective output path is `cli-output.html`

#### Scenario: Author overridden under generate

- **GIVEN** the TOML config file sets `author = "Config Author"` under the `[filters]` section
- **WHEN** the binary is invoked with `generate --author "CLI Author"`
- **THEN** the effective author value is `"CLI Author"`

#### Scenario: Config value used when CLI flag absent under generate

- **GIVEN** the TOML config file sets `author = "Config Author"` under the `[filters]` section
- **WHEN** the binary is invoked with `generate` and no `--author` flag
- **THEN** the effective author value is `"Config Author"`

### Requirement: Error Output to Stderr

Both `generate` and `init` subcommands SHALL write all error and diagnostic messages to stderr.

#### Scenario: Generate config file not found

- **GIVEN** the binary is invoked with `generate --config nonexistent.toml`
- **WHEN** the config file cannot be found
- **THEN** an error message is written to stderr and the process exits with a non-zero status

#### Scenario: Init target file already exists

- **GIVEN** a file named `showcase.toml` already exists in the current directory
- **WHEN** the binary is invoked with `init`
- **THEN** an error or warning message is written to stderr indicating the file already exists

### Requirement: HTML Output to File

The `generate` subcommand SHALL write generated HTML output to the file specified by the effective output path.

#### Scenario: Successful generation under generate

- **GIVEN** a valid config file exists and commit data is available
- **WHEN** the binary is invoked with `generate`
- **THEN** an HTML file is written to the effective output path and nothing is written to stdout

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
