## MODIFIED Requirements

### Requirement: CLI Argument Parsing

The system SHALL use clap 4 subcommands. The binary SHALL accept two subcommands: `generate` and `init`. The `generate` subcommand SHALL accept `--config`, `--output`, `--author`, `--since`, `--until`, `--timeline-max-buckets`, `--title`, `--types`, and `--exclude-hashes` flags. The `init` subcommand SHALL accept an optional `--output` flag (default: `showcase.toml`). Invoking the binary with no subcommand SHALL print help text and exit with a zero status.

#### Scenario: Generate with all flags provided

- **GIVEN** the binary is invoked with `generate --config custom.toml --output out.html --author "Alice" --since 2024-01-01 --until 2024-12-31 --timeline-max-buckets 7 --title "My Report" --types feat,fix --exclude-hashes abc123,def456`
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

#### Scenario: Title overridden under generate

- **GIVEN** the TOML config file sets `title = "Config Title"`
- **WHEN** the binary is invoked with `generate --title "CLI Title"`
- **THEN** the effective title value is `"CLI Title"`

#### Scenario: Types overridden under generate

- **GIVEN** the TOML config file sets `types = ["feat", "fix", "docs"]` under `[filters]`
- **WHEN** the binary is invoked with `generate --types feat,fix`
- **THEN** the effective types filter is `["feat", "fix"]`

#### Scenario: Exclude hashes overridden under generate

- **GIVEN** the TOML config file sets `exclude_hashes = ["aaa111"]` under `[filters]`
- **WHEN** the binary is invoked with `generate --exclude-hashes bbb222,ccc333`
- **THEN** the effective exclude hashes list is `["bbb222", "ccc333"]`
