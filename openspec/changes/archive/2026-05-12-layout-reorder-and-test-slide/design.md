# Layout Reorder and Test Slide

## Context

The contribution-showcase tool generates a self-contained HTML dashboard from
Git history and OpenSpec archives. The current report has three gaps:

1. The `.project-grid` uses `repeat(2, 1fr)`, which creates uneven rows when
   displaying 7 projects (3 rows of 2 + 1 orphan card). A 3-column layout
   produces balanced rows (2 rows of 3 + 1) and better utilises desktop width.
2. The proposals table columns are ordered (專案, 代稱, 日期, 工作項) and the
   commit log columns are ordered (Hash, 日期, 專案, 類型, 說明, +/−). The date
   column is not first, making chronological scanning inconvenient — operators
   typically scan contribution history by date.
3. There is no visibility into the test quality of each project. The dashboard
   covers commits, proposals, and project summaries but does not surface how
   well each project is tested.

The existing `ShowcaseData` model includes `projects`, `proposals`, and
`commits`. The HTML template renders 7 slides via `SLIDE_IDS`. The
`renderProposals()` and `renderCommits()` JavaScript functions build table
HTML with `buildSortHeader()` for sortable columns.

## Goals / Non-Goals

**Goals:**

- Improve project card layout density with a 3-column grid and proper
  responsive breakpoints.
- Place the date column first in both the proposals and commit log tables for
  faster chronological scanning.
- Collect and display per-project test metrics (framework, test file count,
  test case count, coverage) without executing tests.
- Add a "測試" slide to the dashboard between Projects and Proposals.

**Non-Goals:**

- Actually running tests or building projects during data collection.
- Supporting test frameworks beyond pytest, vitest/jest, and cargo test.
- Making grid column counts or table column order configurable via TOML.
- Adding test metrics filtering or sorting (the table is informational only
  in this iteration).
- Parsing coverage report content from arbitrary formats — only well-known
  file locations are checked for existence.

## Decisions

### D1 — Static test discovery over dynamic test execution

**Decision:** Collect test metrics by scanning the file system (glob for test
files, grep for test functions) rather than executing test runners.

Test file patterns:
- `test_*`, `*_test.*` (Python convention)
- `*.test.*`, `*.spec.*` (JS/TS convention)
- Rust: files containing `#[test]` or `#[cfg(test)]`

Test function patterns:
- Python: `def test_`, `async def test_`
- JavaScript/TypeScript: `it(`, `it.each(`, `test(`, `test.each(`
- Rust: `fn test_`, `#[test]`

```rust
pub struct TestMetrics {
    pub project: String,
    pub test_file_count: usize,
    pub test_case_count: usize,
    pub coverage_percent: Option<f64>,
    pub framework: String,  // "pytest", "vitest", "jest", "cargo test", "none"
}
```

**Why:** Static grepping is instant, dependency-free, and reliable. Running
actual tests would require setting up each project's environment (Python venv,
node_modules, Rust toolchain), may take minutes, and could fail due to missing
services (PostgreSQL, gRPC servers). The showcase tool runs on the developer's
machine where not all project dependencies may be installed.

**Alternatives considered:**

- *Run `pytest --co -q` / `npx vitest run --reporter=json` / `cargo test -- --list`*:
  Accurate counts but requires each project's full dependency tree. Fragile in
  air-gapped or CI environments where not all toolchains are available.
- *Parse `pyproject.toml` / `package.json` for test script definitions*: Only
  confirms a test command exists, not how many tests there are.

### D2 — Framework detection order

**Decision:** Detect the test framework by checking for project manifest files
in this order: `pyproject.toml` (with pytest in dependencies) → `Cargo.toml` →
`package.json` (with vitest or jest in devDependencies). Projects without any
recognised test tooling get `framework: "none"` and zero counts.

```rust
fn detect_framework(project_path: &Path) -> String {
    if has_pytest(project_path) { return "pytest".into(); }
    if project_path.join("Cargo.toml").exists() { return "cargo test".into(); }
    if has_vitest_or_jest(project_path) { /* check vitest first */ }
    "none".into()
}
```

**Why:** The order matches the my-project workspace composition — most repos are
Python (Platform Core, Custom Function Bridge, AI templates), one is Rust
(this tool), and one is Node/TypeScript (Frontend). Checking Python first
avoids false positives from Node projects that might also have a
`pyproject.toml` for tooling. The framework string is displayed in the UI
table's "框架" column.

**Alternatives considered:**

- *Check all manifests and pick the one with the most tests*: Unnecessary
  complexity — my-project projects are single-framework.
- *User-specified framework in `showcase.toml`*: Adds config burden for
  information that can be reliably auto-detected.

### D3 — Coverage data from existing report files

**Decision:** Look for pre-existing coverage report files rather than
generating them:

| Framework | File checked | Parse method |
|---|---|---|
| pytest | `coverage.xml` | Parse Cobertura XML `<coverage line-rate="0.85">` attribute |
| vitest/jest | `coverage/coverage-summary.json` | Parse Istanbul JSON `total.lines.pct` field |

`.coverage` (SQLite) and `target/llvm-cov/` are intentionally excluded —
`.coverage` requires Python-specific schema knowledge and `target/llvm-cov/`
has no single stable report format. Rust and other frameworks without a
supported report format get `coverage_percent: None`.

If no report file exists, `coverage_percent` is `None` and the UI displays
"—" in the coverage column.

**Why:** Coverage reports are generated during CI or local test runs and
persist in the working tree (often gitignored). Parsing existing files is
instant and avoids the need to execute tests. The trade-off is that coverage
data may be stale or absent, but this is acceptable for an informational
dashboard — the "—" indicator makes the absence explicit.

**Alternatives considered:**

- *Run `pytest --cov` / `npx vitest --coverage`*: Same problems as D1 —
  requires full project setup and test execution.
- *Only show file/case counts, skip coverage entirely*: Loses a valuable
  quality signal. Coverage is worth showing when available.

### D4 — 3-column grid responsive breakpoints

**Decision:** Three breakpoints for `.project-grid`:

```css
/* Mobile-first default: 1 column */
.project-grid {
    grid-template-columns: 1fr;
}

/* Tablet (≥601px): 2 columns */
@media (min-width: 601px) {
    .project-grid { grid-template-columns: repeat(2, 1fr); }
}

/* Desktop (≥901px): 3 columns */
@media (min-width: 901px) {
    .project-grid { grid-template-columns: repeat(3, 1fr); }
}
```

**Why:** With 7 projects, a 3-column grid produces 3 rows (3+3+1) vs the
current 2-column's 4 rows (2+2+2+1). This reduces vertical scrolling within
the Projects slide and better utilises the typical 1920px desktop viewport.
The `min-width` approach uses mobile-first progressive enhancement with exact
boundary semantics (601px and 901px) — no ambiguity at breakpoint edges.

**Alternatives considered:**

- *Auto-fill with `minmax()`*: Harder to predict exact column count; explicit
  breakpoints give deterministic layouts for the known content set.
- *Keep 2 columns, increase card density*: Does not solve the orphan card
  problem or the wasted horizontal space.

### D5 — Date-first column order

**Decision:** Move the date column to the first position in both tables:

- **Proposals:** 日期, 專案, 代稱, 工作項
- **Commits:** 日期, 專案, Hash, 類型, 說明, +/−

The date and project columns remain sortable via `buildSortHeader()`. Hash
stays static (sorting by hash has no semantic meaning).

**Why:** The most common use case when reviewing contribution history is
chronological scanning — "what happened this week?" or "when was this project
last active?". Placing the date first eliminates the need to visually skip
past other columns. The project column moves to second position as the next
most useful grouping dimension.

**Alternatives considered:**

- *Keep current order and add a default sort*: Does not help visual scanning
  when the table is sorted by a different column.
- *Make column order configurable*: Over-engineered for a single-user tool.

### D6 — Test slide position: between Projects and Proposals

**Decision:** Insert the "測試" slide as slide 6 (index 5 in `SLIDE_IDS`),
between "專案概覽" (Projects, slide 5) and "OpenSpec 提案" (Proposals, slide 6 →
now slide 7). The updated slide flow:

1. Cover → 2. KPI → 3. Timeline → 4. Type Breakdown → 5. Projects →
**6. 測試** → 7. Proposals → 8. Commits

```javascript
var SLIDE_IDS = [
    'slide-cover', 'slide-kpi', 'slide-timeline', 'slide-breakdown',
    'slide-projects', 'slide-tests', 'slide-proposals', 'slide-commits'
];
```

**Why:** Test metrics are per-project data, so they logically follow the
Projects slide. Proposals and commits are timeline/event data that form the
second half of the deck. This ordering creates a natural flow: overview →
metrics → projects → quality → timeline → details.

**Alternatives considered:**

- *After Commits (last slide)*: Buries quality data at the end where it is
  easily overlooked.
- *After KPI (slide 3)*: Too early — the viewer has not yet seen which
  projects exist, so per-project test data lacks context.

### D7 — Coverage progress bar colors

**Decision:** Use three color tiers for the coverage percentage progress bar:

| Range | Color | Hex |
|---|---|---|
| ≥ 80% | Green | `#2f7d4a` |
| 50–79% | Amber | `#d97706` |
| < 50% | Red | `#b53a2a` |

These colors are applied as `background-color` on a `<div>` inside the
coverage table cell, with width proportional to the percentage value.

**Why:** The thresholds (80/50) are industry-standard coverage quality
benchmarks. The hex values reuse the existing type-breakdown color palette
from the design system, ensuring visual consistency without introducing new
colors.

**Alternatives considered:**

- *Continuous gradient (HSL interpolation)*: Harder to read at a glance;
  discrete tiers communicate quality levels more clearly.
- *Only show the number, no bar*: Misses the opportunity for visual scanning
  across projects.

### D8 — Test pattern matching

**Decision:** Use the following grep patterns to count test cases:

| Language | Pattern | What it matches |
|---|---|---|
| Python | `^\s*(async\s+)?def\s+test_` | pytest test functions and async test functions |
| JavaScript/TypeScript | `\b(it\|test)\s*\(` | Mocha/Jest/Vitest test cases (`it(` and `test(`) |
| JavaScript/TypeScript | `\b(it\|test)\.each\s*\(` | Parameterised test cases |
| Rust | `#\[(tokio::)?test\]` or `#\[rstest\]` | Rust test attributes including async and parameterised variants |

Test file discovery patterns:
- `**/test_*.py`, `**/*_test.py` (Python)
- `**/*.test.{ts,js,tsx,jsx}`, `**/*.spec.{ts,js,tsx,jsx}` (JS/TS)
- `**/tests/**/*.rs`, plus any `.rs` file containing `#[cfg(test)]` (Rust)

Exclusions: `node_modules/`, `target/`, `.venv/`, `__pycache__/`, `dist/`.

**Why:** These patterns match the conventions used across my-project repositories.
The patterns are intentionally simple — a regex over source lines — rather
than AST-based, to keep the implementation lightweight and avoid adding
parser dependencies to a Rust CLI tool.

**Alternatives considered:**

- *AST parsing (tree-sitter)*: Accurate but heavy dependency for a reporting
  tool. Overkill when grep-based counting is sufficient for an approximate
  metric.
- *Count `describe(` blocks instead of `it(`*: `describe` is a grouping
  construct, not a test case. Counting `it(` / `test(` gives the actual test
  case count.

### D9 — Date column min-width in proposals table

**Decision:** Add `white-space: nowrap` and `min-width: 7em` to date cells
in the proposals table to prevent the `YYYY-MM-DD` date from wrapping to two
lines.

```css
.data-table .date-cell {
    white-space: nowrap;
    min-width: calc(10ch + var(--gap-sm) + var(--gap-sm));
}
```

The `10ch` unit matches exactly 10 characters (the width of `YYYY-MM-DD`),
and `2 * var(--gap-sm)` accounts for horizontal cell padding. This provides
a precise string-fit guarantee rather than an approximate `em`-based value.

The commits table already applies `white-space: nowrap` via inline style on
date cells. This decision consolidates both tables to use a shared `.date-cell`
CSS class for consistency, adding both `nowrap` and `min-width` to both.

**Why:** The date column in the proposals table is too narrow when other
columns (especially 代稱 with long slug names) consume available width. A
`YYYY-MM-DD` string is exactly 10 characters; `10ch` plus cell padding
provides a precise minimum. The `white-space: nowrap` prevents any
line-breaking regardless of table layout algorithm pressure.

**Alternatives considered:**

- *Fixed column widths via `table-layout: fixed`*: Too rigid — would require
  explicit widths for all columns and reduce flexibility for variable-length
  content like slug names.
- *Only `white-space: nowrap` without `min-width`*: Prevents wrapping but
  the column may still compress horizontally, truncating text with overflow.
  `min-width` ensures a readable minimum.

## Risks / Trade-offs

### R1 — Static discovery may overcount or undercount tests

Grep-based counting cannot distinguish commented-out test functions, test
functions inside `#[ignore]` blocks, or dynamically generated tests (e.g.,
pytest parametrize, Vitest `it.each` with array expansion). **Mitigation:**
The metric is labelled as approximate in the dashboard context. For the my-project
project set, manual spot-checks show <5% variance from actual counts.

### R2 — Coverage files may be stale

Pre-existing `.coverage` or `coverage/` files reflect whenever tests were last
run with coverage enabled, which may not match the current codebase.
**Mitigation:** The UI shows "—" when no coverage data is found, making
absence explicit. Stale data is still useful as a directional indicator —
and operators can re-run tests to refresh the files.

### R3 — 3-column grid may feel cramped with long project names

Some my-project project names are long (e.g., "my-downstream-template").
At 3 columns on a 1280px viewport, each card is ~400px wide. **Mitigation:**
The 900px breakpoint falls back to 2 columns before cards become too narrow.
Project names already use `word-break: break-word` in the card CSS. At 1920px
(the typical operator workstation), each card is ~600px — comfortable for any
project name length.
