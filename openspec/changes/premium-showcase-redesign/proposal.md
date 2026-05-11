# premium-showcase-redesign

## Why

The current HTML dashboard uses a "Soft Paper" theme with simple cards on a plain background — it looks crude and unprofessional. The continuous scrolling layout makes it hard for viewers to focus on individual sections, and the complete absence of animations or visual transitions leaves the presentation feeling static and lifeless.

A key data gap also exists: there is no metric for average daily code line changes, which is a natural way to communicate development velocity to stakeholders. The dashboard needs to be **presentation-ready** — something that can be shown directly in stakeholder reviews without apology.

## What Changes

- **Complete HTML template redesign** — replace the continuous scroll layout with slide-style page-by-page scrolling (`scroll-snap-type: y mandatory`). Each section becomes a full-viewport slide, giving every topic visual room to breathe.
- **New "Editorial Paper" design system** — serif display fonts, warm paper palette, professional typography craft (negative tracking for headlines, a 3-weight type system, tabular numbers for data columns).
- **Scroll-triggered animations** — fade-in sections, animated counters for numeric KPIs, and bar chart growth animations, all driven by `IntersectionObserver`.
- **Navigation indicator** — a fixed dot navigation rail on the side showing the current slide position and allowing click-to-jump.
- **7-slide UX flow**: Cover → KPI Dashboard → Timeline → Type Breakdown → Projects → Proposals → Commit Log.
- **New data metric: average daily lines changed** — computed as total line changes divided by the count of unique commit dates (not calendar days), giving a more accurate picture of coding velocity on active days.
- **Enhanced table design** — row hover effects, improved visual density, and refined type/status badges.
- **Responsive behaviour** — slides reflow gracefully on smaller viewports and mobile screens.

## Capabilities

### New Capabilities

- **`slide-navigation`** — Full-viewport slide-based navigation using CSS scroll-snap, a dot indicator rail, and keyboard arrow-key support.
- **`scroll-animations`** — `IntersectionObserver`-based scroll-triggered animations for numeric counters, horizontal bar growth, and fade-in entrance effects.
- **`daily-lines-metric`** — New "average daily lines changed" metric computed from the count of unique commit dates in the repository data.

### Modified Capabilities

- **`html-report-generation`** — Complete template redesign adopting the Editorial Paper design system, slide layout, scroll-triggered animations, and refined visual language.
- **`rust-cli-scaffold`** — `ShowcaseData` model extended with an `avg_daily_lines: f64` field in the `Summary` struct.
- **`git-log-collection`** — Collector updated to compute the number of unique commit dates and derive the average daily line changes metric.

## Impact

| File | Change |
|---|---|
| `templates/page.html` | Complete rewrite (~800+ lines of HTML/CSS/JS) |
| `src/model.rs` | Add `avg_daily_lines: f64` to `Summary` struct |
| `src/collector.rs` | Compute unique commit dates count and average daily lines metric |
| `src/main.rs` | No changes needed (template injection unchanged) |

No dependency changes. Output JSON schema adds `avg_daily_lines` field (additive, non-breaking).
