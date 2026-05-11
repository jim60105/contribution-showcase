# Tasks: premium-showcase-redesign

Complete redesign of the contribution-showcase HTML template (`templates/page.html`, ~835 lines) into a slide-style, editorial-paper presentation with scroll-triggered animations, plus a new backend metric.

---

## 1. Backend: Average Daily Lines Metric

- [x] In `src/model.rs`, add `avg_daily_lines: f64` field to the `Summary` struct
- [x] In `src/collector.rs` `collect()`, after filtering, compute unique commit dates using a `HashSet<String>` of `commit.date` values
- [x] Calculate `avg_daily_lines = (lines_added + lines_removed) as f64 / unique_dates.max(1) as f64`
- [x] Set `avg_daily_lines` in the `Summary` struct construction

## 2. Backend: Unit Tests for New Metric

- [x] `test_avg_daily_lines_multiple_dates`: commits on 3 different dates with known line counts → verify correct average
- [x] `test_avg_daily_lines_same_date`: multiple commits on same date → unique dates = 1
- [x] `test_avg_daily_lines_no_commits`: empty commit list → avg = 0.0

## 3. Template: Design System & Base Styles

- [x] Define `:root` CSS custom properties — palette (`--bg`, `--surface`, `--fg`, `--muted`, `--border`, `--accent`), typography (`--font-display`, `--font-body`, `--font-mono`), scale (`--fs-display` through `--fs-xs` using `clamp()`), spacing (8-point grid `--gap-xs` to `--gap-2xl`), commit type colors
- [x] CSS reset: `box-sizing: border-box`, margin/padding reset, `-webkit-font-smoothing: antialiased`
- [x] Base body styles: `font-family: var(--font-body)`, `background: var(--bg)`, `color: var(--fg)`, `font-size: var(--fs-body)`, `line-height: 1.6`
- [x] Typography classes: `.display` (serif, display size, tight tracking), `.h1`, `.h2`, `.label` (sans, small, caps tracking), `.mono-num` (mono, tabular-nums)
- [x] Container: `max-width: var(--container)`, `margin-inline: auto`, `padding-inline: var(--gap-lg)`
- [x] Card component: white surface, 1px border, `--radius`, `20px` padding
- [x] Type badge component: inline pill with background tint, colored text, small font

## 4. Template: Slide Layout & Navigation

- [x] `<main>` element with `scroll-snap-type: y mandatory`, `overflow-y: auto`, `height: 100vh`
- [x] `<section class="slide">` for each slide: `min-height: 100vh`, `scroll-snap-align: start`, flex centering
- [x] Fixed navigation dots: `position: fixed`, right side, vertically centered, one dot per slide
- [x] Active dot styling: larger/filled vs small/hollow
- [x] Dot click handler: scroll to corresponding section via `scrollIntoView({ behavior: 'smooth' })`
- [x] IntersectionObserver to update active dot on scroll (observe each section)
- [x] Keyboard navigation: listen for ArrowUp/ArrowDown/PageUp/PageDown to navigate between slides
- [x] Set `aria-current="true"` on active navigation dot, remove from previous

## 5. Template: Cover Slide (Slide 1)

- [x] Full viewport centered content
- [x] Title in `.display` (serif, large)
- [x] Subtitle: author name • date range • generated timestamp
- [x] Hero metric: total commits as large counter number
- [x] "↓" scroll indicator at bottom with subtle pulse animation

## 6. Template: KPI Dashboard (Slide 2)

- [x] Section title: "關鍵指標" (Key Metrics)
- [x] 3×2 grid of metric cards (responsive to 2×3 on tablet, 1×6 on mobile)
- [x] Each card: label (`.label`, small caps), value (`.mono-num`, large), optional delta/unit
- [x] Cards: Total Commits, Active Repos, OpenSpec Proposals, Lines Added (+, green tint), Lines Removed (−, red tint), Avg Daily Lines (行/天, accent tint)
- [x] Counter animation triggered by IntersectionObserver
- [x] Stagger animation: cards appear 100ms apart

## 7. Template: Timeline Chart (Slide 3)

- [x] Section title: "時間軸" (Timeline)
- [x] CSS flexbox bar chart: flex container with items at bottom
- [x] Each bar: width proportional to allocated space, height from `entry.height` percentage
- [x] Bar color: `var(--accent)` at reduced opacity
- [x] Week labels below bars (rotated 45° for space if many weeks)
- [x] Bar height animation: start at 0, grow to final height on scroll-in
- [x] Peak bar highlighted with accent color and annotation label

## 8. Template: Type Breakdown (Slide 4)

- [x] Section title: "貢獻類型" (Contribution Types)
- [x] Horizontal bar chart: each type gets a row with label, bar, percentage
- [x] Bar width = percentage, colored by commit type CSS variable
- [x] Label column: zh-TW type name + count
- [x] Animated bar width growth on scroll-in
- [x] Stagger: bars appear 80ms apart

## 9. Template: Project Cards (Slide 5)

- [x] Section title: "專案" (Projects)
- [x] Card grid: 2 columns on desktop (responsive to 1 on mobile)
- [x] Each card: project name (bold), description, stats row (commits / proposals / lines +/−)
- [x] Top commit types as colored badges
- [x] Cards fade-in with stagger on scroll

## 10. Template: Proposals Table (Slide 6)

- [x] Section title: "OpenSpec 提案" (Proposals)
- [x] Internal scroll container (`overflow-y: auto`, `max-height: calc(100vh - 160px)`)
- [x] Table with columns: Project, Slug, Date, Description, Tasks
- [x] Clean table design with alternating row tint, hover highlight
- [x] Group by project visually (project name in first column spans or shown once)

## 11. Template: Commit Log (Slide 7)

- [x] Section title: "提交紀錄" (Commit Log)
- [x] Internal scroll container
- [x] Sortable table: Date, Project, Type, Description, +/− columns
- [x] Sort by clicking column header (ascending/descending toggle with arrow indicator)
- [x] Type badges with commit type colors
- [x] 100-row initial limit with "顯示全部" (Show All) toggle button
- [x] Compact row design for data density
- [x] Sortable `<th>` cells contain `<button>` elements with `aria-sort` attribute
- [x] Sort buttons respond to Enter and Space keys

## 12. Template: Animations Engine

- [x] `initAnimations()` function creating IntersectionObserver with threshold 0.2
- [x] `.animate-in` class: elements with `opacity: 0; transform: translateY(24px)`, transition to final state when `.in-view` added
- [x] `.animate-stagger` container: children animate with `transition-delay: calc(var(--i, 0) * 100ms)` (set `--i` via `style` attribute or data attribute)
- [x] `animateCounter(element, target, duration)`: animate number from 0 to target with easing, format with `toLocaleString('zh-TW')`
- [x] `animateBar(element, width, delay)`: animate bar width from 0 to target percentage
- [x] All animations respect `prefers-reduced-motion: reduce` — instant display without animation

## 13. Template: Responsive Behavior

- [x] `@media (max-width: 1199px)`: KPI grid becomes 2×3, project grid stays 2 columns
- [x] `@media (max-width: 920px)`: disable scroll-snap, hide nav dots, all grids become single column, sections become normal flow with padding
- [x] Typography scales via `clamp()` — no media query needed for font sizes

## 14. Build and Test Verification

- [x] `cargo test` — verify all existing + new unit tests pass
- [x] `cargo build --release` — verify successful compilation
- [x] Run the tool — verify output contains `avg_daily_lines` in JSON
- [x] Verify HTML is valid (opens without errors in browser)
- [x] Verify slide navigation works (scroll-snap, dots, keyboard)
- [x] Verify animations trigger on scroll
- [x] Verify counter animations work
- [x] Verify responsive behavior at 1199px and 920px breakpoints
