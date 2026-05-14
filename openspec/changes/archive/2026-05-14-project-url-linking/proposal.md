## Why

Users want their generated HTML report to link back to the source repository for each project. Currently, the project name renders as plain text with no way to navigate to the project's hosted repository (GitHub, GitLab, Gitea, etc.).

Rather than auto-detecting repository URLs from Git remotes — which introduces complexity around multiple remotes, authentication, and varying Git hosting providers — a simple user-specified URL keeps the implementation straightforward and provider-agnostic. Users already configure each project manually in TOML; adding an optional `url` field is a natural extension.

## What Changes

A new optional `url` field is added to the `[[projects]]` table in the TOML configuration. When provided, the project name in the generated HTML report becomes a clickable hyperlink opening in a new tab. When omitted, the project name renders as plain text, preserving current behavior.

- **Config**: `[[projects]]` gains an optional `url` string field.
- **Data model**: `ProjectData` carries the optional URL through to the template.
- **HTML rendering**: The project name heading conditionally wraps in an `<a>` tag targeting `_blank` when a URL is present.
- **Example config**: `showcase.example.toml` is updated to document the new field.

No existing behavior changes for configurations that omit the field. No network requests are introduced — the URL is used purely as an `href` value in the generated HTML.

## Capabilities

### New Capabilities

- `project-url-linking`: Allow users to specify an optional URL per project that renders the project name as a hyperlink in the HTML report.

### Modified Capabilities

- `toml-config-loader`: Accept a new optional `url` field in `[[projects]]`.
- `html-report-generation`: Render the project name as a hyperlink when a URL is configured.

## Impact

- **`src/config.rs`**: Add `url: Option<String>` to `ProjectConfig`.
- **`src/model.rs`**: Add `url: Option<String>` to `ProjectData`.
- **`src/collector.rs`**: Pass the URL from config through to the project data model.
- **`templates/page.html`**: Conditionally wrap the project name `<h3>` content in an `<a href="..." target="_blank" rel="noopener noreferrer">` tag.
- **`showcase.example.toml`**: Add a commented `url` example to the `[[projects]]` section.
- **No breaking changes**: The field is optional with no default; existing configs work unchanged.
- **No new dependencies**: No crates are added.
