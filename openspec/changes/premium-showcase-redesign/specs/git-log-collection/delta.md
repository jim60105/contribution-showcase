# git-log-collection — Delta

## Added Requirements

### Requirement: Unique Commit Date Collection

After collecting and filtering commits, the system SHALL count the number of
distinct calendar dates (YYYY-MM-DD) on which at least one commit was authored.

#### Scenario: Commits span multiple dates
- **GIVEN** filtered commits on dates 2025-01-01, 2025-01-01, 2025-01-03
- **WHEN** unique dates are counted
- **THEN** the count is 2

### Requirement: Average Daily Lines Computation

The system SHALL compute the average daily lines changed as
`(total_insertions + total_deletions) / unique_date_count`, guarding against
division by zero (returning `0.0` when no commits exist).

#### Scenario: Normal computation
- **GIVEN** 2 unique dates, total insertions = 100, total deletions = 50
- **WHEN** the average is computed
- **THEN** `avg_daily_lines` equals `75.0`

#### Scenario: No commits after filtering
- **GIVEN** 0 commits after applying filters
- **WHEN** the average is computed
- **THEN** `avg_daily_lines` equals `0.0`

## Removed Requirements

_None._
