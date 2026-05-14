# Project URL Linking

## Purpose

Enables optional URL association with project entries, allowing project names in the HTML report to render as clickable hyperlinks when a URL is configured.

## Requirements

### Requirement: Project URL Configuration

Users SHALL be able to specify an optional `url` field per `[[projects]]` entry in the TOML config file. When the `url` field is omitted, no URL is associated with the project. The `url` field is an opaque string; the system SHALL NOT perform any validation or normalization on its value.

#### Scenario: Project with URL specified
- **GIVEN** a `[[projects]]` entry includes `url = "https://github.com/example/repo"`
- **WHEN** the config is loaded
- **THEN** the project's URL field contains `Some("https://github.com/example/repo")`

#### Scenario: Project without URL
- **GIVEN** a `[[projects]]` entry omits the `url` field
- **WHEN** the config is loaded
- **THEN** the project's URL field is `None`

### Requirement: Project URL Data Flow

The `url` field SHALL be propagated from the parsed TOML config through the data model into the JSON payload embedded in the generated HTML report. The JSON payload consumed by the HTML template SHALL include the `url` field for each project entry, with the value being either the URL string or `null` when not specified.

#### Scenario: URL present in JSON payload
- **GIVEN** a project entry has `url = "https://github.com/example/repo"` in the config
- **WHEN** the HTML report is generated
- **THEN** the JSON payload for that project contains `"url": "https://github.com/example/repo"`

#### Scenario: URL absent in JSON payload
- **GIVEN** a project entry does not have a `url` field in the config
- **WHEN** the HTML report is generated
- **THEN** the JSON payload for that project contains `"url": null`

### Requirement: Project Name Hyperlink Rendering

When a project has a URL, the project name in the project cards SHALL render as a clickable hyperlink using an `<a>` tag with `target="_blank"` and `rel="noopener noreferrer"` attributes. The hyperlink SHALL wrap the project name text within the `<h3>` heading element. When no URL is present or the URL is an empty string, the project name SHALL render as plain text without any `<a>` tag, preserving the current behavior.

#### Scenario: Project name rendered as hyperlink
- **GIVEN** a project entry has a URL value of `"https://github.com/example/repo"`
- **WHEN** the project card is rendered in the HTML report
- **THEN** the project name is wrapped in `<a href="https://github.com/example/repo" target="_blank" rel="noopener noreferrer">` inside the `<h3>` heading

#### Scenario: Project name rendered as plain text
- **GIVEN** a project entry has no URL (value is `null`)
- **WHEN** the project card is rendered in the HTML report
- **THEN** the project name is rendered as plain text inside the `<h3>` heading without any `<a>` tag

#### Scenario: Empty string URL treated as absent
- **GIVEN** a project entry has a URL value of `""`
- **WHEN** the project card is rendered in the HTML report
- **THEN** the project name is rendered as plain text inside the `<h3>` heading without any `<a>` tag

#### Scenario: URL with special HTML characters
- **GIVEN** a project entry has a URL containing characters that require HTML escaping (e.g., `&`)
- **WHEN** the project card is rendered in the HTML report
- **THEN** the URL is properly escaped in the `href` attribute to prevent XSS or rendering issues
