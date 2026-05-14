# Capability: html-report-generation (delta)

## ADDED Requirements

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
