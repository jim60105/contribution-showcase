# Tasks: safety-hardening

## 1. HTML-Safe JSON Escaping

- [x] Extract a helper function `escape_json_for_html_script(json: &str) -> String` in `src/main.rs` that replaces HTML-sensitive characters after JSON serialization:
  ```rust
  fn escape_json_for_html_script(json: &str) -> String {
      json.replace('<', r"\u003c")
          .replace('>', r"\u003e")
          .replace('&', r"\u0026")
          .replace('\u{2028}', r"\u2028")
          .replace('\u{2029}', r"\u2029")
  }
  ```
- [x] Call the helper in `main()` between `serde_json::to_string(&data)?` and `template.replace("\"__SHOWCASE_DATA__\"", &json_data)`
- [x] This prevents XSS via crafted commit messages or proposal content containing `</script>` or similar sequences

## 2. Git Locale Safety

- [x] In `src/collector.rs` `collect_git_commits_filtered()`, add `.env("LC_ALL", "C")` to the `Command::new("git")` builder before the `.output()?` call
- [x] This forces English `--shortstat` output (e.g. `1 file changed, 2 insertions(+)`) regardless of the system locale

## 3. Date Filter Validation

- [x] Add a standalone `validate_date_range(since: Option<&str>, until: Option<&str>) -> Result<()>` function in `src/config.rs` that parses each value as `chrono::NaiveDate` (format `%Y-%m-%d`) and validates `since <= until` when both are present
- [x] In `src/main.rs`, call `validate_date_range()` **after** CLI overrides are merged into the config's FilterConfig — this ensures both config-file and CLI-provided dates are validated
- [x] In `src/collector.rs` `apply_filters()`, keep the existing string comparison for date filtering — validated YYYY-MM-DD strings sort lexicographically correctly. Date range is inclusive: `since <= item.date <= until`

## 4. Config Path Resolution

- [x] In `src/config.rs` `Config::load()`, after TOML parsing, determine the config file's parent directory via `std::path::Path::new(path).parent()`
- [x] For each `project.path` in the parsed config, if `std::path::Path::is_relative()`, resolve it against the config parent directory using `parent.join(relative_path)` and convert back to string via `.to_string_lossy().to_string()`
- [x] For `output.path`, if relative, resolve it against the config parent directory the same way
- [x] Absolute paths MUST be preserved unchanged

## 5. Configurable Ref Scope

- [x] In `src/config.rs`, add a `branch: Option<String>` field to the `ProjectConfig` struct (serde-optional, defaults to `None`)
- [x] In `Config::load()` or `Config::validate()`, validate that any `branch` value does not start with `-` (prevents git argument injection) and does not contain revspec operators (`..`, `^`, `~`, spaces); fail with a descriptive error if violated
- [x] In `src/collector.rs` `collect_git_commits_filtered()`, accept the optional branch parameter (e.g. `branch: Option<&str>`)
- [x] When `branch` is `Some(name)`, pass the branch name as a direct revision argument to git log; do not include `--all`
- [x] When `branch` is `None`, keep `--all` as the default (backward compatible)
- [x] Update the `collect()` function to pass `project.branch.as_deref()` to `collect_git_commits_filtered()`
- [x] Include the project name and branch in any git log error messages for context

## 6. Dependency Cleanup

- [x] Remove `walkdir = "2"` from `Cargo.toml` `[dependencies]`
- [x] Run `cargo check` to verify no compilation errors and update `Cargo.lock`

## 7. Unit Tests

In `src/main.rs` or a new `src/escape.rs` — add tests for `escape_json_for_html_script()`:

- [x] `test_escape_script_close_tag`: input containing `</script>` does not contain raw `</script` after escaping
- [x] `test_escape_angle_brackets_and_ampersand`: `<`, `>`, `&` are replaced with Unicode escapes
- [x] `test_escape_line_separator_chars`: U+2028 and U+2029 are replaced with `\u2028` and `\u2029`
- [x] `test_escaped_json_is_valid_js`: escaped output of a valid JSON string remains a valid JavaScript expression (no parse errors)

In `src/collector.rs` — add a `#[cfg(test)] mod tests` block with:

- [x] `test_parse_conventional_commit_feat`: `parse_conventional_commit("feat: add login")` → `("feat", "")`
- [x] `test_parse_conventional_commit_with_scope`: `parse_conventional_commit("fix(auth): token refresh")` → `("fix", "auth")`
- [x] `test_parse_conventional_commit_breaking`: `parse_conventional_commit("feat!: remove API")` → `("feat", "")`
- [x] `test_parse_conventional_commit_non_matching`: `parse_conventional_commit("Initial commit")` → `("other", "")`
- [x] `test_parse_conventional_commit_merge`: `parse_conventional_commit("Merge branch 'dev'")` → `("other", "")`
- [x] `test_type_label_known_types`: verify all 10 known conventional commit types map to their correct Traditional Chinese labels
- [x] `test_type_label_unknown_fallback`: verify an unknown type string maps to `"其他"`
- [x] `test_build_type_breakdown_no_duplicate_other`: create `CommitEntry` items with types `"initial"`, `"merge"`, `"unknown"` (all map to `"其他"`), verify only one `"其他"` entry appears in the result
- [x] `test_build_timeline_weekly_aggregation`: create commits spanning multiple ISO weeks, verify correct week-bucket counts in the timeline
- [x] `test_apply_filters_date_range`: create commits with various dates, apply `since`/`until` filters, verify only in-range commits remain (inclusive boundaries)
- [x] `test_apply_filters_type_filter`: create commits with various types, apply type filter, verify only matching commits remain
- [x] `test_shortstat_parsing_insertions_only`: mock `--shortstat` line `" 3 files changed, 42 insertions(+)\n"`, verify insertions=42, deletions=0
- [x] `test_shortstat_parsing_deletions_only`: mock `--shortstat` line `" 1 file changed, 5 deletions(-)\n"`, verify insertions=0, deletions=5
- [x] `test_shortstat_parsing_both`: mock `--shortstat` line `" 2 files changed, 10 insertions(+), 3 deletions(-)\n"`, verify insertions=10, deletions=3

In `src/config.rs` — add a `#[cfg(test)] mod tests` block with:

- [x] `test_config_load_minimal`: parse a minimal valid TOML string, verify struct fields
- [x] `test_config_output_path_default`: verify default output path is `"dist/index.html"` when not specified
- [x] `test_config_output_path_custom`: verify a custom output path is returned correctly
- [x] `test_date_validation_valid`: valid `YYYY-MM-DD` dates pass validation without error
- [x] `test_date_validation_invalid_format`: `"2024/01/01"` causes validation error
- [x] `test_date_validation_non_zero_padded`: `"2026-5-1"` causes validation error
- [x] `test_date_validation_since_after_until`: `since` > `until` causes validation error
- [x] `test_project_path_resolves_relative_to_config_parent`: load a config with relative path `"./repo"` from a config file at `/tmp/subdir/showcase.toml`, verify resolved path is `/tmp/subdir/repo`
- [x] `test_absolute_project_path_is_preserved`: load a config with absolute path `/opt/repos/foo`, verify path is unchanged
- [x] `test_branch_validation_rejects_dash_prefix`: a project with `branch = "--all"` causes validation error

## 8. Build and Test Verification

- [x] Run `cargo build --release` — verify successful compilation with all changes applied
- [x] Run `cargo test` — verify all unit tests pass
- [x] Run the tool from the repository root with default `showcase.toml` — verify same commit/proposal counts as before
- [x] Run the tool from a different CWD with `--config <path>/showcase.toml` — verify it finds repos correctly
- [x] Run with a malformed date `--since 2026/05/01` — verify clear error message and non-zero exit
- [x] Verify generated HTML opens in a browser without JavaScript console errors
- [x] Update `showcase.toml` comments to document config-relative path resolution behavior
