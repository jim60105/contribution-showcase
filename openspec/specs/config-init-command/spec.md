# config-init-command

## Purpose

Provide an `init` subcommand for the contribution-showcase CLI that bootstraps a
well-documented template configuration file (`showcase.toml`) in the current
working directory. The template content is embedded at compile time from
`showcase.example.toml`.

## Requirements

### Requirement: Init Subcommand Default Output

The `init` subcommand SHALL write a template config file to `showcase.toml` in
the current working directory when no `--output` flag is provided. When the
`--output` flag is provided, the subcommand SHALL write the template to the
specified path instead.

#### Scenario: Default output path

- **GIVEN** the user invokes `contribution-showcase init` without a `--output`
  flag
- **WHEN** the command executes
- **THEN** a file named `showcase.toml` SHALL be created in the current working
  directory with the template content

#### Scenario: Custom output path

- **GIVEN** the user invokes `contribution-showcase init --output custom.toml`
- **WHEN** the command executes
- **THEN** a file named `custom.toml` SHALL be created at the specified path
  with the template content

#### Scenario: Output path with non-existent parent directory

- **GIVEN** the user invokes `contribution-showcase init --output nested/dir/showcase.toml`
  and the `nested/dir/` directory does not exist
- **WHEN** the command executes
- **THEN** the parent directories SHALL be created automatically and the
  template file SHALL be written successfully

### Requirement: Template Content Source

The template content SHALL be embedded at compile time from
`showcase.example.toml` using `include_str!`. The written file content SHALL be
byte-for-byte identical to the source example file.

#### Scenario: Content matches example file

- **GIVEN** the `showcase.example.toml` file exists in the project source tree
- **WHEN** the `init` subcommand writes the template to disk
- **THEN** the content of the written file SHALL be identical to the content of
  `showcase.example.toml` as embedded at compile time

### Requirement: Overwrite Guard

The `init` subcommand SHALL refuse to overwrite an existing file at the target
path. The implementation SHALL use atomic file creation (e.g.,
`OpenOptions::create_new(true)`) rather than a check-then-write pattern to avoid
race conditions. When the target file already exists, the command SHALL exit with
a non-zero status and print an error message to stderr indicating the file
already exists.

#### Scenario: Target file already exists

- **GIVEN** a file already exists at the target output path
- **WHEN** the user invokes `contribution-showcase init`
- **THEN** the command SHALL exit with a non-zero status and print an error
  message to stderr indicating that the file already exists
- **AND** the existing file SHALL NOT be modified

#### Scenario: Target file does not exist

- **GIVEN** no file exists at the target output path
- **WHEN** the user invokes `contribution-showcase init`
- **THEN** the template file SHALL be created successfully and the command SHALL
  exit with a zero status

### Requirement: Template Content Requirements

The `showcase.example.toml` file SHALL contain the following structure:

- A top-level `title` field with a placeholder value.
- An `[output]` section with a `path` field.
- A `[filters]` section with commented-out examples for `author`, `since`,
  `until`, `types`, and `exclude_hashes`.
- At least two `[[projects]]` entries with placeholder names (e.g.
  "my-backend", "my-frontend") and `path`, `description`, `branch` fields.
  Each project entry SHALL also include commented-out examples for the optional
  `coverage_command` and `coverage_result_path` fields.

All values SHALL use placeholder or fake data, not real project names.

#### Scenario: Template contains required sections

- **GIVEN** the `showcase.example.toml` file is generated
- **WHEN** its content is inspected
- **THEN** it SHALL contain a `title` field, an `[output]` section with `path`,
  a `[filters]` section with commented-out `author`, `since`, `until`, `types`,
  and `exclude_hashes` fields, and at least two `[[projects]]` entries each with
  `path`, `description`, `branch` fields, and commented-out `coverage_command`
  and `coverage_result_path` fields

#### Scenario: Placeholder values used

- **GIVEN** the `showcase.example.toml` file is generated
- **WHEN** its content is inspected
- **THEN** all project names, paths, and descriptive values SHALL be obviously
  placeholder data (e.g. "my-backend", "my-frontend") and SHALL NOT reference
  any real project names

### Requirement: Template Documentation

The `showcase.example.toml` file SHALL include inline TOML comments explaining
each configuration field, its purpose, accepted values, and whether it is
optional or required.

#### Scenario: Comments explain field purpose

- **GIVEN** the `showcase.example.toml` file is generated
- **WHEN** its content is inspected
- **THEN** each configuration field SHALL be preceded or accompanied by a TOML
  comment that describes its purpose, accepted values, and whether it is
  optional or required

### Requirement: Init Output Messaging

On success, the command SHALL print a confirmation message to stderr indicating
where the file was written and suggesting the user edit it before running
`generate`.

#### Scenario: Success message printed

- **GIVEN** the `init` subcommand completes successfully
- **WHEN** the template file has been written
- **THEN** a confirmation message SHALL be printed to stderr indicating the
  path of the written file and suggesting the user edit it before running
  `generate`

### Requirement: Template Config Validity

The `showcase.example.toml` file SHALL be valid TOML that successfully
deserializes into the application's `Config` struct. This ensures that `init`
never produces a file that fails immediately under `generate`.

#### Scenario: Template parses as valid Config

- **GIVEN** the `showcase.example.toml` content
- **WHEN** it is parsed as TOML and deserialized into the `Config` struct
- **THEN** deserialization SHALL succeed without errors
