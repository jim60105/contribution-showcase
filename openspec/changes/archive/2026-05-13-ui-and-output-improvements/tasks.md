## 1. Dynamic Output Filename

- [x] 1.1 Implement `sanitize_title_for_filename()` helper in `src/config.rs` that sanitizes a title string to a filesystem-safe filename (replace non-ASCII-alnum/hyphen/underscore with hyphen, collapse, trim, lowercase, append `.html`), with fallback to `"index.html"` for empty/None/all-special-char titles
- [x] 1.2 Change `Config::output_path()` return type from `&str` to `String` and use `sanitize_title_for_filename()` when no explicit output path is configured, producing `dist/{sanitized-title}.html` instead of `dist/index.html`
- [x] 1.3 Update all `Config::output_path()` call sites in `src/main.rs` to handle the new `String` return type (the CLI `--output` override logic and output path usage)
- [x] 1.4 Add unit tests for `sanitize_title_for_filename()` covering: normal ASCII title, CJK-only title (fallback), special characters, empty string, None title, leading/trailing hyphens, consecutive hyphens
- [x] 1.5 Update existing `test_config_output_path_default` test to account for the new title-derived filename behavior and `String` return type

## 2. Adaptive Timeline Granularity

- [x] 2.1 Refactor `build_timeline()` in `src/collector.rs` to compute the date span (max date − min date) and select daily (<14 days), weekly (14–60 days), or monthly (>60 days) bucket granularity, using the appropriate label format (`%Y-%m-%d`, `%G-W%V`, `%Y-%m`)
- [x] 2.2 Add unit tests for `build_timeline()` covering: daily granularity (<14 days span), weekly granularity (14–60 days), monthly granularity (>60 days), boundary values (exactly 14 days, exactly 60 days, 61 days), single-day edge case, empty commits, height normalization consistency

## 3. Test Metrics Layout Reorder

- [x] 3.1 In `templates/page.html`, modify the `renderTestMetrics()` function to emit the `.test-kpi-row` summary cards (total files, total cases, average coverage) ABOVE the `.data-table` detail table

## 4. Scrollable Project Grid

- [x] 4.1 In `templates/page.html`, add `slide--scrollable` class to the `#slide-projects` section element and wrap `#projectGrid` in a `.scroll-container` div, matching the pattern used by commit log, proposal, and test metrics sections

## 5. Validation

- [x] 5.1 Run `cargo test` to verify all existing and new tests pass, `cargo fmt -- --check` for formatting, and `cargo clippy --all-targets` for lints
