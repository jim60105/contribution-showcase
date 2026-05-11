# Daily Lines Metric

## Purpose

Computes the average daily code line changes metric, representing coding
velocity as the mean number of line changes (insertions + deletions) per active
development day. The denominator uses the count of unique dates on which
commits were authored, not total calendar days in the range.

## Requirements

### Requirement: Unique Commit Date Counting

The system SHALL count the number of distinct calendar dates (YYYY-MM-DD) on
which at least one commit was authored, after applying all configured filters
(author, date range).

#### Scenario: Three dates with multiple commits
- **GIVEN** commits on 2025-01-01 (2 commits), 2025-01-03 (1 commit), and
  2025-01-05 (3 commits)
- **WHEN** unique dates are counted
- **THEN** the count is 3

#### Scenario: All commits on one date
- **GIVEN** five commits all authored on 2025-03-15
- **WHEN** unique dates are counted
- **THEN** the count is 1

### Requirement: Average Calculation Formula

The system SHALL compute the average daily lines changed as:

```
avg_daily_lines = (total_insertions + total_deletions) / unique_date_count
```

where `total_insertions` and `total_deletions` are the sums across all commits
after filtering.

#### Scenario: Typical calculation
- **GIVEN** 3 unique dates, total insertions = 300, total deletions = 150
- **WHEN** the average is computed
- **THEN** `avg_daily_lines` is `(300 + 150) / 3 = 150.0`

### Requirement: Zero Commit Handling

When no commits remain after filtering, the system SHALL set `avg_daily_lines`
to `0.0` rather than producing a division-by-zero error.

#### Scenario: Empty commit list
- **GIVEN** zero commits after filtering
- **WHEN** the average is computed
- **THEN** `avg_daily_lines` is `0.0`

### Requirement: Data Model Field

The `Summary` struct SHALL include an `avg_daily_lines` field of type `f64`,
serialised to JSON for use by the HTML template.

#### Scenario: JSON output contains the field
- **GIVEN** a `ShowcaseData` instance with computed metrics
- **WHEN** the data is serialised to JSON
- **THEN** the JSON contains `"avg_daily_lines"` with a floating-point value

### Requirement: Display Formatting

The HTML template SHALL display `avg_daily_lines` as a rounded integer
(`Math.round()`) followed by the unit label "行/天". The underlying JSON
value remains a floating-point `f64`.

#### Scenario: Non-integer average
- **GIVEN** `avg_daily_lines` is `123.7`
- **WHEN** the KPI card renders
- **THEN** the displayed text is `"124"` (or locale-formatted equivalent)
  with the unit label "行/天"
