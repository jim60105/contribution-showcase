## Why

The current timeline granularity uses fixed thresholds (≤13 days → daily, ≤60 days → weekly, >60 days → monthly) which makes the timeline unreadable for very long contribution periods. A project spanning multiple years would still show monthly buckets, producing 50+ bars that are too narrow to be useful. We need a uniform "escalation at 14 buckets" rule that naturally scales from days up to years, keeping the chart consistently readable regardless of time span.

## What Changes

- **Replace the three-level granularity selection** with a five-level cascade using a uniform threshold of 14:
  1. If span includes ≤ 14 calendar days (inclusive count: `max - min + 1`) → daily buckets (`YYYY-MM-DD`)
  2. Else if span produces ≤ 14 distinct ISO weeks (`%G-W%V` labels) → weekly buckets (`YYYY-Www`)
  3. Else if span produces ≤ 14 distinct months → monthly buckets (`YYYY-MM`)
  4. Else if span produces ≤ 14 distinct quarters → quarterly buckets (`YYYY-Qn`)
  5. Else → yearly buckets (`YYYY`) — note: yearly is the final fallback and may exceed 14 buckets for very long histories
- **Add quarterly bucket support** with `YYYY-Qn` label format (e.g., `2025-Q1`)
- **Add yearly bucket support** with `YYYY` label format (e.g., `2025`)
- **Generate contiguous buckets** for all five granularities (fill gaps with zero-line entries)
- **Update existing tests** to match new thresholds and add boundary tests for each level transition

## Capabilities

### New Capabilities

_None — this change extends the existing capability._

### Modified Capabilities

- `adaptive-timeline-granularity`: Replace the three-level day/week/month selection with a five-level day/week/month/quarter/year cascade using uniform ≤14 bucket thresholds. Add quarterly and yearly bucket formats and contiguous generation for all levels.

## Impact

- `src/collector.rs` — `build_timeline()` function: threshold logic, bucket label formatting, contiguous label generation
- `src/collector.rs` — All existing timeline tests need threshold updates
- `templates/page.html` — Frontend JS already renders arbitrary labels from JSON; no changes expected unless label formatting needs special display handling
- `openspec/specs/adaptive-timeline-granularity/spec.md` — Requirements updated
