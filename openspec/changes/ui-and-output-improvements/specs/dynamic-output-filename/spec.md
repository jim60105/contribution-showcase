## ADDED Requirements

### Requirement: Title-Derived Default Filename
When no explicit output path is configured, the system SHALL derive the output filename from the `title` field in the TOML configuration. The derived filename SHALL be placed in the `dist/` output directory.

#### Scenario: Normal ASCII title produces kebab-case filename
- **WHEN** the config `title` is `"My Contribution Showcase"` and no explicit output path is configured
- **THEN** the output file SHALL be `dist/my-contribution-showcase.html`

#### Scenario: CJK title is transliterated to hyphens
- **WHEN** the config `title` is `"專案貢獻報告"` and no explicit output path is configured
- **THEN** each non-ASCII character SHALL be replaced with a hyphen, consecutive hyphens collapsed, and the output file SHALL be `dist/-.html` or fall back to `dist/index.html` if the sanitized result is empty

### Requirement: Filename Sanitization
The system SHALL sanitize the title for filesystem safety by applying the following transformations in order:
1. Replace any character that is not ASCII alphanumeric, hyphen (`-`), or underscore (`_`) with a hyphen.
2. Collapse consecutive hyphens into a single hyphen.
3. Trim leading and trailing hyphens.
4. Convert the result to lowercase.
5. Append the `.html` extension.

#### Scenario: Title with special characters is sanitized
- **WHEN** the config `title` is `"My Team's Contribution Showcase"`
- **THEN** the output filename SHALL be `my-team-s-contribution-showcase.html` within the `dist/` directory

#### Scenario: Title with mixed special characters and spaces
- **WHEN** the config `title` is `"Report #1: Q2 (2025) — Results!"`
- **THEN** non-alphanumeric/non-hyphen/non-underscore characters SHALL be replaced, consecutive hyphens collapsed, and the result converted to lowercase with `.html` appended

#### Scenario: CJK-only title sanitizes to empty string
- **WHEN** the config `title` is `"專案報告"` (all non-ASCII characters)
- **THEN** after sanitization the result is empty and the system SHALL fall back to `dist/index.html`

### Requirement: Empty or Missing Title Fallback
When the `title` field is `None`, empty (`""`), or sanitizes to an empty string after applying all sanitization rules, the system SHALL fall back to `dist/index.html`.

#### Scenario: Title is None
- **WHEN** the config `title` is not set (None)
- **THEN** the output file SHALL be `dist/index.html`

#### Scenario: Title is an empty string
- **WHEN** the config `title` is `""`
- **THEN** the output file SHALL be `dist/index.html`

#### Scenario: Title sanitizes to empty after processing
- **WHEN** the config `title` consists entirely of characters that are replaced and trimmed (e.g., `"---"`)
- **THEN** the output file SHALL be `dist/index.html`

### Requirement: Explicit Path Override
When an explicit output path is configured (via the TOML `output` field or the CLI `--output` flag), the system SHALL use that path as-is, regardless of the `title` value. This MUST preserve existing behavior.

#### Scenario: Explicit output path overrides title-derived filename
- **WHEN** the config `title` is `"My Report"` and an explicit output path `reports/custom.html` is configured
- **THEN** the output file SHALL be `reports/custom.html`

#### Scenario: CLI --output flag overrides title-derived filename
- **WHEN** the config `title` is `"My Report"` and the CLI flag `--output out.html` is provided
- **THEN** the output file SHALL be `out.html`
