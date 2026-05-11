## Summary

Set up the complete project structure, CLI interface, git log parser, OpenSpec scanner, data model, and self-contained HTML report generator for the `contribution-showcase` Rust CLI tool. This establishes a standalone developer utility that aggregates conventional commits and archived OpenSpec proposals across all MapSight VMS repositories into a single-file interactive HTML dashboard — providing a consolidated contribution overview without requiring internet access or external services.

## Motivation

The MapSight VMS workspace spans seven-plus independent Git repositories, each with its own commit history and OpenSpec change archive. Understanding the overall contribution landscape — who built what, when features landed, how work distributes across projects — requires manually running `git log` in each repository and cross-referencing OpenSpec archives by hand. This is tedious, error-prone, and produces no shareable artifact. A dedicated CLI tool that scans all repositories in one pass, parses conventional commit messages into structured categories, collects archived OpenSpec proposal metadata, and renders a polished HTML dashboard eliminates this manual aggregation. The output is a single self-contained HTML file with no external dependencies, suitable for air-gapped environments and offline sharing with stakeholders.

## New Capabilities

- **rust-cli-scaffold**: Cargo project with `clap` 4 for argument parsing, providing `--config`, `--output`, `--author`, `--since`, and `--until` flags that override TOML configuration values, plus `anyhow` for ergonomic error handling throughout the codebase.
- **toml-config-loader**: TOML-based configuration system (`showcase.toml`) defining project entries (`[[projects]]` with `name`, `path`, `description`), output path, report title, and filter options (`author`, `since`, `until`, `types`), with CLI flags taking precedence over config file values.
- **git-log-collector**: Subprocess-based git log parser that invokes `git log --all --format='COMMIT_DELIM%H|||%aN|||%aI|||%s' --shortstat` with optional `--author` filtering, extracts commit hash, author, date, subject, and diff stats (insertions/deletions), and applies date-range and commit-type filters.
- **conventional-commit-parser**: Hand-written byte-level parser that identifies the `type(scope):` pattern in commit subjects, categorizing commits into `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`, `build`, `style`, and `perf` types, with unmatched subjects falling back to `other`.
- **openspec-archive-scanner**: Filesystem scanner using `std::fs::read_dir` that discovers archived OpenSpec proposals under each repository's `openspec/changes/archive/` directory, extracts slug and date from `YYYY-MM-DD-slug` directory names, reads proposal descriptions from `.openspec.yaml`, and counts completed tasks (`- [x]`) in `tasks.md` files. Note: the author filter does not apply to proposals — all proposals in the date range are included regardless of author.
- **html-report-generator**: Template-based renderer that serializes the aggregated `ShowcaseData` model as JSON, escapes HTML-sensitive characters for safe `<script>` embedding, injects it into a self-contained HTML template featuring a "Soft Paper" design theme (canvas `#f2f2f0`, surface `#fff`, ink `#0a0a0a`), and writes a single `dist/index.html` with all CSS and JavaScript inline — no external fonts, CDN links, or runtime dependencies. The dashboard includes hero section, KPI strip (5 cards), weekly timeline bar chart, commit-type horizontal bar breakdown, per-project summary cards, OpenSpec proposals table, and a sortable commit log (initially 100 rows, expandable to show all), all rendered in Traditional Chinese (zh-TW).

## Modified Capabilities

(none — this is a greenfield scaffold)

## Affected Areas

- Cargo project manifest and dependency declarations
- CLI argument parsing and configuration loading
- Git log subprocess invocation and output parsing
- Conventional commit message categorization
- OpenSpec archive filesystem scanning
- Data model serialization (ShowcaseData → JSON)
- HTML template and "Soft Paper" visual theme
- Default TOML configuration for VMS workspace repositories
