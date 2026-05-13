# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/jim60105/contribution-showcase/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jim60105/contribution-showcase/releases/tag/v0.1.0
