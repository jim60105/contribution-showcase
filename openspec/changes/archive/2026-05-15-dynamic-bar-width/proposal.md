## Why

The timeline bar chart currently uses `flex: 1` on `.bar-wrapper`, making each
bar expand equally to fill the chart width. With a fixed CSS `gap` between bars,
the visual density depends on the number of buckets—many bars crowd together
while few bars appear overly wide. A width-based approach where total bar area
occupies exactly half the chart width gives consistent, balanced spacing
regardless of bucket count.

## What Changes

- Replace the `flex: 1` sizing on `.bar-wrapper` with a JavaScript-computed
  width: `100% / (N * 2)` where N is the number of timeline buckets.
- Remove the `gap` property from `.timeline-chart` since spacing is now an
  implicit consequence of each bar taking half the available space per slot.
- Add `justify-content: space-evenly` (or equivalent) to `.timeline-chart` so
  the remaining 50 % of width distributes as even gaps around bars.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `html-report-generation`: The timeline bar chart rendering logic changes from
  flex-equal sizing with a fixed pixel gap to dynamically computed bar widths
  where total bar area equals half the chart width.

## Impact

- `templates/page.html` — CSS changes to `.timeline-chart` and `.bar-wrapper`;
  JavaScript changes in the timeline rendering function to compute and apply bar
  widths.
- No Rust code changes required (pure front-end change).
- No data model or config changes.
