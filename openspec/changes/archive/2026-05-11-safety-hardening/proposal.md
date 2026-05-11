# safety-hardening

## Summary

This proposal addresses seven security, correctness, and quality issues identified during the initial review of the contribution-showcase tool: HTML-safe JSON escaping to prevent script injection in generated reports, Git locale pinning to ensure reliable shortstat parsing across non-English systems, strict date filter validation via `chrono::NaiveDate` parsing, config-relative path resolution so that `--config` works from any working directory, an optional per-project `branch` field to replace the non-deterministic `--all` ref scope, removal of the unused `walkdir` dependency, and a unit test suite covering the conventional commit parser, Git log parsing, OpenSpec scanning, filtering, aggregation, and a regression test for the duplicate "其他" grouping bug.

## Motivation

The contribution-showcase tool is about to be shared beyond its original author, yet its current implementation contains a critical XSS-class vulnerability (unescaped JSON in HTML `<script>` tags), silent data-loss bugs on non-English Git locales and malformed date inputs, and a usability trap where relative paths break when the config file lives outside the working directory. There are also no automated tests, meaning regressions like the duplicate "其他" grouping bug can only be caught by manual inspection. Fixing all of these before wider adoption is far cheaper than patching them after users encounter corrupt reports, missing line counts, or — worst case — injected JavaScript in an operator's browser.

## Dependencies

- `init-project-structure`: provides the baseline CLI, collector, and template

## New Capabilities

- **unit-test-suite**: A comprehensive set of unit tests covering `parse_conventional_commit`, Git shortstat/log output parsing, OpenSpec archive scanning, date and type filtering, timeline and type breakdown aggregation, and a regression test for the duplicate "其他" grouping bug.

## Modified Capabilities

- **html-report-generation**: JSON payloads injected into HTML `<script>` tags are now escaped for HTML-sensitive characters (`<`, `>`, `&`, U+2028, U+2029) after serialization, preventing script injection from commit messages.
- **git-log-collection**: The Git subprocess environment is pinned to `LC_ALL=C` so that shortstat output is always in English, regardless of the host locale.
- **date-filtering**: The `since` and `until` values are parsed as `chrono::NaiveDate` with strict `YYYY-MM-DD` validation and a `since <= until` ordering check. Validation occurs after CLI overrides are merged into config, ensuring both config-file and CLI-provided dates are validated. Malformed or inverted ranges produce an immediate, clear error.
- **config-path-resolution**: All relative paths in `showcase.toml` — both project paths and output path — are resolved against the config file's parent directory instead of the process working directory.
- **git-ref-scope**: An optional `branch` field is added per project in the TOML config to restrict `git log` to specific refs; the value is validated to not start with `-` to prevent argument injection. The default remains `--all` for backward compatibility.
- **dependency-manifest**: The unused `walkdir` crate is removed from `Cargo.toml`.

## Affected Areas

- `src/main.rs` — HTML-safe JSON escaping before template injection
- `src/collector.rs` — Git subprocess environment (`LC_ALL=C`), ref scope (`--all` vs branch), unit tests
- `src/config.rs` — Date validation logic, config-relative path resolution, new `branch` field in `ProjectConfig`, unit tests
- `Cargo.toml` — Remove unused `walkdir` dependency
