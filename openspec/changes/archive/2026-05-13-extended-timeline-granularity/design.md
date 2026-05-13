## Context

The `build_timeline()` function in `src/collector.rs` currently uses a three-level granularity selection:
- span ≤ 13 days → daily (`%Y-%m-%d`)
- span ≤ 60 days → weekly (`%G-W%V`)
- span > 60 days → monthly (`%Y-%m`)

This produces at most ~60 bars for the monthly case, but projects spanning multiple years would produce 24+ monthly bars which becomes visually cramped. The uniform "14 bucket" rule keeps the bar count predictable (always ≤ 14) while adding quarterly and yearly levels for longer time spans.

No backward compatibility concerns — version 0.1.0 has zero users.

## Goals / Non-Goals

**Goals:**
- Replace the three-tier granularity with five tiers using a uniform "≤ 14 distinct buckets at the current level" decision rule (yearly fallback may exceed 14 for multi-decade spans)
- Add quarterly (`YYYY-Qn`) and yearly (`YYYY`) bucket formats
- Generate contiguous buckets at all five granularities
- Maintain height normalization behavior (0–100 based on max)
- Keep single-function simplicity (no config knobs)
- Use inclusive day counting: `(max_date - min_date).num_days() + 1` for the daily threshold

**Non-Goals:**
- Custom user-configurable thresholds
- Frontend display changes (the JS already handles arbitrary label strings)
- Half-year or decade granularity levels

## Decisions

### Decision 1: Threshold Calculation by Calendar Units

Use actual calendar semantics with inclusive counting:
- Day count = `(max_date - min_date).num_days() + 1` (inclusive). If ≤ 14 → daily.
- Week count = number of distinct `%G-W%V` labels (ISO week-year format) from min_date through max_date. If ≤ 14 → weekly.
- Month count = `(max_year - min_year) * 12 + (max_month - min_month) + 1`. If ≤ 14 → monthly.
- Quarter count = `(max_year - min_year) * 4 + (max_quarter - min_quarter) + 1`. If ≤ 14 → quarterly.
- Otherwise → yearly (no upper bound — yearly is the terminal fallback).

**Rationale**: Calendar-based counting avoids edge cases with fixed day thresholds. Inclusive counting matches intuition (a span from day 1 to day 14 = 14 days).

### Decision 2: ISO Week Counting Uses `%G-W%V` Format

Count weeks using ISO week-year (`%G`) and ISO week number (`%V`), NOT calendar year. This correctly handles year boundaries where December dates may belong to ISO week 1 of the following year (e.g., 2024-12-30 is `2025-W01`).

**Rationale**: Matches the label format used for display; ensures counting logic and generation logic produce consistent results.

### Decision 3: Quarter Label Format

Use `YYYY-Qn` format where n ∈ {1, 2, 3, 4}. Example: `2025-Q1` for January–March 2025. This is unambiguous and widely recognized.

### Decision 4: Year Label Format

Use plain `YYYY` format. Example: `2025`.

### Decision 5: Contiguous Quarterly Buckets

Enumerate quarters from (year, quarter) of min_date to (year, quarter) of max_date. Quarter is derived as `(month - 1) / 3 + 1`.

### Decision 6: Compute Granularity Once via Enum

Determine the granularity level once into a local enum value, then use that single value for both bucket aggregation and contiguous label generation. This avoids duplicated cascade logic that could drift between aggregation and generation paths.

## Risks / Trade-offs

- **More test cases**: Five granularity levels require more boundary tests than three. Mitigation: structured test coverage per level with explicit 14→15 boundary tests.
- **Quarter boundary display**: Some users might not immediately recognize `Q1`–`Q4` labels. Mitigation: the format is widely standard and the chart already renders arbitrary labels.
- **Yearly fallback has no upper bound**: For multi-decade projects, yearly buckets may exceed 14. This is acceptable since yearly is the terminal fallback and there is no meaningful larger granularity for a contribution report.
- **ISO week-year boundaries**: Dates near Dec/Jan boundaries may produce unexpected ISO week-year labels. Mitigation: use `%G-W%V` consistently and add explicit boundary tests.
