## Why

The timeline chart's X-axis labels (week/month/day identifiers) are rotated -45° and positioned too close to the bars, causing them to overlap with bar content and become unreadable. Additionally, some CSS variable references in the template were undefined (e.g., `--card-bg`, `--fs-sm`), which could cause fallback rendering issues. These visual quality issues need to be fixed to ensure the chart is readable and CSS is well-defined.

## What Changes

- Adjust `.bar-label` positioning and the chart's `padding-bottom` to give rotated labels enough space and prevent overlap with bars.
- Verify all CSS custom property references resolve to defined `:root` variables (already fixed in previous changes).

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `html-report-generation`: Fix timeline chart label positioning and CSS variable consistency.

## Impact

- `templates/page.html` — CSS adjustments for `.bar-label`, `.timeline-chart`, and any undefined CSS variable references.
- `docs/contribution-showcase.html` — Regenerate.
