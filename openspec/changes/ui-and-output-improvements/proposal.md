## Why

The generated HTML report has several usability gaps: the output filename is always `index.html` regardless of content, the test metrics section buries aggregate KPI cards below the detail table, the project grid overflows the viewport without scrolling, and the timeline x-axis always shows weekly buckets even when the date range is a few days or several months. These four improvements enhance readability, navigation, and information density of the showcase report.

## What Changes

- **Dynamic output filename**: When no explicit output path is configured, the default filename derives from the `title` field in `showcase.toml` (sanitized to filesystem-safe characters) instead of the hardcoded `index.html`. The output directory remains `dist/` by default.
- **Test metrics layout reorder**: Move the `.test-kpi-row` summary cards (total files, total cases, average coverage) above the `.data-table` detail table in the 測試 section, so users see the high-level KPIs first.
- **Scrollable project grid**: Wrap `#projectGrid` in a `scroll-container` div so it becomes self-scrollable when content exceeds viewport height, matching the behavior of the commit log, proposal, and test metrics sections.
- **Adaptive timeline x-axis**: Change the timeline chart's time granularity based on the date range span:
  - Less than 2 weeks → daily buckets (label: `YYYY-MM-DD`)
  - 2 weeks to 2 months → weekly buckets (label: `YYYY-Www`)
  - More than 2 months → monthly buckets (label: `YYYY-MM`)

## Capabilities

### New Capabilities

- `dynamic-output-filename`: Derives the output HTML filename from the showcase title when no explicit path is configured, with filesystem-safe character sanitization.
- `adaptive-timeline-granularity`: Dynamically selects daily, weekly, or monthly time buckets for the timeline chart based on the commit date range span.

### Modified Capabilities

- `html-report-generation`: The test metrics section layout changes to render KPI summary cards above the detail table. The project cards section gains a scroll container for overflow handling.

## Impact

- **`src/config.rs`**: `output_path()` method changes to derive filename from `self.title`.
- **`src/collector.rs`**: `build_timeline()` function refactored to select granularity based on date span.
- **`src/model.rs`**: No structural changes expected; `TimelineEntry.label` format varies by granularity.
- **`templates/page.html`**: Test metrics section reordered (KPI row above table), project grid wrapped in scroll container, project slide gains `slide--scrollable` class.
- **Existing tests**: Timeline and config output_path tests need updates.
