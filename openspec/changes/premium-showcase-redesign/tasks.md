# Tasks: premium-showcase-redesign

Complete redesign of the contribution-showcase HTML template (`templates/page.html`, ~835 lines) into a slide-style, editorial-paper presentation with scroll-triggered animations, plus a new backend metric.

---

## 1. Backend: Average Daily Lines Metric

- [ ] In `src/model.rs`, add `avg_daily_lines: f64` field to the `Summary` struct
- [ ] In `src/collector.rs` `collect()`, after filtering, compute unique commit dates using a `HashSet<String>` of `commit.date` values
- [ ] Calculate `avg_daily_lines = (lines_added + lines_removed) as f64 / unique_dates.max(1) as f64`
- [ ] Set `avg_daily_lines` in the `Summary` struct construction

## 2. Backend: Unit Tests for New Metric

- [ ] `test_avg_daily_lines_multiple_dates`: commits on 3 different dates with known line counts → verify correct average
- [ ] `test_avg_daily_lines_same_date`: multiple commits on same date → unique dates = 1
- [ ] `test_avg_daily_lines_no_commits`: empty commit list → avg = 0.0

## 3. Template: Design System & Base Styles

- [ ] Define `:root` CSS custom properties — palette (`--bg`, `--surface`, `--fg`, `--muted`, `--border`, `--accent`), typography (`--font-display`, `--font-body`, `--font-mono`), scale (`--fs-display` through `--fs-xs` using `clamp()`), spacing (8-point grid `--gap-xs` to `--gap-2xl`), commit type colors
- [ ] CSS reset: `box-sizing: border-box`, margin/padding reset, `-webkit-font-smoothing: antialiased`
- [ ] Base body styles: `font-family: var(--font-body)`, `background: var(--bg)`, `color: var(--fg)`, `font-size: var(--fs-body)`, `line-height: 1.6`
- [ ] Typography classes: `.display` (serif, display size, tight tracking), `.h1`, `.h2`, `.label` (sans, small, caps tracking), `.mono-num` (mono, tabular-nums)
- [ ] Container: `max-width: var(--container)`, `margin-inline: auto`, `padding-inline: var(--gap-lg)`
- [ ] Card component: white surface, 1px border, `--radius`, `20px` padding
- [ ] Type badge component: inline pill with background tint, colored text, small font

## 4. Template: Slide Layout & Navigation

- [ ] `<main>` element with `scroll-snap-type: y mandatory`, `overflow-y: auto`, `height: 100vh`
- [ ] `<section class="slide">` for each slide: `min-height: 100vh`, `scroll-snap-align: start`, flex centering
- [ ] Fixed navigation dots: `position: fixed`, right side, vertically centered, one dot per slide
- [ ] Active dot styling: larger/filled vs small/hollow
- [ ] Dot click handler: scroll to corresponding section via `scrollIntoView({ behavior: 'smooth' })`
- [ ] IntersectionObserver to update active dot on scroll (observe each section)
- [ ] Keyboard navigation: listen for ArrowUp/ArrowDown/PageUp/PageDown to navigate between slides
- [ ] Set `aria-current="true"` on active navigation dot, remove from previous

## 5. Template: Cover Slide (Slide 1)

- [ ] Full viewport centered content
- [ ] Title in `.display` (serif, large)
- [ ] Subtitle: author name • date range • generated timestamp
- [ ] Hero metric: total commits as large counter number
- [ ] "↓" scroll indicator at bottom with subtle pulse animation

## 6. Template: KPI Dashboard (Slide 2)

- [ ] Section title: "關鍵指標" (Key Metrics)
- [ ] 3×2 grid of metric cards (responsive to 2×3 on tablet, 1×6 on mobile)
- [ ] Each card: label (`.label`, small caps), value (`.mono-num`, large), optional delta/unit
- [ ] Cards: Total Commits, Active Repos, OpenSpec Proposals, Lines Added (+, green tint), Lines Removed (−, red tint), Avg Daily Lines (行/天, accent tint)
- [ ] Counter animation triggered by IntersectionObserver
- [ ] Stagger animation: cards appear 100ms apart

## 7. Template: Timeline Chart (Slide 3)

- [ ] Section title: "時間軸" (Timeline)
- [ ] CSS flexbox bar chart: flex container with items at bottom
- [ ] Each bar: width proportional to allocated space, height from `entry.height` percentage
- [ ] Bar color: `var(--accent)` at reduced opacity
- [ ] Week labels below bars (rotated 45° for space if many weeks)
- [ ] Bar height animation: start at 0, grow to final height on scroll-in
- [ ] Peak bar highlighted with accent color and annotation label

## 8. Template: Type Breakdown (Slide 4)

- [ ] Section title: "貢獻類型" (Contribution Types)
- [ ] Horizontal bar chart: each type gets a row with label, bar, percentage
- [ ] Bar width = percentage, colored by commit type CSS variable
- [ ] Label column: zh-TW type name + count
- [ ] Animated bar width growth on scroll-in
- [ ] Stagger: bars appear 80ms apart

## 9. Template: Project Cards (Slide 5)

- [ ] Section title: "專案" (Projects)
- [ ] Card grid: 2 columns on desktop (responsive to 1 on mobile)
- [ ] Each card: project name (bold), description, stats row (commits / proposals / lines +/−)
- [ ] Top commit types as colored badges
- [ ] Cards fade-in with stagger on scroll

## 10. Template: Proposals Table (Slide 6)

- [ ] Section title: "OpenSpec 提案" (Proposals)
- [ ] Internal scroll container (`overflow-y: auto`, `max-height: calc(100vh - 160px)`)
- [ ] Table with columns: Project, Slug, Date, Description, Tasks
- [ ] Clean table design with alternating row tint, hover highlight
- [ ] Group by project visually (project name in first column spans or shown once)

## 11. Template: Commit Log (Slide 7)

- [ ] Section title: "提交紀錄" (Commit Log)
- [ ] Internal scroll container
- [ ] Sortable table: Date, Project, Type, Description, +/− columns
- [ ] Sort by clicking column header (ascending/descending toggle with arrow indicator)
- [ ] Type badges with commit type colors
- [ ] 100-row initial limit with "顯示全部" (Show All) toggle button
- [ ] Compact row design for data density
- [ ] Sortable `<th>` cells contain `<button>` elements with `aria-sort` attribute
- [ ] Sort buttons respond to Enter and Space keys

## 12. Template: Animations Engine

- [ ] `initAnimations()` function creating IntersectionObserver with threshold 0.2
- [ ] `.animate-in` class: elements with `opacity: 0; transform: translateY(24px)`, transition to final state when `.in-view` added
- [ ] `.animate-stagger` container: children animate with `transition-delay: calc(var(--i, 0) * 100ms)` (set `--i` via `style` attribute or data attribute)
- [ ] `animateCounter(element, target, duration)`: animate number from 0 to target with easing, format with `toLocaleString('zh-TW')`
- [ ] `animateBar(element, width, delay)`: animate bar width from 0 to target percentage
- [ ] All animations respect `prefers-reduced-motion: reduce` — instant display without animation

## 13. Template: Responsive Behavior

- [ ] `@media (max-width: 1199px)`: KPI grid becomes 2×3, project grid stays 2 columns
- [ ] `@media (max-width: 920px)`: disable scroll-snap, hide nav dots, all grids become single column, sections become normal flow with padding
- [ ] Typography scales via `clamp()` — no media query needed for font sizes

## 14. Build and Test Verification

- [ ] `cargo test` — verify all existing + new unit tests pass
- [ ] `cargo build --release` — verify successful compilation
- [ ] Run the tool — verify output contains `avg_daily_lines` in JSON
- [ ] Verify HTML is valid (opens without errors in browser)
- [ ] Verify slide navigation works (scroll-snap, dots, keyboard)
- [ ] Verify animations trigger on scroll
- [ ] Verify counter animations work
- [ ] Verify responsive behavior at 1199px and 920px breakpoints
