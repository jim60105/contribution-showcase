## 1. Data Model

- [x] 1.1 Add `type_lines: BTreeMap<String, usize>` field to `TimelineEntry` in `src/model.rs` and add `use std::collections::BTreeMap` import

## 2. Collector — `build_timeline()` Updates

- [x] 2.1 Add `bucket_type_lines: HashMap<String, BTreeMap<String, usize>>` accumulator alongside existing `bucket_lines` in `build_timeline()` in `src/collector.rs`
- [x] 2.2 During commit iteration, canonicalize unknown commit types to "other" (matching `build_type_breakdown()`) and accumulate `insertions + deletions` into `bucket_type_lines[bucket_key][canonical_type]` for each commit; skip entries where lines == 0
- [x] 2.3 When building the final `Vec<TimelineEntry>`, populate `type_lines` from `bucket_type_lines` (default to empty `BTreeMap` for gap buckets with no commits)
- [x] 2.4 Update all existing `TimelineEntry` construction sites in tests to include `type_lines: BTreeMap::new()` (or appropriate values)

## 3. Collector Tests

- [x] 3.1 Add test: bucket with multiple commit types records correct per-type lines and values sum to `lines`
- [x] 3.2 Add test: bucket with single commit type produces single-entry `type_lines` map
- [x] 3.3 Add test: zero-line types are omitted from `type_lines` map
- [x] 3.4 Add test: contiguous gap bucket has empty `type_lines` map

## 4. HTML Template — Section Rename

- [x] 4.1 Change the `<h2>` heading in `slide-timeline` section from "時間軸" to "提交趨勢" in `templates/page.html`
- [x] 4.2 Change the `NAV_LABELS` array entry from "時間軸" to "提交趨勢" in `templates/page.html`

## 5. HTML Template — Stacked Bar Rendering

- [x] 5.1 Rewrite `renderTimeline()` in `templates/page.html` to compute a global type order (descending by total lines across all buckets, `other` always last)
- [x] 5.2 Render each bar as a flex column with one `<div>` segment per type in `type_lines`, colored with `var(--<type>)` and height proportional to its share of the bucket total
- [x] 5.3 Preserve existing peak annotation logic (`高峰：{label}（{lines}行）`)
- [x] 5.4 Add CSS for stacked bar segments (flex column layout, segment styling)
- [x] 5.5 Ensure empty timeline state still renders "無提交記錄"

## 6. Integration Tests

- [x] 6.1 Add test verifying "提交趨勢" heading appears in generated HTML (replaces "時間軸")
- [x] 6.2 Add test verifying `type_lines` field appears in embedded JSON data for generated output
- [x] 6.3 Run full test suite (`cargo test`) to verify all 186+ tests pass

## 7. Regenerate Docs

- [x] 7.1 Run `cargo run -- generate` and copy `dist/index.html` to `docs/contribution-showcase.html`
