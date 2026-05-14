# HTML Report Generation

## Purpose

Renders the collected `ShowcaseData` into a single, self-contained HTML file
suitable for air-gapped environments. The output embeds all CSS and JavaScript
inline, and presents an "Editorial Paper" themed slide-based dashboard entirely
in Traditional Chinese (zh-TW). External dependencies are limited to Google
Fonts for typography.

## Requirements

### Requirement: JSON Serialization and Escaping

The system SHALL serialize `ShowcaseData` to JSON and apply HTML-safe escaping
before injecting the result into the HTML template. After JSON serialization,
the characters `<`, `>`, `&`, U+2028, and U+2029 SHALL be replaced with their
Unicode escape sequences (`\u003c`, `\u003e`, `\u0026`, `\u2028`, `\u2029`)
to prevent XSS when the JSON is embedded inside a `<script>` block.

#### Scenario: Data contains HTML-sensitive characters
- **GIVEN** a `ShowcaseData` instance where a commit message contains
  `<script>alert(1)</script>`
- **WHEN** the renderer serializes and escapes the data
- **THEN** the output JSON contains `\u003cscript\u003ealert(1)\u003c/script\u003e`
  instead of literal angle brackets

#### Scenario: Data contains Unicode line separators
- **GIVEN** a `ShowcaseData` instance containing U+2028 or U+2029 characters
- **WHEN** the renderer serializes and escapes the data
- **THEN** those characters are replaced with `\u2028` and `\u2029` respectively

### Requirement: Template Injection

The system SHALL load the HTML template at compile time via
`include_str!("../templates/page.html")` and replace the placeholder
`"__SHOWCASE_DATA__"` with the escaped JSON string.

#### Scenario: Template contains the placeholder
- **GIVEN** the compiled-in HTML template contains the string
  `"__SHOWCASE_DATA__"`
- **WHEN** the renderer injects data
- **THEN** the placeholder is replaced with the escaped JSON payload exactly
  once

### Requirement: Self-Contained Output

The system SHALL produce a single HTML file with all CSS and JavaScript
inlined. The output SHALL NOT reference any external resources aside from
Google Fonts loaded via `<link>` tags — no other CDN links, no network
requests beyond font loading.

#### Scenario: Output file opened in an air-gapped browser
- **GIVEN** the generated HTML file
- **WHEN** it is opened in a browser with no network connectivity
- **THEN** the dashboard renders fully (with fallback fonts) without errors or
  missing resources

### Requirement: Editorial Paper Design System

The system SHALL apply the "Editorial Paper" visual theme with warm paper
background `#f8f7f4`, surface color `#ffffff`, foreground `#1a1916`, muted
text `#6b6964`, border `#e8e5df`, and accent `#2563eb`. The theme uses serif
fonts for display headings and sans-serif for body text.

#### Scenario: Theme colours applied
- **GIVEN** the generated HTML file
- **WHEN** inspecting the CSS `:root` custom properties
- **THEN** `--bg` is `#f8f7f4`, `--surface` is `#ffffff`, `--fg` is
  `#1a1916`, `--muted` is `#6b6964`, `--border` is `#e8e5df`, and
  `--accent` is `#2563eb`

### Requirement: Slide-Based Section Layout

The system SHALL render dashboard sections as full-viewport slides using CSS
scroll-snap, replacing the previous continuous scroll layout.

#### Scenario: Dashboard sections render as slides
- **GIVEN** the generated HTML file
- **WHEN** opened in a desktop browser
- **THEN** each section occupies the full viewport height and the browser
  snaps between sections on scroll

### Requirement: 8-Slide Dashboard Flow

The system SHALL render 8 dashboard slides in order: Cover (hero), KPI Dashboard (6 metric cards), Timeline (weekly bar chart), Type Breakdown, Project Cards, Test Metrics, Proposals Table, and Commit Log.

#### Scenario: All slides present
- **GIVEN** a `ShowcaseData` instance with commits, proposals, projects, and test metrics
- **WHEN** the HTML report is generated
- **THEN** the output contains 8 slide sections in the specified order

### Requirement: KPI Dashboard with 6 Metrics

The system SHALL display 6 KPI metric cards: Total Commits, Active
Repositories, OpenSpec Proposals, Lines Added, Lines Removed, and Average
Daily Lines Changed.

#### Scenario: KPI cards rendered
- **GIVEN** a `ShowcaseData` instance
- **WHEN** the HTML report is generated
- **THEN** the KPI section contains 6 metric cards including one for average
  daily lines changed

### Requirement: Typography Craft System

The system SHALL use a structured typography system with Google Fonts:
serif display (`'Playfair Display', 'Noto Serif TC', Georgia, serif`),
sans-serif body (`'Inter', 'Noto Sans TC', system-ui, sans-serif`), and
monospace for data numbers (`'JetBrains Mono'`,
`font-variant-numeric: tabular-nums`). Headlines use negative letter-spacing
and a multi-weight hierarchy (400 body / 500–600 labels / 700 headlines).
Font loading uses `<link rel="preconnect">` and `font-display: swap`.

The report SHALL enforce a minimum font size floor of 18px across all text
elements, with a proportionally scaled hierarchy.

The CSS custom properties SHALL be defined as follows:

| Variable | Value |
|---|---|
| `--fs-xs` | 18px |
| `--fs-small` | 20px |
| `--fs-body` | 22px |
| `--fs-h2` | clamp(28px, 3vw, 40px) |
| `--fs-h1` | clamp(36px, 4vw, 52px) |
| `--fs-display` | clamp(52px, 6vw, 84px) |

The following component-level font sizes SHALL apply:

| Selector | Value |
|---|---|
| `.metric` | clamp(40px, 5vw, 60px) |
| `.kpi-value` | clamp(32px, 4vw, 48px) |
| `.hero-metric .metric` | clamp(60px, 8vw, 100px) |
| `.sort-arrow` | 18px |
| `.scroll-indicator` | 28px |

#### Scenario: Typography applied
- **GIVEN** the generated HTML file
- **WHEN** inspecting CSS
- **THEN** display headings use Playfair Display/Noto Serif TC with negative
  tracking, body text uses Inter/Noto Sans TC, and numeric values use
  JetBrains Mono with tabular-nums

#### Scenario: No text element renders below 18px

Given the report is rendered at any viewport width,
then no visible text element SHALL have a computed font-size below 18px.

#### Scenario: Proportional hierarchy is preserved

Given the typography scale,
then `--fs-xs` < `--fs-small` < `--fs-body` < `--fs-h2` (min) < `--fs-h1` (min) < `--fs-display` (min) MUST hold.

### Requirement: Lines-Based Timeline Chart

The Timeline chart JavaScript SHALL read `entry.lines` (total lines changed)
for value annotations and peak identification, while continuing to use
`entry.height` (backend-normalized 0–100) for bar rendering dimensions.

The peak annotation label SHALL use the format:

```
高峰：{label}（{lines.toLocaleString('zh-TW')}行）
```

#### Scenario: Timeline chart renders lines changed

Given a timeline data entry with `lines: 1500`, `height: 100`, and `label: "2024-W03"`,
when the timeline chart is rendered,
then the bar height MUST be proportional to `entry.height`
and the peak annotation SHALL read "高峰：2024-W03（1,500行）".

### Requirement: Lines-Based Type Breakdown Chart

The Type Breakdown chart JavaScript SHALL read `entry.lines` (total lines
changed) for value labels, while continuing to use `entry.percentage`
(backend-normalized) for bar width rendering.

The breakdown label SHALL use the format:

```
{label}（{lines.toLocaleString('zh-TW')}行）
```

#### Scenario: Type breakdown chart renders lines changed

Given a type breakdown entry with `label: "feat"`, `lines: 12345`, and `percentage: 45.2`,
when the breakdown chart is rendered,
then the bar width MUST be proportional to `entry.percentage`
and the label SHALL read "feat（12,345行）".

#### Scenario: Breakdown unit consistency

Given the type breakdown chart is rendered,
then all segment labels SHALL use "行" as the unit suffix
and values MUST be formatted with `toLocaleString('zh-TW')`.

### Requirement: Empty Data State

When a slide's data set is empty (zero commits, zero proposals, zero timeline
entries), the slide SHALL still render with a centred message indicating no
data is available (e.g. "無提交記錄", "無提案紀錄"). Slides are never hidden.

#### Scenario: No commits after filtering
- **GIVEN** a `ShowcaseData` instance with zero commits
- **WHEN** the HTML report is generated
- **THEN** the Timeline, Type Breakdown, and Commit Log slides each display
  an empty-state message instead of charts or tables

### Requirement: Commit Log Pagination

The system SHALL initially display 100 rows in the commit log section and
provide a toggle to reveal all remaining rows.

#### Scenario: Commit log exceeds 100 entries
- **GIVEN** a `ShowcaseData` instance with 250 commits
- **WHEN** the HTML report is opened
- **THEN** the commit log displays the first 100 rows with a toggle control
  to show all 250

#### Scenario: Commit log has fewer than 100 entries
- **GIVEN** a `ShowcaseData` instance with 42 commits
- **WHEN** the HTML report is opened
- **THEN** all 42 rows are displayed and no toggle control is shown

### Requirement: Output File Creation

The system SHALL write the generated HTML to the specified output path,
creating parent directories automatically if they do not exist.

#### Scenario: Output path parent directory does not exist
- **GIVEN** an output path of `reports/2025/showcase.html` where the
  `reports/2025/` directory does not yet exist
- **WHEN** the renderer writes the file
- **THEN** the parent directories are created and the file is written
  successfully

### Requirement: Traditional Chinese UI

The system SHALL render all user-facing labels, headings, and descriptions in
Traditional Chinese (zh-TW).

#### Scenario: Dashboard language
- **GIVEN** the generated HTML file
- **WHEN** reading section headings and labels
- **THEN** all UI text is in Traditional Chinese

### Requirement: Commit Table Column Order

The commit log table columns SHALL be ordered: 日期, 專案, Hash, 類型, 說明, +/−. The Hash column SHALL display the first 8 characters of the commit hash rendered in a monospace font (`font-family: var(--font-mono)`).

#### Scenario: Commit table column order
- **GIVEN** the commit log table is rendered
- **WHEN** viewing the table headers
- **THEN** the column order is 日期, 專案, Hash, 類型, 說明, +/−

#### Scenario: Commit hash displayed as short hash
- **GIVEN** a commit with hash `"dd33ee63950bb49a284de835528343561f1a70d5"`
- **WHEN** the commit log table is rendered
- **THEN** the Hash column of that row displays `"dd33ee63"` in monospace font

### Requirement: Emoji Favicon

The HTML page SHALL include a favicon `<link>` tag in `<head>` using a percent-encoded inline SVG data URI containing the 📊 (bar chart) emoji. This provides a tab icon appropriate for a data/contribution showcase without requiring an external favicon file.

#### Scenario: Page loaded in browser
- **GIVEN** the generated HTML file
- **WHEN** the page is loaded in a browser
- **THEN** the browser tab displays the 📊 emoji as the favicon

### Requirement: Proposal Table Layout

The OpenSpec proposals table columns SHALL be ordered: 日期, 專案, 代稱, 工作項.

#### Scenario: Proposals table column order
- **GIVEN** the proposals table is rendered
- **WHEN** viewing the table headers
- **THEN** the column order is 日期, 專案, 代稱, 工作項

### Requirement: Project Cards Grid Layout

The project cards grid SHALL use a 3-column layout at desktop widths (≥901px), 2 columns at tablet widths (601–900px), and 1 column at mobile widths (≤600px). The CSS SHALL use a mobile-first `min-width` approach. The project cards section SHALL be wrapped in a scroll container (`.scroll-container`) to enable vertical scrolling when content exceeds viewport height, matching the behavior of the commit log, proposal, and test metrics sections. The section SHALL use the `slide--scrollable` class.

#### Scenario: Desktop viewport
- **GIVEN** a viewport width ≥901px
- **WHEN** the project cards section is rendered
- **THEN** the grid displays 3 columns

#### Scenario: Tablet viewport
- **GIVEN** a viewport width between 601px and 900px
- **WHEN** the project cards section is rendered
- **THEN** the grid displays 2 columns

#### Scenario: Mobile viewport
- **GIVEN** a viewport width ≤600px
- **WHEN** the project cards section is rendered
- **THEN** the grid displays 1 column

#### Scenario: Project cards scroll container
- **GIVEN** more project cards than fit within the viewport height
- **WHEN** the project cards section is rendered
- **THEN** the `#projectGrid` element SHALL be wrapped in a `.scroll-container` div that enables vertical scrolling

#### Scenario: Project slide uses scrollable class
- **GIVEN** the project cards slide is rendered
- **WHEN** inspecting the slide element's CSS classes
- **THEN** the slide element SHALL include the `slide--scrollable` class

### Requirement: Date Column Width in Data Tables

The date cells in both the proposals table and the commit log table SHALL apply `white-space: nowrap` and `min-width: calc(10ch + var(--gap-sm) + var(--gap-sm))` via a shared `.date-cell` CSS class, so that `YYYY-MM-DD` dates always render on a single line without wrapping.

#### Scenario: Proposals date displays on one line
- **GIVEN** a proposals table with a date value "2026-05-08"
- **WHEN** the table is rendered at any supported viewport width
- **THEN** the date displays on a single line without wrapping

#### Scenario: Commits date displays on one line
- **GIVEN** a commit log table with a date value "2026-05-08"
- **WHEN** the table is rendered at any supported viewport width
- **THEN** the date displays on a single line without wrapping

#### Scenario: Date column minimum width
- **GIVEN** either the proposals or commit log table is rendered
- **WHEN** the table layout algorithm distributes column widths
- **THEN** the date column is at least `calc(10ch + var(--gap-sm) + var(--gap-sm))` wide

### Requirement: Test Metrics Slide

The dashboard SHALL include a "測試" slide displaying test metrics for each project. The slide SHALL contain a KPI card row and a detail table. The KPI card row SHALL render ABOVE the detail table, showing totals: 總測試檔案 (sum of all test_file_count), 總測試案例 (sum of all test_case_count), 平均覆蓋率 (arithmetic mean of coverage_percent values from projects where coverage is not null; display "—" if no project has coverage data). Below the KPI card row, the detail table SHALL display columns: 專案, 框架, 測試檔案, 測試案例, 覆蓋率.

#### Scenario: KPI row renders above the detail table
- **GIVEN** test metrics data for multiple projects
- **WHEN** the test metrics slide is rendered
- **THEN** the KPI card row (總測試檔案, 總測試案例, 平均覆蓋率) SHALL appear above the per-project detail table in the DOM order

#### Scenario: Test metrics table rendered
- **GIVEN** test metrics data for 3 projects with varying coverage
- **WHEN** the test metrics slide is rendered
- **THEN** the table displays one row per project with framework, file count, case count, and coverage percentage

#### Scenario: Coverage progress bar colors
- **GIVEN** a project with coverage ≥80%
- **WHEN** the coverage cell is rendered
- **THEN** the progress bar uses the green color (#2f7d4a)

#### Scenario: Coverage unavailable
- **GIVEN** a project with no coverage data (coverage_percent is null)
- **WHEN** the coverage cell is rendered
- **THEN** the cell displays "—" with no progress bar

#### Scenario: No test metrics
- **GIVEN** zero test metrics entries
- **WHEN** the test metrics slide is rendered
- **THEN** the slide displays an empty-state message "無測試資料"

#### Scenario: Framework detected but zero test files
- **GIVEN** a project with framework "pytest" but no files matching test patterns
- **WHEN** the project row is rendered
- **THEN** test_file_count and test_case_count display 0

#### Scenario: Test files found but zero test cases
- **GIVEN** a project with 3 test files but no lines matching test function patterns
- **WHEN** the project row is rendered
- **THEN** test_file_count displays 3 and test_case_count displays 0

### Requirement: Conditional Project Name Linking

When rendering project cards, the HTML template SHALL conditionally render the project name as a hyperlink or plain text based on the presence of a `url` field in the project data. If the project's `url` field is a non-null string, the project name SHALL be rendered as an `<a>` element with `href` set to the URL, `target="_blank"`, and `rel="noopener noreferrer"`, placed inside the `<h3 class="h2">` heading. If the `url` field is `null` or absent, the project name SHALL be rendered as plain escaped text inside the `<h3 class="h2">` heading, preserving the existing behavior.

#### Scenario: Project card with URL renders hyperlink
- **GIVEN** a project entry in the JSON payload has `"url": "https://github.com/example/repo"`
- **WHEN** the project card is rendered
- **THEN** the `<h3 class="h2">` heading contains `<a href="https://github.com/example/repo" target="_blank" rel="noopener noreferrer">` wrapping the escaped project name

#### Scenario: Project card without URL renders plain text
- **GIVEN** a project entry in the JSON payload has `"url": null`
- **WHEN** the project card is rendered
- **THEN** the `<h3 class="h2">` heading contains the escaped project name as plain text without any `<a>` tag

#### Scenario: Hyperlink styling is consistent with card design
- **GIVEN** a project card renders the project name as a hyperlink
- **WHEN** the card is displayed
- **THEN** the hyperlink inherits the heading text color and does not introduce unexpected visual changes to the card layout
