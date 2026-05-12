# AGENTS.md — contribution-showcase

## Project Overview

**contribution-showcase** is a Rust CLI tool that generates a self-contained,
single-page HTML report summarizing Git contributions across multiple
repositories. It parses conventional commits, collects statistics (lines
added/removed, commit types, OpenSpec proposals, test metrics), and renders an
offline-ready HTML page with embedded JSON data.

## File Structure

```
src/
  main.rs        — CLI entry point (clap derive: `generate`, `init` subcommands),
                   HTML template embedding via include_str!, JSON escaping
  config.rs      — TOML config loading, date/branch validation, relative path resolution
  model.rs       — Data model structs (ShowcaseData, Summary, CommitEntry, ProjectData, etc.)
  collector.rs   — Git log parsing, conventional commit parsing, stat collection,
                   OpenSpec proposal scanning, test metrics
templates/
  page.html      — HTML template with __SHOWCASE_DATA__ placeholder (contains embedded CSS/JS)
showcase.example.toml — Starter config, embedded at compile time for `init` subcommand
openspec/        — Spec-driven change management (proposals, specs, tasks)
```

## Technology Stack

- **Rust** (edition 2021)
- **clap 4** (derive) — CLI parsing
- **serde / serde_json / toml** — serialization
- **chrono** — date handling
- **anyhow** — error propagation (`anyhow::Result` everywhere)
- **walkdir** — directory traversal
- **tempfile** (dev-only) — filesystem tests

## Commands

```bash
cargo build              # Build
cargo test               # Run all 74 tests
cargo fmt -- --check     # Check formatting
cargo clippy             # Lint
```

## Conventions

### Language

- Code comments, docstrings, commit messages, and log output: **English**.
- User-facing documentation (README): **Traditional Chinese (zh-TW)**.

### Coding Style

- Standard `rustfmt` formatting (`cargo fmt`).
- Use `anyhow::Result` for all fallible functions — do not use raw
  `std::result::Result` with custom error types.
- Only comment where it adds value (non-obvious invariants, edge cases).
  Do not narrate trivial code.

### Error Handling

- Propagate errors with `?` and `anyhow::Context` where helpful.
- Never `unwrap()` or `expect()` in non-test code unless the invariant is
  provably safe and documented.

### Testing

- **TDD approach** (Red → Green → Refactor).
- Tests live in `#[cfg(test)] mod tests` inside each source file.
- Use `tempfile::tempdir()` for any test that touches the filesystem.
- Run `cargo test` to verify all tests pass before and after changes.

### Git Workflow

- **Gitflow**: `master` (releases), `dev` (integration), `feature/*`, `fix/*`.
- **Conventional Commits** with `--signoff`.
- Merges are `--no-ff`; rebase onto `dev` before merging.

### Config File

- `showcase.toml` is **gitignored** (contains real project paths).
- `showcase.example.toml` is the checked-in template and is embedded at
  compile time via `include_str!` for the `init` subcommand.

## Key Implementation Details

### Init Subcommand

The `init` command writes `showcase.example.toml` content to a new file.
It uses `OpenOptions::new().create_new(true)` for an atomic overwrite guard —
the operation fails if the target file already exists.

### HTML Generation

The generated HTML must be **fully self-contained** — no external CDN links,
no remote fonts, no network requests. All CSS and JS are inlined in
`templates/page.html`. The `__SHOWCASE_DATA__` placeholder is replaced with
JSON-escaped showcase data at generation time.

### Security

- **Branch name validation**: branch names are validated before being passed
  to Git commands to prevent argument injection.
- **No network calls**: the tool makes zero external network requests at
  runtime.

## OpenSpec Change Management

This project uses **spec-driven change management** under `openspec/`.
Non-trivial changes should start as a proposal in `openspec/changes/<name>/`
before implementation begins. See the OpenSpec skills for the workflow.
