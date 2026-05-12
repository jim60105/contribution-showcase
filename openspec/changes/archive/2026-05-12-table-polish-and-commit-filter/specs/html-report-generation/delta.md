# HTML Report Generation — Delta

## ADDED Requirements

### Requirement: Commit Hash Column

The commit log table SHALL include a "Hash" column as the first column. The column SHALL display the first 8 characters of the commit hash rendered in a monospace font (`font-family: var(--font-mono)`).

#### Scenario: Commit hash displayed as short hash

- **GIVEN** a commit with hash `"dd33ee63950bb49a284de835528343561f1a70d5"`
- **WHEN** the commit log table is rendered
- **THEN** the first column of that row displays `"dd33ee63"` in monospace font

### Requirement: Emoji Favicon

The HTML page SHALL include a favicon `<link>` tag in `<head>` using a percent-encoded inline SVG data URI containing the 📊 (bar chart) emoji. This provides a tab icon appropriate for a data/contribution showcase without requiring an external favicon file.

#### Scenario: Page loaded in browser

- **GIVEN** the generated HTML file
- **WHEN** the page is loaded in a browser
- **THEN** the browser tab displays the 📊 emoji as the favicon

## MODIFIED Requirements

### Requirement: Proposal Table Layout

The OpenSpec proposals table SHALL remove the "說明" (description) column. The remaining columns SHALL be: 專案, 代稱, 日期, 工作項.

#### Scenario: Proposal with description is rendered without description column

- **GIVEN** a proposal entry that includes a description field
- **WHEN** the proposals table is rendered
- **THEN** the table contains only the columns 專案, 代稱, 日期, 工作項 and the description is not displayed
