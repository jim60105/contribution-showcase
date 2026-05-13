# Adaptive Timeline Granularity

## Purpose

Automatically selects daily, weekly, or monthly timeline granularity based on
the date span of commits, ensuring the timeline chart is readable regardless of
whether the contribution period covers days or months.

## Requirements

### Requirement: Granularity Selection Based on Date Span
The system SHALL compute the date span between the earliest and latest commit dates and select the timeline granularity as follows:
- Span < 14 days → daily buckets
- 14 days ≤ span ≤ 60 days → weekly buckets (ISO week numbering)
- Span > 60 days → monthly buckets

#### Scenario: Short date range selects daily granularity
- **WHEN** the earliest commit date is `2025-03-10` and the latest commit date is `2025-03-18` (span = 8 days, < 14)
- **THEN** the timeline SHALL use daily buckets

#### Scenario: Medium date range selects weekly granularity
- **WHEN** the earliest commit date is `2025-01-01` and the latest commit date is `2025-02-15` (span = 45 days, between 14 and 60)
- **THEN** the timeline SHALL use weekly buckets

#### Scenario: Long date range selects monthly granularity
- **WHEN** the earliest commit date is `2025-01-01` and the latest commit date is `2025-06-30` (span = 180 days, > 60)
- **THEN** the timeline SHALL use monthly buckets

#### Scenario: Exactly 14 days selects weekly granularity
- **WHEN** the date span is exactly 14 days
- **THEN** the timeline SHALL use weekly buckets (14 days ≤ span ≤ 60 days)

#### Scenario: Exactly 60 days selects weekly granularity
- **WHEN** the date span is exactly 60 days
- **THEN** the timeline SHALL use weekly buckets (14 days ≤ span ≤ 60 days)

#### Scenario: 61 days selects monthly granularity
- **WHEN** the date span is 61 days
- **THEN** the timeline SHALL use monthly buckets (span > 60 days)

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

### Requirement: Contiguous Bucket Generation
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

### Requirement: Height Normalization
The `height` field SHALL be normalized to 0–100 based on the maximum lines value across all buckets, regardless of the selected granularity. This matches current behavior.

#### Scenario: Height normalization applies consistently across granularities
- **WHEN** monthly granularity is selected and the bucket with the most lines changed has 500 lines
- **THEN** that bucket's `height` SHALL be `100` and a bucket with 250 lines SHALL have `height` of `50`

#### Scenario: Single bucket gets height 100
- **WHEN** there is exactly one bucket (e.g., single-day edge case)
- **THEN** that bucket's `height` SHALL be `100`
