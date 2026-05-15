## Why

The HTML `<head><title>` element is hardcoded as `貢獻總覽` in
`templates/page.html`. Although the user already configures a `title` field in
`showcase.toml` (and JavaScript uses it for the visible cover heading), the
browser tab / bookmark / SEO-relevant `<title>` tag never reflects that
configuration. Users who customise the title expect the page title to match.

## What Changes

- Introduce a new `__PAGE_TITLE__` placeholder in `templates/page.html` that
  replaces the static `<title>貢獻總覽</title>`.
- During HTML generation, replace `__PAGE_TITLE__` with the configured title
  (HTML-entity-escaped) before writing the output file.
- No separate fallback needed — `collector::collect()` already resolves
  `Config.title` to `"貢獻總覽"` when absent, so `ShowcaseData.title` always
  has a value by the time generation runs.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `html-report-generation`: Add a new requirement for `<title>` placeholder
  replacement so the browser tab reflects the user-configured title.

## Impact

- `templates/page.html` — the `<title>` tag changes from a literal string to a
  placeholder.
- `src/main.rs` (`run_generate`) — one additional `.replace()` call before
  the existing `__SHOWCASE_DATA__` replacement, applying HTML-entity escaping to
  the title string.
- No new dependencies, no API changes, no migration needed.
