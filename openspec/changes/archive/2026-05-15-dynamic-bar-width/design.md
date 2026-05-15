## Context

The timeline bar chart in `templates/page.html` currently sizes bars via
`flex: 1` on `.bar-wrapper` with a `gap: 10px` on `.timeline-chart`. This
makes every bar equal-width, filling the container entirely. The gap creates
fixed-pixel spacing regardless of the number of bars or the viewport width.

The user wants a layout where total bar area equals exactly half the chart
width, with the remaining half distributed as spacing. This means bar width is
dynamically computed as `chartWidth / (N × 2)` where N is the bucket count.

## Goals / Non-Goals

**Goals:**

- Each bar width = `100% / (N × 2)` where N is the number of timeline buckets.
- Total bar area consumes exactly half the chart width; spacing consumes the
  other half.
- Bars are evenly distributed across the chart with equal gaps between and
  around them.
- Responsive: recalculates on window resize.

**Non-Goals:**

- Changing bar height calculation or segment stacking logic.
- Adding new config options for bar width control.
- Modifying Rust code or data models.

## Decisions

### 1. JavaScript-computed widths over pure CSS

**Decision**: Compute and apply bar widths in JavaScript after counting
timeline entries.

**Rationale**: CSS alone cannot express `100% / (N × 2)` where N is dynamic
data. CSS `calc()` requires a known value. JavaScript already renders the
bars and knows N.

**Alternative**: CSS custom property `--bar-count` set via JS, then
`width: calc(100% / (var(--bar-count) * 2))` in CSS. Rejected because it
still requires JS and adds indirection with no benefit.

### 2. Use `justify-content: space-evenly` for gap distribution

**Decision**: Remove the `gap` CSS property from `.timeline-chart` and use
`justify-content: space-evenly` to distribute remaining space equally.

**Rationale**: `space-evenly` distributes the unused 50% of width into
equal-sized gaps between and around bars, giving a balanced appearance.

**Alternative**: `space-between` (no outer padding) or `space-around` (half
outer padding). `space-evenly` was chosen for the most uniform look.

### 3. Set width on `.bar-wrapper` via inline style

**Decision**: Apply `width` as an inline style on each `.bar-wrapper` element
during rendering, calculated as `(100 / (timeline.length * 2))` percent.

**Rationale**: The JS already builds bar HTML in a loop. Adding a width style
there is trivial and keeps the logic co-located with the data.

### 4. Remove `flex: 1` from `.bar-wrapper` CSS

**Decision**: Remove `flex: 1` so the inline width is respected.

**Rationale**: `flex: 1` overrides explicit widths in a flex container by
making all items grow equally.

## Risks / Trade-offs

- **[Very many bars]** → With a large N (e.g., 365 daily buckets), each bar
  becomes very narrow (`100/(365×2) ≈ 0.14%`). This is acceptable as the
  existing layout has the same problem; extreme bucket counts already produce
  thin bars.
- **[Single bar]** → With N=1, the bar takes 50% width centered in the chart.
  This looks fine with `space-evenly`.
- **[Label overlap]** → Narrower bars may cause rotated labels to overlap more.
  No mitigation needed—existing label rotation and ellipsis handling applies.
