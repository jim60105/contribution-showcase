# Tasks: lines-charts-and-type-floor

Switch Timeline and Type Breakdown charts from commit count to lines changed,
and raise the minimum font size from 12 px to 18 px with a proportionally
scaled typography hierarchy.

---

## 1. Backend: Model Field Rename

- [ ] 1.1 In `src/model.rs`, rename `TimelineEntry.count` to `TimelineEntry.lines`
- [ ] 1.2 In `src/model.rs`, rename `TypeBreakdown.count` to `TypeBreakdown.lines`

## 2. Backend: Timeline Lines Accumulation

- [ ] 2.1 In `src/collector.rs` `build_timeline()`, change accumulation from `+= 1` to `+= commit.insertions + commit.deletions`
- [ ] 2.2 Verify height normalisation still uses `lines / max_lines * 100.0`

## 3. Backend: Type Breakdown Lines Accumulation

- [ ] 3.1 In `src/collector.rs` `build_type_breakdown()`, change accumulation from `+= 1` to `+= commit.insertions + commit.deletions`
- [ ] 3.2 Change percentage from `count / total_count * 100` to `type_lines / total_lines * 100`
- [ ] 3.3 Ensure sorting is by descending `lines` (not commit count)

## 4. Backend: Project top_types Lines Semantics

- [ ] 4.1 In `src/collector.rs` `build_project_data()`, change `top_types` accumulation from commit count to lines changed
- [ ] 4.2 Ensure `top_types` sorting is by descending `lines`

## 5. Backend: Unit Test Updates

- [ ] 5.1 Update existing timeline unit tests to assert on lines-changed values instead of commit counts
- [ ] 5.2 Update existing type breakdown unit tests to assert on lines-changed values instead of commit counts
- [ ] 5.3 Add test for zero-stat commits (insertions=0, deletions=0) contributing 0 to timeline
- [ ] 5.4 Add test for all-zero timeline (all entries have lines=0) — height should be 0.0, no division by zero
- [ ] 5.5 Add test for all-zero type breakdown — percentage should be 0.0, no division by zero
- [ ] 5.6 Add test for type breakdown ordering (descending by lines)
- [ ] 5.7 Run `cargo test` — all tests pass

## 6. Template: Timeline Labels

- [ ] 6.1 In `renderTimeline()`, change peak annotation from `{count}次` to `{lines.toLocaleString('zh-TW')}行`
- [ ] 6.2 Change `entry.count` data access to `entry.lines` for value annotations (keep `entry.height` for bar rendering)

## 7. Template: Type Breakdown Labels

- [ ] 7.1 In `renderBreakdown()`, change label from `{count}` to `{lines.toLocaleString('zh-TW')}行`
- [ ] 7.2 Change `entry.count` data access to `entry.lines` for value labels (keep `entry.percentage` for bar width)

## 8. Template: Typography Scale — CSS Custom Properties

- [ ] 8.1 Update `--fs-xs` from `12px` to `18px`
- [ ] 8.2 Update `--fs-small` from `14px` to `20px`
- [ ] 8.3 Update `--fs-body` from `16px` to `22px`
- [ ] 8.4 Update `--fs-h2` from `clamp(24px, 3vw, 36px)` to `clamp(28px, 3vw, 40px)`
- [ ] 8.5 Update `--fs-h1` from `clamp(32px, 4vw, 48px)` to `clamp(36px, 4vw, 52px)`
- [ ] 8.6 Update `--fs-display` from `clamp(48px, 6vw, 76px)` to `clamp(52px, 6vw, 84px)`

## 9. Template: Typography Scale — Component Selectors

- [ ] 9.1 Update `.metric` from `clamp(36px, 5vw, 56px)` to `clamp(40px, 5vw, 60px)`
- [ ] 9.2 Update `.kpi-value` from `clamp(28px, 4vw, 44px)` to `clamp(32px, 4vw, 48px)`
- [ ] 9.3 Update `.hero-metric .metric` from `clamp(56px, 8vw, 96px)` to `clamp(60px, 8vw, 100px)`
- [ ] 9.4 Update `.sort-arrow` from `10px` to `18px`
- [ ] 9.5 Update `.scroll-indicator` from `24px` to `28px`

## 10. Build and Verification

- [ ] 10.1 `cargo test` — all tests pass
- [ ] 10.2 `cargo build --release` — successful compilation
- [ ] 10.3 Run the tool — verify output contains `lines` field (not `count`) in JSON
- [ ] 10.4 Verify Timeline bars reflect lines-changed values
- [ ] 10.5 Verify Type Breakdown bars reflect lines-changed values
- [ ] 10.6 Verify no text in the rendered HTML is below 18px
- [ ] 10.7 Screenshot key slides for visual verification
