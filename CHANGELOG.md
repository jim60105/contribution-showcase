# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.3.0] - 2026-05-14

### Added

- Stacked type bar chart: timeline bars now show colored segments per commit type with consistent ordering
- Hover tooltip on timeline bars displaying per-type line breakdown with colored indicators and zh-TW formatted counts
- GitHub Pages hosting with automated deployment workflow
- Clickable project names in report cards linking to repository URLs
- URL sanitization (`safeProjectUrl()`) rejecting non-http(s) schemes for XSS prevention
- Deno framework detection (`deno.json`) for test metrics collection
- Framework-agnostic coverage execution: coverage commands now run regardless of framework detection

### Fixed

- Timeline X-axis labels no longer overlap with bar content (changed rotation geometry to extend labels downward)
- Undefined CSS variables resolved (`--card-bg` → `--surface`, `--fs-sm` → `--fs-xs`)
- XSS hardening: `escapeHtml()` applied to bar labels and peak annotation
- Windows compatibility fix for coverage test using echo with relative paths

## [0.2.0] - 2026-05-13

### Added

- Extended timeline granularity to five levels: daily, weekly, monthly, quarterly (`YYYY-Qn`), and yearly (`YYYY`), selected automatically using a ≤14-bucket cascade rule

### Fixed

- Corrected coverage configuration examples in `showcase.example.toml` (wrong `--html` flag replaced with `--cobertura`; result path updated to `coverage.xml`)

## [0.1.0] - 2026-05-13

### Added

- Multi-repository Git contribution scanner with TOML-based configuration
- Self-contained single-page HTML report with embedded CSS/JS (no external dependencies)
- Conventional Commit parsing with type-based categorization and distribution charts
- Commit timeline visualization with adaptive granularity (daily/weekly/monthly buckets)
- KPI dashboard showing commit counts, lines added/removed, and project statistics
- Per-project cards with description, branch info, and commit breakdowns
- OpenSpec proposal listing grouped by project
- Sortable commit log with type badges and line-change indicators
- Test metrics slide with per-project coverage percentages
- Per-project coverage execution via configurable shell commands with parallel spawning
- Auto-discovery of coverage results (Cobertura XML and Istanbul JSON formats)
- Dynamic output filename support with `{title}` and `{date}` placeholders
- `init` subcommand to generate starter configuration file
- `--version` CLI flag
- Author, date range, commit type, and hash-based filtering
- GitHub Actions CI pipeline with formatting, linting, testing (Linux/macOS/Windows), and security audit
- GitHub Actions release pipeline with cross-compilation (6 targets), smoke tests, and crates.io publishing
- CI code coverage job with cargo-llvm-cov and Codecov integration
- Release smoke tests for x86_64 (native) and aarch64 (QEMU) Linux binaries
- Full Traditional Chinese (zh-TW) user interface

### Fixed

- Breakdown labels no longer wrap unexpectedly in narrow viewports
- Path handling is now Windows-safe across all operations and tests

### Changed

- Project licensed under GPL-3.0-only

---

[Unreleased]: https://github.com/jim60105/contribution-showcase/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/jim60105/contribution-showcase/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jim60105/contribution-showcase/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jim60105/contribution-showcase/releases/tag/v0.1.0
