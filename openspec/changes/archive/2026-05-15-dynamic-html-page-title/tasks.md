## 1. HTML Escaping Utility

- [x] 1.1 Add `escape_html` function in `src/main.rs` that escapes `&`, `<`, `>`, and `"` to their HTML entity equivalents
- [x] 1.2 Add unit tests for `escape_html`: plain text passthrough, ampersand, angle brackets, double quotes, combined characters, empty string

## 2. Template Placeholder

- [x] 2.1 Replace the static `<title>貢獻總覽</title>` in `templates/page.html` with `<title>__PAGE_TITLE__</title>`

## 3. Generation Pipeline

- [x] 3.1 In `run_generate()`, add a `__PAGE_TITLE__` replacement call using the escaped title, placed before the existing `__SHOWCASE_DATA__` replacement
- [x] 3.2 Add integration test: generated HTML `<title>` matches the configured title
- [x] 3.3 Add integration test: title with HTML-sensitive characters (`&`, `<`, `>`) is correctly escaped in `<title>`

## 4. Documentation

- [x] 4.1 Update `showcase.example.toml` comment (if needed) to note that the title also appears in the browser tab
