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

### Requirement: 7-Slide Dashboard Flow

The system SHALL render the following 7 dashboard slides in order: Cover
(hero), KPI Dashboard (6 metric cards), Timeline (weekly bar chart), Type
Breakdown, Project Cards, Proposals Table, and Commit Log.

#### Scenario: All slides present
- **GIVEN** a `ShowcaseData` instance with commits, proposals, and projects
- **WHEN** the HTML report is generated
- **THEN** the output contains 7 slide sections: Cover, KPI Dashboard,
  Timeline, Type Breakdown, Projects, Proposals, and Commit Log

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

#### Scenario: Typography applied
- **GIVEN** the generated HTML file
- **WHEN** inspecting CSS
- **THEN** display headings use Playfair Display/Noto Serif TC with negative
  tracking, body text uses Inter/Noto Sans TC, and numeric values use
  JetBrains Mono with tabular-nums

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
