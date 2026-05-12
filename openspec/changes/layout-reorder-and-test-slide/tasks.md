# Tasks: layout-reorder-and-test-slide

Four changes: project grid 3-column, table column reorder, test metrics slide, and date column min-width.

---

## 1. Template: Project Grid 3-Column

- [ ] 1.1 In `templates/page.html`, change `.project-grid` to mobile-first: default `1fr`, `@media (min-width: 601px)` → `repeat(2, 1fr)`, `@media (min-width: 901px)` → `repeat(3, 1fr)`
- [ ] 1.2 Remove the existing `max-width: 600px` breakpoint for `.project-grid` (replaced by mobile-first approach)
- [ ] 1.3 Verify the grid renders 3 columns on desktop, 2 on tablet, 1 on mobile

## 2. Template: Proposal Table Column Reorder

- [ ] 2.1 In `renderProposals()` in `templates/page.html`, change header order from `<th>專案</th><th>代稱</th><th>日期</th><th>工作項</th>` to `<th>日期</th><th>專案</th><th>代稱</th><th>工作項</th>`
- [ ] 2.2 Reorder the row `<td>` cells to match: date first, then project, then slug, then task_count
- [ ] 2.3 Add a `.date-cell` CSS class with `white-space: nowrap` and `min-width: calc(10ch + 2 * var(--gap-sm))`. Apply it to the date `<td>` in `renderProposals()` so that `YYYY-MM-DD` dates never wrap to two lines
- [ ] 2.4 Also apply the `.date-cell` class to the date `<td>` in `renderCommits()`, replacing the existing inline `white-space: nowrap` style for consistency

## 3. Template: Commit Table Column Reorder

- [ ] 3.1 In `renderCommits()` in `templates/page.html`, reorder headers to: 日期, 專案, Hash, 類型, 說明, +/− (move Hash from first to third position, after 專案)
- [ ] 3.2 Reorder the row `<td>` cells to match the new header order
- [ ] 3.3 Verify that `buildSortHeader()` sort behavior works correctly after column reorder

## 4. Model: Test Metrics Struct

- [ ] 4.1 Add `TestMetrics` struct to `src/model.rs` with fields: `project: String`, `test_file_count: usize`, `test_case_count: usize`, `coverage_percent: Option<f64>`, `framework: String`
- [ ] 4.2 Add `test_metrics: Vec<TestMetrics>` field to `ShowcaseData`

## 5. Backend: Test Metrics Collection

- [ ] 5.1 Add `collect_test_metrics()` function in `src/collector.rs` that iterates over project configs and collects test metrics for each
- [ ] 5.2 Implement framework detection: check for `pyproject.toml` containing "pytest" → `Cargo.toml` → `package.json` containing "vitest" or "jest"; return "none" if undetected
- [ ] 5.3 Implement test file discovery: count files matching `test_*`, `*_test.*`, `*.test.*`, `*.spec.*` patterns, excluding `node_modules/`, `target/`, `.venv/`, `__pycache__/`, `.git/` directories
- [ ] 5.4 Implement test case counting: grep test files for framework-specific patterns (`def test_`, `async def test_`, `fn test_`, `#[test]`, `it(`, `test(`)
- [ ] 5.5 Implement coverage report discovery: parse `coverage.xml` (Cobertura, `line-rate` attribute) and `coverage/coverage-summary.json` (Istanbul, `total.lines.pct`); return `None` if no supported report found. Skip `.coverage` SQLite and `target/llvm-cov/`.
- [ ] 5.6 Wire `collect_test_metrics()` into `collect_all()` to populate `ShowcaseData.test_metrics`

## 6. Template: Test Metrics Slide

- [ ] 6.1 Add a new slide section `<section class="slide slide--scrollable" id="slide-tests">` between `slide-projects` and `slide-proposals` with title "測試"
- [ ] 6.2 Add CSS for test coverage progress bar: `.coverage-bar` container with background track and filled bar, colored by threshold (green ≥80%, amber 50-79%, red <50%)
- [ ] 6.3 Implement `renderTestMetrics()` JavaScript function that renders a table with columns: 專案, 框架, 測試檔案, 測試案例, 覆蓋率
- [ ] 6.4 In the coverage column, render a progress bar with percentage label; show "—" when coverage is null
- [ ] 6.5 Below the table, add KPI card row with totals: 總測試檔案 (sum), 總測試案例 (sum), 平均覆蓋率 (mean of projects where coverage_percent is not null; "—" if none have coverage)
- [ ] 6.6 Handle empty state: display "無測試資料" when no test metrics exist
- [ ] 6.7 Update `NAV_LABELS` and `SLIDE_IDS` arrays to include the new test slide (insert between '專案' and '提案')

## 7. Unit Tests

- [ ] 7.1 Add test for framework detection: Python project with pytest dependency detected as "pytest"
- [ ] 7.2 Add test for framework detection: project with no test config detected as "none"
- [ ] 7.3 Add test for test file discovery: counts files matching test patterns correctly
- [ ] 7.4 Add test for test case counting: counts `def test_*` patterns in Python test files
- [ ] 7.5 Add test for test case counting: counts `it(` and `test(` patterns in JS/TS test files
- [ ] 7.6 Add test for coverage discovery: returns None when no coverage report exists

## 8. Build and Verification

- [ ] 8.1 Run `cargo test` — all existing and new tests pass
- [ ] 8.2 Run `cargo build --release` — no warnings, no errors
- [ ] 8.3 Run the tool and verify: project grid shows 3 columns, table columns reordered, test slide displays metrics
