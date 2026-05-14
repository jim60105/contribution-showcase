# Adaptive Timeline Granularity

## Purpose

Automatically selects daily, weekly, monthly, quarterly, or yearly timeline
granularity based on the date span of commits, ensuring the timeline chart is
readable regardless of whether the contribution period covers days, months, or
years.

## Requirements

### Requirement: Granularity Selection Based on Date Span
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

### Requirement: Daily Bucket Labels
When daily granularity is selected, each bucket label SHALL be formatted as `YYYY-MM-DD` (e.g., `2025-03-15`).

#### Scenario: Daily bucket labels use YYYY-MM-DD format
- **WHEN** daily granularity is selected and commits span from `2025-03-10` to `2025-03-12`
- **THEN** the bucket labels SHALL be `2025-03-10`, `2025-03-11`, and `2025-03-12`

### Requirement: Weekly Bucket Labels
When weekly granularity is selected, each bucket label SHALL be formatted as `YYYY-Www` using ISO week numbering (e.g., `2025-W12`). This matches the current behavior.

#### Scenario: Weekly bucket labels use YYYY-Www format
- **WHEN** weekly granularity is selected and commits fall in ISO weeks 10 through 12 of 2025
- **THEN** the bucket labels SHALL be `2025-W10`, `2025-W11`, and `2025-W12`

### Requirement: Monthly Bucket Labels
When monthly granularity is selected, each bucket label SHALL be formatted as `YYYY-MM` (e.g., `2025-03`).

#### Scenario: Monthly bucket labels use YYYY-MM format
- **WHEN** monthly granularity is selected and commits span from January to March 2025
- **THEN** the bucket labels SHALL be `2025-01`, `2025-02`, and `2025-03`

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

### Requirement: Single-Day Edge Case
When all commits are on the same date, the span is 0 days (< 14), so daily granularity SHALL be used, producing exactly one bucket.

#### Scenario: All commits on the same day produce one daily bucket
- **WHEN** all commits have the date `2025-03-15`
- **THEN** the timeline SHALL use daily granularity and produce exactly one bucket with label `2025-03-15`

### Requirement: Empty Commits
When there are zero commits, the timeline SHALL be empty (no buckets). This matches current behavior.

#### Scenario: Zero commits produce an empty timeline
- **WHEN** the commit list is empty
- **THEN** the timeline SHALL contain zero buckets

### Requirement: Contiguous Bucket Generation for Daily/Weekly/Monthly
The timeline SHALL generate contiguous buckets covering the entire date span from the earliest to the latest commit date, filling gaps (periods with no commits) with zero-line buckets. This ensures the timeline accurately represents the passage of time without visual discontinuities.

#### Scenario: Daily contiguous buckets fill gaps
- **WHEN** daily granularity is selected and commits exist only on `2025-03-01` and `2025-03-05`
- **THEN** the timeline SHALL produce 5 buckets: `2025-03-01`, `2025-03-02`, `2025-03-03`, `2025-03-04`, `2025-03-05`, with 0 lines for dates without commits

#### Scenario: Monthly contiguous buckets fill gaps
- **WHEN** monthly granularity is selected and commits exist only in `2025-01` and `2025-04`
- **THEN** the timeline SHALL produce 4 buckets: `2025-01`, `2025-02`, `2025-03`, `2025-04`, with 0 lines for months without commits

#### Scenario: Weekly contiguous buckets fill gaps
- **WHEN** weekly granularity is selected and commits exist only in `2025-W10` and `2025-W13`
- **THEN** the timeline SHALL produce 4 buckets: `2025-W10`, `2025-W11`, `2025-W12`, `2025-W13`, with 0 lines for weeks without commits

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

### Requirement: Height Normalization
The `height` field SHALL be normalized to 0–100 based on the maximum lines value across all buckets, regardless of the selected granularity. This matches current behavior.

#### Scenario: Height normalization applies consistently across granularities
- **WHEN** monthly granularity is selected and the bucket with the most lines changed has 500 lines
- **THEN** that bucket's `height` SHALL be `100` and a bucket with 250 lines SHALL have `height` of `50`

#### Scenario: Single bucket gets height 100
- **WHEN** there is exactly one bucket (e.g., single-day edge case)
- **THEN** that bucket's `height` SHALL be `100`

### Requirement: Per-Type Lines Breakdown in Timeline Entries

Each `TimelineEntry` SHALL include a `type_lines` field (a map from commit type string to `usize`) recording lines changed per conventional commit type within that bucket. The `lines` field remains the aggregate total. Types with zero lines in a bucket SHALL be omitted from the map. The `type_lines` values SHALL sum to `lines`.

#### Scenario: Bucket with multiple commit types records per-type lines

- **WHEN** a weekly bucket contains commits of type `feat` adding 300 lines and type `fix` adding 120 lines
- **THEN** the `TimelineEntry` SHALL have `lines: 420`, `type_lines: {"feat": 300, "fix": 120}`, and `type_lines` values SHALL sum to 420

#### Scenario: Bucket with a single commit type

- **WHEN** a daily bucket contains only `docs` commits totalling 50 lines
- **THEN** the `TimelineEntry` SHALL have `lines: 50` and `type_lines: {"docs": 50}`

#### Scenario: Zero-line types are omitted from the map

- **WHEN** a bucket contains `feat` commits with 200 lines and no `fix` commits
- **THEN** the `type_lines` map SHALL contain `{"feat": 200}` and SHALL NOT contain a `"fix"` key

#### Scenario: Empty bucket has an empty type_lines map

- **WHEN** a contiguous gap bucket is generated with zero commits
- **THEN** the `TimelineEntry` SHALL have `lines: 0` and `type_lines` SHALL be an empty map
