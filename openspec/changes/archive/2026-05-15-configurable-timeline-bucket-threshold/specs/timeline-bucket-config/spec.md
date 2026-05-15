# Timeline Bucket Config

## Purpose

Exposes the timeline granularity threshold as a configurable parameter via TOML
config (`timeline_max_buckets` in `[output]`) and CLI argument
(`--timeline-max-buckets`), enabling users to control when the timeline
escalates from finer to coarser granularity.

## ADDED Requirements

### Requirement: TOML Config Field for Timeline Max Buckets
The `[output]` section of the TOML configuration file SHALL accept an optional
`timeline_max_buckets` field of type positive integer. When present, this value
SHALL be used as the bucket threshold for timeline granularity selection. When
omitted, the default value of `14` SHALL be used.

#### Scenario: Config file specifies timeline_max_buckets
- **WHEN** the TOML config contains `[output]` with `timeline_max_buckets = 21`
- **THEN** the timeline granularity cascade SHALL use `21` as the bucket threshold

#### Scenario: Config file omits timeline_max_buckets
- **WHEN** the TOML config does not contain a `timeline_max_buckets` field
- **THEN** the timeline granularity cascade SHALL use the default value of `14` as the bucket threshold

### Requirement: CLI Argument for Timeline Max Buckets
The `generate` subcommand SHALL accept an optional `--timeline-max-buckets <N>`
argument. When provided, this value SHALL override any `timeline_max_buckets`
value set in the TOML config file.

#### Scenario: CLI argument provided without config value
- **WHEN** the CLI is invoked with `--timeline-max-buckets 7` and the config file does not set `timeline_max_buckets`
- **THEN** the timeline granularity cascade SHALL use `7` as the bucket threshold

#### Scenario: CLI argument overrides config value
- **WHEN** the TOML config sets `timeline_max_buckets = 21` and the CLI is invoked with `--timeline-max-buckets 10`
- **THEN** the timeline granularity cascade SHALL use `10` as the bucket threshold (CLI wins)

#### Scenario: Config value used when CLI argument is absent
- **WHEN** the TOML config sets `timeline_max_buckets = 21` and the CLI is invoked without `--timeline-max-buckets`
- **THEN** the timeline granularity cascade SHALL use `21` as the bucket threshold

### Requirement: Validation of Timeline Max Buckets Value
The `timeline_max_buckets` value, whether from config or CLI, SHALL be a
positive integer ≥ 1. If the value is less than 1 or not a valid integer, the
tool SHALL report a validation error and exit without generating output.

#### Scenario: Value of zero is rejected
- **WHEN** the CLI is invoked with `--timeline-max-buckets 0`
- **THEN** the tool SHALL report a validation error and exit with a non-zero status code

#### Scenario: Negative value is rejected
- **WHEN** the TOML config sets `timeline_max_buckets = -5`
- **THEN** the tool SHALL report a validation error and exit with a non-zero status code

#### Scenario: Value of 1 is accepted
- **WHEN** the CLI is invoked with `--timeline-max-buckets 1`
- **THEN** the tool SHALL accept the value and use `1` as the bucket threshold (effectively forcing yearly granularity for most spans)

### Requirement: Default Behavior Preserves Backward Compatibility
The system SHALL behave identically to the prior hardcoded threshold of `14`
when `timeline_max_buckets` is not specified in either the TOML config or the
CLI arguments. Existing config files without `timeline_max_buckets` SHALL
continue to work without modification.

#### Scenario: Default behavior matches legacy behavior
- **WHEN** neither the TOML config nor the CLI specifies `timeline_max_buckets`
- **THEN** the timeline granularity cascade SHALL use `14` as the bucket threshold, producing identical output to the previous hardcoded implementation

### Requirement: Parameter Passed to Timeline Granularity Cascade
The resolved `timeline_max_buckets` value (from CLI, config, or default) SHALL
be passed through to the timeline granularity cascade logic as the bucket
threshold parameter, replacing the previously hardcoded constant. Note: the
yearly fallback is a terminal level and may still produce more buckets than the
threshold for very long date ranges — `timeline_max_buckets` controls when the
cascade *escalates*, not an absolute cap on bucket count.

#### Scenario: Custom threshold flows through to cascade
- **WHEN** `timeline_max_buckets` is resolved to `7`
- **THEN** the `build_timeline()` function SHALL use `7` as the comparison threshold at each cascade level instead of the hardcoded `14`
