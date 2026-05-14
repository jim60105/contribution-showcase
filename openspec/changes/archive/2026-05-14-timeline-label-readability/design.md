# Design: timeline-label-readability

## Context

The timeline chart in `templates/page.html` has X-axis labels (`.bar-label`) that are rotated -45° and positioned `bottom: -36px` with only `padding-bottom: 40px` on the chart container. This causes labels to overlap with bars above them, especially for longer labels like "2026-W20" or daily labels like "2025-03-15". The CSS variables were also audited and all are now properly defined (fixed in previous change).

## Design Decisions

### D1: Increase chart padding-bottom

- **Choice**: Increase `.timeline-chart` padding-bottom from `40px` to `80px` to accommodate worst-case rotated labels.
- **Rationale**: With 18px font and -45° rotation, the vertical projection of a label is approximately `labelWidth × sin(45°)`. Weekly labels like "2026-W20" need ~70px, daily labels like "2025-03-15" need ~80px. 80px padding covers all granularity levels.
- **Alternative**: Could reduce font size or rotation angle, but labels would become too small or harder to read.

### D2: Adjust bar-label geometry

- **Choice**: Change `.bar-label` from `bottom: -70px; transform-origin: top left` to `top: 100%; right: 50%; margin-top: 4px; transform-origin: top right`. This anchors the label's top-right corner at the bottom-center of the bar wrapper, making the rotated text extend downward-left into the padding zone.
- **Rationale**: With `transform-origin: top left` and `rotate(-45deg)`, the right end of the label extends upward into the bar area. Switching to `transform-origin: top right` ensures all parts of the rotated label remain below the bars.

### D3: CSS variable audit (already complete)

- **Choice**: Verify every static `var(--X)` reference resolves correctly.
- **Note**: The `--card-bg` → `--surface` and `--fs-sm` → `--fs-xs` fixes were already applied in the tooltip change. No further CSS variable changes needed.

## Goals

- Labels readable without overlap at all granularities (daily, weekly, monthly, quarterly, yearly)
- No layout shift on existing chart content area

## Non-Goals

- Redesigning the label rotation or layout approach
- Adding hover/truncation for very long labels

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Increased chart vertical space | ~40px more padding | Minimal impact on overall page layout |
| Very long labels at daily granularity | May still extend beyond chart bounds | 80px padding handles up to ~10-char date labels at 18px font |
