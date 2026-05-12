# table-polish-and-commit-filter

## Why

The generated HTML report needs several quality-of-life improvements before the next delivery: one problematic commit (hash `dd33ee63…`) must be excluded from all metrics to avoid skewing the report, the commit log table lacks a hash column that would help operators trace entries back to source control, the proposals table carries a redundant "說明" column that adds visual noise without value, and the page has no favicon — leaving the browser tab with a generic icon.

## What Changes

1. **Commit hash exclusion filter** — Add an `exclude_hashes` config field under `[filters]` in `showcase.toml`. Commits whose full SHA-1 matches any entry in this list are excluded from all processing (timeline, breakdown, counts, tables).
2. **Commit hash column** — In the "提交紀錄" (commit log) table, insert a "Hash" column as the first column (before "日期") showing the first 8 characters of the commit hash in monospace font.
3. **Remove 說明 column from proposals table** — In the "OpenSpec 提案" table, drop the "說明" (description) column. The table retains: 專案, 代稱, 日期, 工作項.
4. **Emoji favicon** — Add an inline SVG data-URI favicon in `<head>` using a meaningful emoji.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- **toml-config-loader**: Add `exclude_hashes: Option<Vec<String>>` to `FilterConfig`. The field is optional; when absent, no hash-based exclusion occurs. Values are full 40-character hex strings.
- **git-log-collection**: In `apply_filters()`, add a hash-exclusion predicate that rejects any `CommitEntry` whose `hash` field matches an entry in `exclude_hashes`. This runs alongside existing date/type filters.
- **html-report-generation**: (a) Insert a "Hash" column as the first column in the commit log table header and rows, displaying the first 8 characters of `commit.hash` in a `<td class="mono">` cell. (b) Remove the "說明" `<th>` and corresponding `<td>` from the proposals table. (c) Add a `<link rel="icon">` tag in `<head>` with an inline SVG data URI containing an emoji.

## Impact

- `src/config.rs` — New `exclude_hashes` field in `FilterConfig`
- `src/collector.rs` — Hash exclusion predicate in `apply_filters()`
- `templates/page.html` — Commit table hash column, proposals table column removal, favicon link tag
- `showcase.toml` — Add `exclude_hashes` entry under `[filters]`
