# Tasks: table-polish-and-commit-filter

Add commit-hash exclusion filter, commit hash column, remove proposal description column, and add emoji favicon.

---

## 1. Config: exclude_hashes Field

- [x] 1.1 Add `exclude_hashes: Option<Vec<String>>` to `FilterConfig` in `src/config.rs`
- [x] 1.2 Update `Config::filters()` to clone the new `exclude_hashes` field into the returned `FilterConfig`
- [x] 1.3 Add `exclude_hashes` entry under `[filters]` in `showcase.toml` with the hash `dd33ee63950bb49a284de835528343561f1a70d5`

## 2. Backend: Hash Exclusion Filter

- [x] 2.1 Update all existing `FilterConfig` struct literals in tests to include `exclude_hashes: None`
- [x] 2.2 In `apply_filters()` in `src/collector.rs`, add a hash-exclusion predicate that rejects any `CommitEntry` whose `hash` matches an entry in `filters.exclude_hashes` (exact full-hash comparison, before existing date/type checks)

## 3. Template: Commit Hash Column

- [x] 3.1 In `renderCommits()` in `templates/page.html`, add a static (non-sortable) "Hash" `<th>` as the first column header, before the "日期" sort header
- [x] 3.2 In the commit row loop, add a `<td class="mono">` as the first cell displaying `c.hash.substring(0, 8)` (escaped)
- [x] 3.3 Add `overflow-x: auto` to the `.scroll-container` CSS rule to enable horizontal scroll when the table overflows

## 4. Template: Remove Proposal Description Column

- [x] 4.1 In `renderProposals()` in `templates/page.html`, remove the "說明" `<th>` from the header row
- [x] 4.2 Remove the corresponding `<td>` that renders `p.description` from the proposal row loop

## 5. Template: Emoji Favicon

- [x] 5.1 In `templates/page.html` `<head>`, add `<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'%3E%3Ctext y='.9em' font-size='90'%3E📊%3C/text%3E%3C/svg%3E">` after the `<title>` tag

## 6. Unit Tests

- [x] 6.1 Add test in `src/config.rs` — config with `exclude_hashes` parses correctly and populates `FilterConfig.exclude_hashes`
- [x] 6.2 Add test in `src/config.rs` — config without `exclude_hashes` results in `None`
- [x] 6.3 Add test in `src/collector.rs` — `apply_filters()` excludes a commit whose hash is in `exclude_hashes`
- [x] 6.4 Add test in `src/collector.rs` — `apply_filters()` retains all commits when `exclude_hashes` is empty or `None`
- [x] 6.5 Add test in `src/collector.rs` — `apply_filters()` retains all commits when `exclude_hashes` contains a hash that matches no commit
- [x] 6.6 Add test in `src/collector.rs` — `apply_filters()` does NOT exclude a commit when `exclude_hashes` contains only a short prefix of that commit's hash

## 7. Build and Verification

- [x] 7.1 Run `cargo test` — all existing and new tests pass
- [x] 7.2 Run `cargo build --release` — no warnings, no errors
- [x] 7.3 Run the tool against the workspace (`cargo run -- showcase.toml`) and verify the generated HTML: hash column present in commit table, 說明 column absent from proposals table, favicon visible in browser tab
