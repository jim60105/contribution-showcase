## Why

The Timeline and Type Breakdown charts currently use **commit count** as their
unit, which gives equal weight to a one-line typo fix and a 2 000-line feature
branch. Switching to **lines changed** (insertions + deletions) makes these
charts reflect actual effort and code volume, producing a more honest
contribution picture.

Separately, the smallest text in the dashboard is **12 px** (labels, badges,
bar annotations). On high-DPI displays and at normal viewing distance this is
physically tiny and strains readability. Raising the typographic floor to
**18 px** improves legibility across the board and aligns with large-screen
data-dashboard conventions where no text should compete for attention below
a comfortable reading threshold.

## What Changes

### Charts: lines as unit

- **Timeline chart** — each weekly bar represents `lines_added + lines_removed`
  for that week instead of commit count. Height normalisation is recalculated
  against the peak-lines week. The peak annotation shows the lines total.
- **Type Breakdown chart** — each bar represents total lines changed for that
  commit type instead of commit count. Percentages are recomputed as
  `type_lines / total_lines * 100`.
- **Backend data model** — `TimelineEntry.count` is replaced by `lines` (usize).
  `TypeBreakdown.count` is replaced by `lines` (usize). Percentage computation
  changes from commit-ratio to lines-ratio.
- **Frontend labels** — "次" (times) replaced by "行" (lines); peak annotation
  and breakdown labels updated to match.

### Typography floor at 18 px

- **`--fs-xs`** raised from 12 px → 18 px.
- **`--fs-small`** raised from 14 px → 20 px.
- **`--fs-body`** raised from 16 px → 22 px.
- All `clamp()` heading/metric scales shift proportionally upward.
- `.sort-arrow` raised from 10 px → 18 px.
- `.scroll-indicator` raised from 24 px → 28 px.
- Visual hierarchy at the bottom of the scale (xs/small/body) is maintained
  through weight, colour, and letter-spacing rather than large size gaps.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `git-log-collection`: `build_timeline()` and `build_type_breakdown()` switch
  from commit counting to lines-changed summation; `TimelineEntry` and
  `TypeBreakdown` struct fields change.
- `html-report-generation`: Timeline and Breakdown JavaScript renderers consume
  the new `lines` field instead of `count`; labels change from "次" to "行";
  CSS typography custom properties and all size values that were below 18 px
  are raised.
- `scroll-animations`: Animation parameters (translateY, stagger delays) may
  need minor tuning if larger text alters layout heights (no functional change
  expected, but delta reserves the option).

## Impact

- **`src/model.rs`** — `TimelineEntry` and `TypeBreakdown` field rename/type
  change.
- **`src/collector.rs`** — `build_timeline()` and `build_type_breakdown()`
  rewrite accumulation logic; `build_project_data()` top_types switch to
  lines-based semantics; existing unit tests must be updated for new
  semantics.
- **`templates/page.html`** — CSS `:root` variables, `.caps`, `.badge`,
  `.bar-label`, `.peak-annotation`, `.data-table`, `.sort-arrow`,
  `.cover-generated`, `.nav-dot-label`, `.kpi-unit`, `.breakdown-label`,
  `.breakdown-pct`, `.project-stats`, `.project-lines`, `.show-more-btn`,
  `.empty-state` — all size tokens updated. JavaScript `renderTimeline()` and
  `renderBreakdown()` consume new data fields.
- **`openspec/specs/`** — 3 main specs receive delta updates.
- **No API changes** — the CLI interface and config file are unaffected.
- **No migration** — project is pre-release with zero users.
