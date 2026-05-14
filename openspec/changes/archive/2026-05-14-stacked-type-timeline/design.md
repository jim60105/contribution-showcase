# Design: Stacked Type Timeline

**Change**: `stacked-type-timeline`
**Proposal**: [proposal.md](proposal.md)

## Context

The timeline chart currently renders single-color bars showing aggregate lines changed per time bucket. Commit-type information is already collected per commit (`CommitEntry.commit_type`) and displayed separately in the 貢獻類型 breakdown, but the timeline discards it during aggregation. This change enriches the timeline data model and rendering to show per-type composition within each bar.

## Goals / Non-Goals

**Goals**:
- Add per-type line counts to `TimelineEntry` so stacked bars can be rendered.
- Render stacked columns in `renderTimeline()` using existing CSS color variables.
- Rename the section from "時間軸" to "提交趨勢".
- Keep the change additive — existing JSON consumers see a new field but nothing removed.

**Non-Goals**:
- Interactive tooltips or hover details (keep rendering simple; can be added later).
- A standalone legend component — the 貢獻類型 section already serves as the color key.
- Changes to the granularity cascade algorithm; bucket selection logic is untouched.

## Decisions

### D1: `BTreeMap<String, usize>` for `type_lines`

**Choice**: `BTreeMap<String, usize>` over `HashMap` or `Vec<(String, usize)>`.

**Rationale**:
- `BTreeMap` produces **deterministic iteration order** (alphabetical by type key), which guarantees consistent segment stacking across all bars and stable JSON serialization for snapshot testing.
- `HashMap` would require a separate sort step before rendering; `Vec<(String, usize)>` would require manual dedup and is less ergonomic for accumulation (`entry()` API).
- The number of commit types is small (≤11 known types), so `BTreeMap`'s overhead vs `HashMap` is negligible.

**Field addition**:
```rust
pub struct TimelineEntry {
    pub label: String,
    pub lines: usize,
    pub height: f64,
    pub type_lines: BTreeMap<String, usize>,  // new
}
```

`type_lines` only contains entries for types that have lines > 0 in that bucket. Zero-line types are omitted, not stored with value 0. This keeps serialized JSON compact — empty buckets produce `"type_lines": {}`.

### D2: Stacking order

**Choice**: Global descending order by total lines across all buckets, with `other` always last.

**Rationale**:
- The most prominent commit types appear at the bottom of each bar, making them easiest to read and compare across time.
- `other` is always stacked last (topmost) regardless of its total, since it's a catch-all category.
- The global order is computed once in JS from all timeline entries, then applied consistently to every bar — cross-bar comparison remains easy.
- Alternative (alphabetical `BTreeMap` order) was rejected because it doesn't surface the most visually important types prominently.
- `BTreeMap` is still used for storage (deterministic JSON serialization), but the JS renderer sorts by global totals before rendering.

### D3: Stacked bar height calculation

The existing `height` field (0–100, normalized against the peak bucket) remains the **total bar height**. Segment heights are percentages **within the bar**, not of the overall chart:

```
bar CSS height     = entry.height + '%'           (of chart area)
segment CSS height = (type_lines[type] / lines) * 100 + '%'  (of bar)
```

The parent `.bar` element is sized by `entry.height`; inner segment `<div>`s use percentage heights relative to the bar. This avoids double-scaling — a bar at 80% height with 70/30 segments renders segments occupying 70% and 30% of the bar's visual area.

For buckets where `lines == 0`, the bar renders at height 0 with no segments (same as current behavior for zero-activity buckets).

### D4: Collector changes — accumulation strategy

Inside `build_timeline()`, the current `bucket_lines: HashMap<String, usize>` is replaced with:

```rust
bucket_lines: HashMap<String, usize>,                       // unchanged aggregate
bucket_type_lines: HashMap<String, BTreeMap<String, usize>>, // new per-type map
```

During the commit iteration loop, each commit's `insertions + deletions` is added to both the aggregate bucket total and the per-type sub-map. The commit type key is **canonicalized**: known types (`feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`, `build`, `style`, `perf`) are stored as-is; all other types are mapped to `"other"`. This matches the existing `build_type_breakdown()` canonicalization and ensures unknown types aggregate correctly.

Commits with zero lines changed (`insertions + deletions == 0`) are **skipped** in the per-type map to satisfy the spec invariant that zero-line types are omitted from `type_lines`.

The `format_label` closure and contiguous-label generation remain untouched. When building the final `TimelineEntry` vec, `type_lines` is pulled from `bucket_type_lines` (defaulting to an empty `BTreeMap` for buckets with no commits).

### D5: JS rendering approach

`renderTimeline()` changes:
1. Each `.bar` div becomes a flex column (`display: flex; flex-direction: column; align-items: stretch; overflow: hidden`) so segments stack bottom-up with the first segment at the bottom.
2. Compute a global type order once: sum each type's lines across all timeline entries, sort descending, move `other` to the end. Emit segments in this fixed order for every bar.
3. For each entry, iterate the globally-sorted types and emit a `<div>` per segment with:
   - `height`: `(type_lines[type] / lines) * 100 + '%'` (percentage of the bar).
   - `background-color`: `var(--<type>)`, falling back to `var(--other)` for unknown types.
4. The peak annotation logic is unchanged — it still finds the bar with max `lines`. The peak bar **must not** override segment colors with `background: var(--accent)`.
5. `animateTimeline()` continues to work by setting total bar height via `data-height`; inner segments use percentage heights relative to the bar.

### D6: Section rename

Two locations in `page.html`:
- `NAV_LABELS` array: `'時間軸'` → `'提交趨勢'` (line 679).
- The `<h2>` heading for the timeline slide: `'時間軸'` → `'提交趨勢'`.

No config or model changes needed — these are purely template strings.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Bars with many types become visually noisy with thin slivers | Acceptable for ≤11 types; most buckets will have 2–4 active types. No minimum segment height is enforced — very small segments simply become hairlines, which is informative rather than misleading. |
| `BTreeMap` alphabetical order may not be the most visually intuitive stacking | Alphabetical is predictable and stable. If a "frequency-based" order is later desired, it can be computed in JS without model changes since `type_lines` carries all the data. |
| No standalone legend in the timeline section | The 貢獻類型 section uses the same colors and is typically viewed in the same session. Adding a legend later is a non-breaking enhancement. |
| Increased JSON payload size | Marginal — at most 14 buckets × ~4 active types × ~20 bytes each ≈ 1 KB additional. |
| Existing tests assert on `TimelineEntry` struct construction | Tests must be updated to include `type_lines`. Since the field is additive, this is mechanical — set `type_lines: BTreeMap::new()` for tests that don't care about type breakdown. |
