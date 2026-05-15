# Adaptive Timeline Granularity

## Purpose

Automatically selects daily, weekly, monthly, quarterly, or yearly timeline
granularity based on the date span of commits and the configurable
`timeline_max_buckets` threshold, ensuring the timeline chart is readable
regardless of whether the contribution period covers days, months, or years.

## MODIFIED Requirements

### Requirement: Granularity Selection Based on Date Span
The system SHALL compute the date span between the earliest and latest commit dates and select the timeline granularity using a five-level cascade. At each level, count the number of distinct buckets that would be produced; if ≤ `timeline_max_buckets`, use that granularity:
1. Count inclusive calendar days: `(max_date - min_date).num_days() + 1`. If ≤ `timeline_max_buckets` → daily buckets.
2. Else count distinct ISO week labels (`%G-W%V` format) from min_date through max_date. If ≤ `timeline_max_buckets` → weekly buckets.
3. Else count distinct months: `(max_year - min_year) * 12 + (max_month - min_month) + 1`. If ≤ `timeline_max_buckets` → monthly buckets.
4. Else count distinct quarters: `(max_year - min_year) * 4 + (max_quarter - min_quarter) + 1`. If ≤ `timeline_max_buckets` → quarterly buckets.
5. Else → yearly buckets (terminal fallback; may produce more than `timeline_max_buckets` buckets for multi-decade spans).

The `timeline_max_buckets` parameter is supplied at runtime via CLI argument, TOML config, or defaults to `14` when unspecified. See the `timeline-bucket-config` capability for resolution rules.

#### Scenario: Default threshold preserves original behavior
- **WHEN** `timeline_max_buckets` is `14` (the default) and commits span from `2025-03-01` to `2025-03-14` (14 days)
- **THEN** the timeline SHALL use daily buckets (14 ≤ 14)

#### Scenario: Lower threshold escalates daily to weekly sooner
- **WHEN** `timeline_max_buckets` is `7` and commits span from `2025-03-01` to `2025-03-10` (10 days, > 7)
- **THEN** the timeline SHALL NOT use daily buckets and SHALL escalate to weekly granularity (if distinct weeks ≤ 7)

#### Scenario: Higher threshold keeps daily granularity longer
- **WHEN** `timeline_max_buckets` is `21` and commits span from `2025-03-01` to `2025-03-20` (20 days, ≤ 21)
- **THEN** the timeline SHALL use daily buckets (20 ≤ 21)

#### Scenario: Lower threshold escalates weekly to monthly sooner
- **WHEN** `timeline_max_buckets` is `7` and commits span 10 distinct ISO weeks
- **THEN** the timeline SHALL NOT use weekly buckets (10 > 7) and SHALL escalate to monthly granularity (if distinct months ≤ 7)

#### Scenario: Higher threshold keeps weekly granularity longer
- **WHEN** `timeline_max_buckets` is `21` and commits span 18 distinct ISO weeks
- **THEN** the timeline SHALL use weekly buckets (18 ≤ 21)

#### Scenario: Threshold of 1 forces yearly for most spans
- **WHEN** `timeline_max_buckets` is `1` and commits span from `2025-01-01` to `2025-06-30` (181 days, 1 year)
- **THEN** the cascade SHALL skip daily (181 > 1), weekly (> 1), monthly (6 > 1), quarterly (2 > 1), and select yearly buckets (1 ≤ 1)

#### Scenario: Full cascade with non-default threshold
- **WHEN** `timeline_max_buckets` is `7` and commits span 16 distinct months (e.g., `2024-01` to `2025-04`)
- **THEN** the cascade SHALL skip daily (> 7), weekly (> 7), monthly (16 > 7), and check quarterly; if quarters ≤ 7, use quarterly buckets
