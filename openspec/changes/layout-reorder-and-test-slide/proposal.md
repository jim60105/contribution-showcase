# layout-reorder-and-test-slide

## Why

The contribution showcase dashboard has three usability gaps: the project grid is cramped at 2 columns when displaying 7 projects, the proposals and commit log tables place the date column too far right for efficient chronological scanning, and there is no visibility into the test quality metrics of each project. Addressing all three in a single change avoids multiple layout-disrupting releases.

## What Changes

1. **Project cards → 3-column grid** — Change `.project-grid` to a mobile-first layout using `min-width` breakpoints: default 1 column, ≥601 px → 2 columns, ≥901 px → 3 columns.

2. **Reorder table columns (date-first)** — In the "OpenSpec 提案" table, reorder columns from (專案, 代稱, 日期, 工作項) to **(日期, 專案, 代稱, 工作項)**. In the "提交紀錄" table, reorder columns from (Hash, 日期, 專案, 類型, 說明, +/−) to **(日期, 專案, Hash, 類型, 說明, +/−)**. The date and project columns remain sortable; Hash stays static.

3. **Test metrics slide** — Add a new 8th slide "測試" between the Projects and Proposals slides. The slide displays a table of test metrics (框架, 測試檔案, 測試案例, 覆蓋率) per project, collected statically from each project's source tree without executing tests. Below the table, a KPI row shows totals: 總測試檔案, 總測試案例, 平均覆蓋率.

4. **Date column min-width** — The date column in the OpenSpec 提案 table wraps to two lines (e.g., "2026-05-\n08") when the table is wide enough for other columns but not the date. Add `white-space: nowrap` and `min-width` to the date cells in the proposals table so that the `YYYY-MM-DD` format always displays on a single line. The commits table date column already has `white-space: nowrap`.

## Capabilities

### New Capabilities

- `test-metrics-collection`: Static collection of test file counts, test case counts, framework detection, and coverage data from each project's source tree. No test execution — uses file pattern matching and grep-based counting.

### Modified Capabilities

- `html-report-generation`: Project grid layout changes to 3-column with a new 900 px breakpoint; proposals and commit log table column reorder (date-first); new "測試" slide (8th slide) with test metrics table and KPI summary cards.
- `toml-config-loader`: No config schema changes required — test metrics collection uses the existing `projects` list and their `path` fields.

## Impact

- `src/model.rs` — Add `TestMetrics` struct and `test_metrics: Vec<TestMetrics>` field to `ShowcaseData`
- `src/collector.rs` — New `collect_test_metrics()` function for static test discovery
- `templates/page.html` — 3-column grid CSS, table column reorder in `renderProposals()` and `renderCommits()`, new slide section and `renderTestMetrics()` function, updated `SLIDE_IDS` and `NAV_LABELS` arrays, date column `white-space: nowrap` and `min-width` in proposals table
- `src/main.rs` — Wire `collect_test_metrics()` into the data pipeline
