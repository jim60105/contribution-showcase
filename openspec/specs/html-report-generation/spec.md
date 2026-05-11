# HTML Report Generation

## Purpose

Renders the collected `ShowcaseData` into a single, self-contained HTML file
suitable for air-gapped environments. The output embeds all CSS and JavaScript
inline, requires no external resources, and presents a "Soft Paper" themed
dashboard entirely in Traditional Chinese (zh-TW).

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
inlined. The output SHALL NOT reference any external resources — no CDN links,
no external fonts, no network requests.

#### Scenario: Output file opened in an air-gapped browser
- **GIVEN** the generated HTML file
- **WHEN** it is opened in a browser with no network connectivity
- **THEN** the dashboard renders fully without errors or missing resources

### Requirement: Dashboard Sections

The system SHALL render the following dashboard sections: Hero banner, KPI
strip (5 cards), timeline bar chart (weekly aggregation), type breakdown,
project cards, proposals table, and commit log.

#### Scenario: All data sections populated
- **GIVEN** a `ShowcaseData` instance with commits, proposals, and project
  information
- **WHEN** the HTML report is generated
- **THEN** the output contains all dashboard sections: Hero, KPI strip,
  timeline chart, type breakdown, project cards, proposals table, and commit
  log

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

### Requirement: Soft Paper Theme

The system SHALL apply the "Soft Paper" visual theme with canvas color
`#f2f2f0`, surface color `#fff`, and ink color `#0a0a0a`.

#### Scenario: Theme colors applied
- **GIVEN** the generated HTML file
- **WHEN** inspecting the CSS styles
- **THEN** the background uses `#f2f2f0`, card/surface backgrounds use
  `#fff`, and primary text uses `#0a0a0a`

### Requirement: Traditional Chinese UI

The system SHALL render all user-facing labels, headings, and descriptions in
Traditional Chinese (zh-TW).

#### Scenario: Dashboard language
- **GIVEN** the generated HTML file
- **WHEN** reading section headings and labels
- **THEN** all UI text is in Traditional Chinese
