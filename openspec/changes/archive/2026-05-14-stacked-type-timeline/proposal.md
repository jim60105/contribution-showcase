## Why

The current timeline chart uses single-color bars showing only aggregate lines changed per time bucket, losing the commit-type composition that is visible in the separate 貢獻類型 breakdown. Stacking each bar by commit type — with matching colors — gives readers immediate insight into **what kind** of work drove each period's activity, combining temporal and categorical dimensions in a single, more informative chart. The section is also renamed from "時間軸" to "提交趨勢" to better describe the chart's analytical purpose.

## What Changes

- **Rename the "時間軸" slide** heading and navigation label to "提交趨勢" throughout the HTML template.
- **Extend `TimelineEntry`** with a `type_lines` field (`BTreeMap<String, usize>`) that records lines-changed per conventional commit type for each time bucket.
- **Update `build_timeline()`** in `collector.rs` to aggregate lines by commit type within each bucket, populating the new `type_lines` map.
- **Rewrite `renderTimeline()` JS** in `page.html` to render each bar as a stacked column of colored segments, where each segment's height is proportional to its type's lines relative to the bucket total, and the overall bar height uses the existing `height` (0–100) normalization. Segment colors use the same CSS custom properties (`--feat`, `--fix`, `--docs`, etc.) defined for the 貢獻類型 section.
- **Update existing tests** in `collector.rs` that assert on `TimelineEntry` to account for the new `type_lines` field.
- **Add new tests** verifying per-type breakdown correctness and stacked rendering data.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `adaptive-timeline-granularity`: Each timeline bucket now carries a per-type lines breakdown (`type_lines` map) alongside the existing aggregate `lines` and `height` fields. The granularity selection algorithm remains unchanged.
- `html-report-generation`: The timeline slide is renamed from "時間軸" to "提交趨勢", and bars are rendered as stacked columns using per-type color coding matching the 貢獻類型 palette.

## Impact

- **`src/model.rs`**: `TimelineEntry` struct gains `type_lines: BTreeMap<String, usize>`.
- **`src/collector.rs`**: `build_timeline()` aggregates by type; existing tests updated.
- **`templates/page.html`**: Section heading, nav label, `renderTimeline()` JS, and supporting CSS for stacked bar segments.
- **No config changes**: The change is purely in data collection and presentation.
- **No breaking changes**: JSON output gains an additive field; existing consumers are unaffected.
