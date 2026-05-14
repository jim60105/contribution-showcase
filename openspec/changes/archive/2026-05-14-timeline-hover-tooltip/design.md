# Design: Timeline Hover Tooltip

## Overview

Add an interactive hover tooltip to the "提交趨勢" (Commit Trends) timeline chart. When users hover over a bar, a tooltip displays per-type line counts for that time bucket. This uses the existing `type_lines` field in the data model and adds a shared tooltip DOM element with vanilla JS event handling in the frontend.

## Design Decisions

### D1: Reuse existing `type_lines` data

- **Choice**: The tooltip displays per-type line counts from the existing `type_lines: BTreeMap<String, usize>` field on `TimelineEntry`. No new backend fields or collector changes are needed.
- **Rationale**: `type_lines` already carries exactly the data needed — per-type line counts per bucket. Adding a separate `type_commits` count field would require backend changes for minimal added value. Lines changed is the consistent unit across the entire report.

### D2: Tooltip as a single shared DOM element

- **Choice**: Create ONE tooltip `<div>` element appended to the timeline chart container. Reposition and repopulate it on `mouseenter` for each `.bar-wrapper`, rather than embedding tooltip markup inside every bar.
- **Rationale**: Reduces DOM bloat, simplifies CSS styling and positioning logic, and avoids overflow/clipping issues that arise when tooltips are children of small bar containers.
- **Alternative rejected**: Per-bar inline tooltip — produces more DOM nodes, is harder to position above bars without clipping.

### D3: Tooltip content structure

- **Choice**: Display the bucket label, total lines changed, then a list of type entries. Each entry shows a colored dot, the type name, and its line count formatted with `toLocaleString('zh-TW')` followed by `行`. Types are sorted in the same global order used for bar segments (descending total lines across all buckets, with "other" always last). Only types with lines > 0 in that bucket are listed.
- **Rationale**: Matching the bar segment stacking order makes the tooltip visually consistent with the chart.
- **Format**:
  ```
  {label}（{lines}行）
  ● feat: 900行
  ● fix: 600行
  ```
  Where `●` is colored using the type's existing CSS variable (e.g., `var(--feat)`).

### D4: Event handling approach

- **Choice**: Attach `mouseenter` and `mouseleave` event listeners on each `.bar-wrapper` element. On `mouseenter`, populate the tooltip content dynamically and position it centered above the hovered bar. On `mouseleave`, hide the tooltip by removing a visibility class.
- **Rationale**: Simple, dependency-free, consistent with the existing vanilla JS architecture in `templates/page.html`. No event delegation needed given the typically small number of bars (usually ≤ 14, except yearly fallback for multi-decade spans).
- **Note**: On touch devices the tooltip will not appear (no hover event). This is acceptable for the report's desktop-primary audience.

### D5: Tooltip positioning

- **Choice**: Position the tooltip using CSS `position: absolute` relative to the timeline chart container (which must have `position: relative`). Center the tooltip horizontally above the hovered bar, offset vertically by a fixed gap (e.g., 8px) above the bar's top edge.
- **Edge handling**: Clamp the tooltip's horizontal `left` value so it does not overflow the chart container's left or right edges. Specifically:
  - Compute the desired center: `barLeft + barWidth / 2 - tooltipWidth / 2`
  - Clamp to `[0, containerWidth - tooltipWidth]`
- **Rationale**: Absolute positioning within the container avoids viewport-relative calculations and works naturally with the existing layout.

### D6: Zero-line buckets

- **Choice**: For gap buckets where `type_lines` is empty, display only `"{label}（0行）"` with no type rows in the tooltip.
- **Rationale**: No meaningful data to display; a minimal tooltip avoids confusion and keeps the UI clean.

## Goals

- Show per-type line counts on bar hover in the timeline chart.
- Maintain fully self-contained HTML output (no new external dependencies).
- Reuse existing CSS variable colors (`--feat`, `--fix`, etc.) for type indicators in the tooltip.

## Non-Goals

- Touch/mobile tooltip support (hover-only interaction is sufficient).
- Click-to-pin tooltip behavior.
- Legend or permanent type key displayed outside the tooltip.
- Tooltip for other chart sections (scope limited to the timeline chart).

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| No hover on mobile/touch devices | Tooltip is inaccessible on tablets/phones | Acceptable for desktop-primary audience. A click handler could be added later as a separate enhancement. |
| Tooltip may overflow on narrow viewports | Tooltip text could be clipped or extend beyond the visible area | Horizontal clamping logic in D5 constrains the tooltip within the chart container bounds. |
| Tooltip flicker on rapid mouse movement between bars | Brief visual artifacts during fast sweeping | `mouseenter`/`mouseleave` pair on `.bar-wrapper` elements provides clean enter/exit semantics without flicker. |
