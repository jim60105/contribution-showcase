# rust-cli-scaffold — Delta

## Added Requirements

### Requirement: Average Daily Lines in Data Model

The `Summary` struct SHALL include an `avg_daily_lines` field of type `f64`
that is serialised to JSON.

#### Scenario: ShowcaseData serialisation
- **GIVEN** a `ShowcaseData` instance with `summary.avg_daily_lines` set to `150.0`
- **WHEN** the data is serialised to JSON
- **THEN** the output contains `"avg_daily_lines": 150.0` within the summary object

## Removed Requirements

_None._
