## Why

The stacked type timeline chart shows per-type line distribution visually, but users cannot see the exact **per-type line counts** for each time interval. Adding a hover tooltip enables quick inspection of the underlying data without cluttering the chart, making it easier to understand contribution patterns at a glance.

## What Changes

- Render a floating tooltip on bar hover in `renderTimeline()` showing the time label, total lines, and a per-type breakdown of lines changed (using existing `type_lines` data, with colored dots matching bar segment colors).
- Add CSS for the tooltip element (positioned above the hovered bar, with a subtle background and pointer).
- Tooltip is hidden by default and appears on `mouseenter`, disappears on `mouseleave`.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `html-report-generation`: Add tooltip rendering on bar hover displaying time label, total lines, and per-type line counts with colored indicators.

## Impact

- `templates/page.html` — Add tooltip HTML generation in `renderTimeline()`, CSS for tooltip styling, mouseenter/mouseleave event listeners
- `docs/contribution-showcase.html` — Regenerate
