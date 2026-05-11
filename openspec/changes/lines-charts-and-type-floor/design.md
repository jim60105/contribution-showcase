# Design: Lines-Based Charts & Typography Floor at 18px

## Context

The contribution-showcase is a Rust CLI that generates a self-contained HTML
report visualizing Git contribution data. Two independent but simultaneously
shipped changes improve the report's information density and readability:

1. **Charts use lines changed instead of commit count.** The Timeline and Type
   Breakdown charts currently count commits — a metric that conflates a
   one-line typo fix with a 500-line refactor. Switching to
   `lines_changed = insertions + deletions` better reflects actual effort and
   impact.

2. **Typography floor raised to 18px.** The current smallest token (`--fs-xs`)
   is 12px, which is borderline illegible on high-DPI displays at normal
   viewing distance. Raising the floor to 18px and compressing the scale upward
   ensures every text element remains comfortable to read while preserving
   visual hierarchy.

### Current Data Flow

```
CommitEntry { insertions, deletions, ... }
        │
        ▼
build_timeline()        → Vec<TimelineEntry { label, count, height }>
build_type_breakdown()  → Vec<TypeBreakdown { commit_type, label, count, percentage }>
        │
        ▼
page.html (Tera template)
  renderTimeline()   → bar chart, peak annotation "高峰：{label}（{count}次）"
  renderBreakdown()  → horizontal bars, labels "{label}（{count}）", percentages
```

Both `build_*` functions aggregate by counting commits. The `insertions` and
`deletions` fields on `CommitEntry` are already parsed from `git log --stat`
but unused in chart computation.

---

## Goals / Non-Goals

**Goals:**

- Replace commit-count with lines-changed (`insertions + deletions`) as the
  primary metric in Timeline and Type Breakdown charts.
- Update all associated labels, annotations, and tooltips to reflect the new
  unit ("行" instead of "次").
- Raise the minimum font size to 18px and redesign the full typography scale so
  that hierarchy is preserved with a compressed bottom range.
- Maintain the self-contained, single-file HTML output contract.

**Non-Goals:**

- Separate "insertions" vs "deletions" breakdown per bar (future enhancement).
- Adding new chart types or metrics beyond what currently exists.
- Responsive breakpoint changes — only the type scale changes, layout stays.
- Backward compatibility with previously generated reports (pre-release,
  0 users).

---

## Decisions

### D1: Unit is `lines_changed = insertions + deletions`, not net LOC

**Decision:** Sum insertions and deletions per commit, then aggregate by week /
type. Do not use net change (`insertions - deletions`).

**Rationale:**

- Net change hides refactoring effort (rename 200 lines → net 0).
- Absolute churn better represents "work done" for a showcase context.
- The field is renamed from `count` to `lines` (see D5) and the type remains
  `usize`.

### D2: Aggregation happens at the same grouping level

**Decision:** `build_timeline()` sums `insertions + deletions` for all commits
in each ISO week. `build_type_breakdown()` sums the same per `commit_type`.
Height normalization (`max → 100%`) and percentage computation remain identical
in logic.

**Rationale:** The normalization math is unit-agnostic; only the raw input
values change. This keeps the diff minimal and the logic auditable.

### D3: Large-number formatting in labels

**Decision:** Format lines-changed values with thousands separators (e.g.,
`12,345 行`) in the rendered HTML. Use a simple Rust helper
(`format_number(n: usize) -> String`) injected into the template context, or
a JS `toLocaleString()` call in the frontend render functions.

**Rationale:** Lines-changed values are 1–2 orders of magnitude larger than
commit counts. Without formatting, "12345" is hard to parse at a glance.

### D4: Typography floor at 18px with compressed scale

**Decision:** New scale:

| Token | Old | New | Delta |
|-------|-----|-----|-------|
| `--fs-xs` | 12px | 18px | +6 |
| `--fs-small` | 14px | 20px | +6 |
| `--fs-body` | 16px | 22px | +6 |
| `--fs-h2` | clamp(24px, 3vw, 36px) | clamp(28px, 3vw, 40px) | +4 |
| `--fs-h1` | clamp(32px, 4vw, 48px) | clamp(36px, 4vw, 52px) | +4 |
| `--fs-display` | clamp(48px, 6vw, 76px) | clamp(52px, 6vw, 84px) | +4/+8 |
| `.metric` | clamp(36px, 5vw, 56px) | clamp(40px, 5vw, 60px) | +4 |
| `.kpi-value` | clamp(28px, 4vw, 44px) | clamp(32px, 4vw, 48px) | +4 |
| `.hero-metric` | clamp(56px, 8vw, 96px) | clamp(60px, 8vw, 100px) | +4 |
| `.sort-arrow` | 10px | 18px | +8 |
| `.scroll-indicator` | 24px | 28px | +4 |

**Rationale:**

- The bottom three steps (xs / small / body) each get +6px to lift them above
  the 18px floor and maintain a 2px inter-step rhythm.
- Heading and display sizes get a smaller +4px bump — they were already
  comfortable; excessive growth would make them dominate the viewport.
- `.sort-arrow` jumps to 18px (the new floor) because below-floor sizes are
  banned by this change.
- The resulting ratio between body (22px) and display-max (84px) is ~3.8×,
  close to the previous 4.75× ratio — hierarchy is slightly compressed but
  still clearly stratified across four distinct levels (body → h2 → h1 →
  display).

### D5: Rename model fields from `count` to `lines`

**Decision:** `TimelineEntry.count` and `TypeBreakdown.count` are renamed to
`lines` to reflect the new semantic. The type remains `usize`.

**Rationale:** Keeping `count` while the value represents total lines changed
would be misleading for anyone reading the code. Since there are no external
consumers and no backward-compat requirement, a clean rename is preferable to
a semantic-only change with a confusing name.

---

## Risks / Trade-offs

### R1: Compressed bottom of the type scale

The gap between `--fs-xs` (18px) and `--fs-body` (22px) is only 4px total
(two 2px steps). On low-DPI screens this may feel visually flat.

**Mitigation:** Rely on font-weight and color contrast (already in use) to
differentiate caption-level text from body text, rather than size alone.

### R2: Large numbers in chart labels

A single week could show 10,000+ lines changed. Bar labels like
"高峰：2025-W03（12,345 行）" are longer than the old commit-count labels.

**Mitigation:**
- Use thousands separator formatting (D3).
- Timeline peak annotation already uses a tooltip-style overlay; verify it
  doesn't overflow the chart container at 5-digit values.
- If overflow is detected during implementation, abbreviate with "K" suffix
  (e.g., "12.3K 行") as a fallback — but prefer full numbers first.

### R3: Commits with zero stat lines

Some commits (e.g., merge commits, empty commits) may have
`insertions = 0, deletions = 0`. These currently contribute `count += 1` but
will now contribute `lines += 0`, effectively vanishing from charts.

**Mitigation:** This is acceptable — zero-line commits represent no code
change and should not inflate the chart. Document this behavior in code
comments.

### R4: Sort-arrow legibility at 18px

Raising `.sort-arrow` from 10px to 18px is a near-doubling. The arrow
character (▲/▼) may appear disproportionately large next to column headers.

**Mitigation:** Visually verify during implementation. If too large, reduce to
16px (still above the spirit of the 18px floor — arrows are decorative
glyphs, not readable text). The floor strictly applies to *text* content.

---

## Summary of Changes by File

| File | Change |
|------|--------|
| `src/model.rs` | Rename `count` → `lines` on `TimelineEntry` and `TypeBreakdown` |
| `src/collector.rs` | `build_timeline()`: sum `insertions + deletions` instead of `+= 1` |
| | `build_type_breakdown()`: same aggregation change |
| `templates/page.html` | Label strings: "次" → "行", add `toLocaleString()` formatting |
| | CSS custom properties: updated type scale per D4 table |
| `src/collector.rs` or `templates/page.html` | Number formatting (Rust helper or JS) |
| Tests | Update expected values in any snapshot/unit tests for charts |
