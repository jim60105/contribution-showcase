## MODIFIED Requirements

### Requirement: Granularity Selection Based on Date Span (REPLACES existing)
The system SHALL compute the date span between the earliest and latest commit dates and select the timeline granularity using a five-level cascade. At each level, count the number of distinct buckets that would be produced; if ≤ 14, use that granularity:
1. Count inclusive calendar days: `(max_date - min_date).num_days() + 1`. If ≤ 14 → daily buckets.
2. Else count distinct ISO week labels (`%G-W%V` format) from min_date through max_date. If ≤ 14 → weekly buckets.
3. Else count distinct months: `(max_year - min_year) * 12 + (max_month - min_month) + 1`. If ≤ 14 → monthly buckets.
4. Else count distinct quarters: `(max_year - min_year) * 4 + (max_quarter - min_quarter) + 1`. If ≤ 14 → quarterly buckets.
5. Else → yearly buckets (terminal fallback; may produce more than 14 buckets for multi-decade spans).

#### Scenario: Short date range selects daily granularity
- **WHEN** commits span from `2025-03-10` to `2025-03-18` (9 days, ≤ 14)
- **THEN** the timeline SHALL use daily buckets

#### Scenario: Exactly 14 days selects daily granularity
- **WHEN** commits span exactly 14 days (e.g., `2025-03-01` to `2025-03-14`)
- **THEN** the timeline SHALL use daily buckets (14 ≤ 14)

#### Scenario: 15 days escalates to weekly if weeks ≤ 14
- **WHEN** commits span 15 days within 3 distinct ISO weeks
- **THEN** the timeline SHALL use weekly buckets (> 14 days, but ≤ 14 weeks)

#### Scenario: Many weeks escalates to monthly
- **WHEN** commits span 16 distinct ISO weeks (e.g., 4 months)
- **THEN** the timeline SHALL use monthly buckets (> 14 weeks, but ≤ 14 months)

#### Scenario: Many months escalates to quarterly
- **WHEN** commits span 16 distinct months (e.g., `2024-01` to `2025-04`)
- **THEN** the timeline SHALL use quarterly buckets (> 14 months, but ≤ 14 quarters)

#### Scenario: Many quarters escalates to yearly
- **WHEN** commits span 16 distinct quarters (e.g., `2021-Q1` to `2024-Q4`)
- **THEN** the timeline SHALL use yearly buckets (> 14 quarters)

### Requirement: Daily Bucket Labels (UNCHANGED)
When daily granularity is selected, each bucket label SHALL be formatted as `YYYY-MM-DD` (e.g., `2025-03-15`).

### Requirement: Weekly Bucket Labels (UNCHANGED)
When weekly granularity is selected, each bucket label SHALL be formatted as `YYYY-Www` using ISO week numbering (e.g., `2025-W12`).

### Requirement: Monthly Bucket Labels (UNCHANGED)
When monthly granularity is selected, each bucket label SHALL be formatted as `YYYY-MM` (e.g., `2025-03`).

## ADDED Requirements

### Requirement: Quarterly Bucket Labels
When quarterly granularity is selected, each bucket label SHALL be formatted as `YYYY-Qn` where n ∈ {1, 2, 3, 4} (e.g., `2025-Q1` for January–March, `2025-Q2` for April–June).

#### Scenario: Quarterly bucket labels use YYYY-Qn format
- **WHEN** quarterly granularity is selected and commits span from `2024-Q2` to `2025-Q1`
- **THEN** the bucket labels SHALL be `2024-Q2`, `2024-Q3`, `2024-Q4`, `2025-Q1`

### Requirement: Yearly Bucket Labels
When yearly granularity is selected, each bucket label SHALL be formatted as `YYYY` (e.g., `2025`).

#### Scenario: Yearly bucket labels use YYYY format
- **WHEN** yearly granularity is selected and commits span from 2020 to 2025
- **THEN** the bucket labels SHALL be `2020`, `2021`, `2022`, `2023`, `2024`, `2025`

### Requirement: Contiguous Bucket Generation for Quarterly
The timeline SHALL generate contiguous quarterly buckets from the quarter containing the earliest commit to the quarter containing the latest commit, filling gaps with zero-line entries.

#### Scenario: Quarterly contiguous buckets fill gaps
- **WHEN** quarterly granularity is selected and commits exist only in `2024-Q1` and `2024-Q4`
- **THEN** the timeline SHALL produce 4 buckets: `2024-Q1`, `2024-Q2`, `2024-Q3`, `2024-Q4`, with 0 lines for quarters without commits

### Requirement: Contiguous Bucket Generation for Yearly
The timeline SHALL generate contiguous yearly buckets from the year of the earliest commit to the year of the latest commit, filling gaps with zero-line entries.

#### Scenario: Yearly contiguous buckets fill gaps
- **WHEN** yearly granularity is selected and commits exist only in `2020` and `2025`
- **THEN** the timeline SHALL produce 6 buckets: `2020`, `2021`, `2022`, `2023`, `2024`, `2025`, with 0 lines for years without commits

### Requirement: Single-Day Edge Case (UNCHANGED)
When all commits are on the same date, the span is 1 day (≤ 14), so daily granularity SHALL be used, producing exactly one bucket.

### Requirement: Empty Commits (UNCHANGED)
When there are zero commits, the timeline SHALL be empty (no buckets).

### Requirement: Height Normalization (UNCHANGED)
The `height` field SHALL be normalized to 0–100 based on the maximum lines value across all buckets, regardless of the selected granularity.

### Requirement: Contiguous Bucket Generation for Daily/Weekly/Monthly (UNCHANGED)
The timeline SHALL generate contiguous buckets for daily, weekly, and monthly granularities as currently specified.
