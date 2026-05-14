## 1. Tooltip CSS

- [x] 1.1 Add CSS for `.timeline-tooltip` element (absolute positioning, background, border-radius, padding, pointer-events none, opacity transition, z-index)
- [x] 1.2 Add CSS for `.tooltip-header` (label + total lines)
- [x] 1.3 Add CSS for `.tooltip-row` with `.tooltip-dot` (colored indicator circle) and line count text

## 2. Tooltip Rendering and Events

- [x] 2.1 Ensure timeline chart container (`#timelineChart`) has `position: relative` for absolute tooltip positioning
- [x] 2.2 In `renderTimeline()`, append a single shared `<div class="timeline-tooltip">` element to the chart container
- [x] 2.3 Add `mouseenter` event listener on each `.bar-wrapper` to populate tooltip content (label, total lines, per-type line counts with colored dots) ordered using global type order, and position it centered above the bar
- [x] 2.4 Add `mouseleave` event listener on each `.bar-wrapper` to hide the tooltip
- [x] 2.5 Clamp tooltip horizontal position to prevent overflow outside the chart container
- [x] 2.6 For zero-line gap buckets, show only "{label}（0行）" with no type rows

## 3. Tests and Verification

- [x] 3.1 Run full test suite (`cargo test`) to verify all existing tests pass
- [x] 3.2 Add integration test verifying `.timeline-tooltip` class appears in generated HTML

## 4. Regenerate Docs

- [x] 4.1 Run `cargo run -- generate` and copy `dist/index.html` to `docs/contribution-showcase.html`
