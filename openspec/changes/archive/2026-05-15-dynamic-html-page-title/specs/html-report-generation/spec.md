## MODIFIED Requirements

### Requirement: Template Injection

The system SHALL load the HTML template at compile time via
`include_str!("../templates/page.html")` and perform two placeholder
replacements before writing the output:

1. Replace `__PAGE_TITLE__` inside the `<title>` element with the
   HTML-entity-escaped showcase title.
2. Replace `"__SHOWCASE_DATA__"` with the JSON-escaped showcase data payload.

The replacements SHALL be applied in the order listed above.

#### Scenario: Template contains the placeholder
- **GIVEN** the compiled-in HTML template contains the string
  `"__SHOWCASE_DATA__"`
- **WHEN** the renderer injects data
- **THEN** the placeholder is replaced with the escaped JSON payload exactly
  once

#### Scenario: Page title reflects configured title
- **GIVEN** a `showcase.toml` with `title = "我的超讚專案！貢獻總覽"`
- **WHEN** the renderer generates the HTML output
- **THEN** the `<title>` element in the output contains
  `我的超讚專案！貢獻總覽`

#### Scenario: Page title escapes HTML-sensitive characters
- **GIVEN** a `showcase.toml` with `title = "A & B <C>"`
- **WHEN** the renderer generates the HTML output
- **THEN** the `<title>` element in the output contains
  `A &amp; B &lt;C&gt;`

## ADDED Requirements

### Requirement: Page Title Placeholder in Template

The HTML template SHALL contain the placeholder `__PAGE_TITLE__` inside the
`<title>` element.

#### Scenario: Default title produces correct page title
- **GIVEN** a config with `title = "貢獻總覽"`
- **WHEN** the HTML is generated
- **THEN** the `<title>` element reads `<title>貢獻總覽</title>`

#### Scenario: Title with ampersand is escaped
- **GIVEN** a config with `title = "Tom & Jerry"`
- **WHEN** the HTML is generated
- **THEN** the `<title>` element reads `<title>Tom &amp; Jerry</title>`
