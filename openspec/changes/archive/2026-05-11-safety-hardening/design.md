# Safety Hardening

## Context

The `contribution-showcase` Rust CLI tool was recently initialized. It scans git
repositories and OpenSpec archives, then generates a self-contained HTML
dashboard by injecting JSON data into an HTML template via
`template.replace("\"__SHOWCASE_DATA__\"", &json_data)`.

A rubber-duck review of the initial implementation identified seven concrete
issues spanning security, correctness, portability, and maintainability:

| # | Area | Severity | File |
|---|------|----------|------|
| 1 | JSON injection in `<script>` | **High** | `src/main.rs` |
| 2 | Git locale dependency | Medium | `src/collector.rs` |
| 3 | Silent date mis-filtering | Medium | `src/collector.rs` |
| 4 | CWD-relative path resolution | Medium | `src/config.rs` |
| 5 | Unused `walkdir` dependency | Low | `Cargo.toml` |
| 6 | Zero test coverage | Medium | all |
| 7 | Unscoped `--all` ref selection | Low | `src/collector.rs` |

This proposal addresses all seven issues in a single coordinated change.

## Goals / Non-Goals

### Goals

- Eliminate the `</script>` injection vector so arbitrary commit messages cannot
  break the generated HTML page.
- Make git `--shortstat` parsing locale-independent across all operating systems.
- Fail fast with a clear error message when date filters are malformed, instead
  of silently producing incorrect results.
- Resolve config-file paths relative to the config file's parent directory,
  matching the convention of `tsconfig.json`, `pyproject.toml`, and
  `Cargo.toml`.
- Allow users to scope git log to a specific branch instead of always using
  `--all`.
- Remove the unused `walkdir` dependency to reduce compile time and audit
  surface.
- Achieve unit test coverage for all parsing and filtering functions in
  `collector.rs` and `config.rs`.

### Non-Goals

- Rewriting the HTML template to use a template engine (out of scope; the
  current `replace()` approach is fine once the injection is fixed).
- Adding integration tests that invoke the real `git` binary against fixture
  repositories (unit tests with mock data are sufficient for now).
- Changing the output format from a single self-contained HTML file to anything
  else.
- Supporting multiple output formats (PDF, Markdown, etc.).
- Migrating from `clap` to a different CLI framework.

## Decisions

### 1. HTML-Safe JSON Escaping — Post-serialization character replacement

**Decision:** After `serde_json::to_string()`, replace dangerous characters with
their Unicode escape equivalents before injecting into the HTML template:

| Character | Replacement |
|-----------|-------------|
| `<`       | `\u003c`    |
| `>`       | `\u003e`    |
| `&`       | `\u0026`    |
| U+2028    | `\u2028`    |
| U+2029    | `\u2029`    |

```rust
fn escape_for_script_tag(json: &str) -> String {
    json.replace('<', r"\u003c")
        .replace('>', r"\u003e")
        .replace('&', r"\u0026")
        .replace('\u{2028}', r"\u2028")
        .replace('\u{2029}', r"\u2029")
}
```

**Why:** The HTML spec terminates a `<script>` block when `</script>` appears in
the source text, regardless of JavaScript string context. A commit message
containing `</script><script>alert(1)</script>` would break the page. Replacing
`<` and `>` with Unicode escapes is transparent to `JSON.parse()` — the
JavaScript side needs zero changes.

**Alternatives considered:**

- *Base64 encoding*: Adds a mandatory `atob()` + `JSON.parse()` step in the
  browser; more complex and harder to debug.
- *`<script type="application/json">` with `document.getElementById()`*:
  Requires DOM parsing on the JS side and changes the template structure.
- *Template engine with auto-escaping (e.g. Tera)*: Adds a heavyweight
  dependency for a single substitution.

### 2. Git Locale — `LC_ALL=C` environment variable

**Decision:** Add `.env("LC_ALL", "C")` to the `Command::new("git")` builder in
`collect_git_commits_filtered()`.

```rust
let output = Command::new("git")
    .env("LC_ALL", "C")
    .args(&["log", "--all", "--format=...", "--shortstat"])
    // ...
```

**Why:** `LC_ALL=C` forces Git to output in the POSIX locale, ensuring the
English strings `"insertion"` and `"deletion"` always appear in `--shortstat`
output. This is a single-line change with zero risk — the environment variable
is scoped to the child process and does not affect the parent.

**Alternatives considered:**

- *Use `--numstat` instead of `--shortstat`*: Produces per-file stats (more
  structured) but requires rewriting the parser and generates larger output.
- *Parse all known locale variants* (e.g. German "Einfügung", French
  "insertion"): Fragile and unmaintainable as Git adds or changes translations.

### 3. Date Filter Validation — `chrono::NaiveDate` parsing

**Decision:** After CLI overrides are merged into the config (in `main.rs`), validate `since` and `until` filter values by parsing them as `chrono::NaiveDate`. Fail fast with `anyhow::bail!()` if parsing fails. When both are present, validate that `since <= until`. Date filtering uses inclusive boundaries: `since <= item.date <= until`.

The validation function should be callable independently so it applies to both config-file and CLI-provided dates:

```rust
fn validate_date_range(since: Option<&str>, until: Option<&str>) -> Result<()> {
    if let Some(s) = since {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .with_context(|| format!("invalid 'since' date: {s}"))?;
    }
    if let Some(u) = until {
        NaiveDate::parse_from_str(u, "%Y-%m-%d")
            .with_context(|| format!("invalid 'until' date: {u}"))?;
    }
    if let (Some(s), Some(u)) = (since, until) {
        if s > u {
            anyhow::bail!("'since' ({s}) must not be after 'until' ({u})");
        }
    }
    Ok(())
}
```

**Why:** The current code uses lexicographic string comparison
(`c.date < *since`), which silently produces wrong results for non-zero-padded
dates like `"2026-5-1"` (compares as greater than `"2026-12-31"`). Fail-fast
with a clear error is strictly better than silent incorrect filtering.
`chrono` is already a transitive dependency.

**Alternatives considered:**

- *Regex validation (`\d{4}-\d{2}-\d{2}`)*: Does not catch semantically invalid
  dates (e.g. `2024-02-31`).
- *No validation (current behavior)*: Causes silent data-integrity bugs.

### 4. Config Path Resolution — Resolve against config file parent directory

**Decision:** In `Config::load()`, after TOML deserialization, resolve each
relative `ProjectConfig.path` and `OutputConfig.path` against the config file's
parent directory using `Path::parent()` + `Path::join()`. Absolute paths are
left unchanged.

```rust
let config_dir = config_path.parent().unwrap_or(Path::new("."));
for project in &mut config.projects {
    if project.path.is_relative() {
        project.path = config_dir.join(&project.path);
    }
}
// Same for config.output.path
```

**Why:** Users expect paths in a config file to be relative to that file, not to
whichever directory they happen to invoke the CLI from. This is the universal
convention: `tsconfig.json`, `pyproject.toml`, `Cargo.toml`, `.eslintrc`, etc.
The current CWD-relative behavior produces different results depending on where
the user runs the command.

**Alternatives considered:**

- *Document CWD-relative behavior*: Confusing and error-prone for users.
- *Always require absolute paths*: Poor UX, especially in checked-in configs.
- *Add `--basedir` flag*: Unnecessary complexity when the config file location
  is a natural anchor.

### 5. Configurable Ref Scope — Optional `branch` field in `ProjectConfig`

**Decision:** Add an optional `branch: Option<String>` field to
`ProjectConfig`. When set, replace `--all` with the specified branch name in the
`git log` command. When absent, default to `--all` for backward compatibility.
The branch value MUST be validated to not start with `-` to prevent argument
injection (e.g., `--since=1970-01-01` being interpreted as a git flag), and must
not contain revspec operators (`..`, `^`, `~`, spaces) to prevent scope
manipulation. The branch is passed as a direct revision argument to `git log`.

```toml
# showcase.toml
[[projects]]
name = "Platform Core"
path = "../my-backend"
branch = "dev"   # optional — omit to scan all refs
```

```rust
if let Some(ref branch) = project.branch {
    if branch.starts_with('-') || branch.contains("..") {
        anyhow::bail!("invalid branch name: {branch}");
    }
    args.push(branch.clone());
} else {
    args.push("--all".to_string());
}
```

**Why:** `--all` includes every ref in the local clone — stale feature branches,
remote tracking refs, and tags — producing noisy, machine-dependent reports.
Letting users specify a branch (e.g. `main` or `dev`) scopes the report to
meaningful history. The `--all` default preserves existing behavior.

**Alternatives considered:**

- *`refs: Vec<String>` array for multiple refs*: More flexible but adds
  complexity for a feature nobody has requested yet. Easy to upgrade to later.
- *Always use the default branch*: Too restrictive; some users want `dev`.
- *Remove `--all` entirely*: Breaking change with no migration path.

### 6. Dependency Cleanup — Remove `walkdir`

**Decision:** Remove `walkdir = "2"` from `[dependencies]` in `Cargo.toml`.

**Why:** `walkdir` is declared but never imported. The OpenSpec scanner uses
`std::fs::read_dir` for shallow directory listing, which is sufficient because
OpenSpec archives are always structured one level deep
(`openspec/changes/<name>/`). Unused dependencies increase compile time, expand
the advisory audit surface, and violate the minimal-footprint principle
established at project initialization.

**Alternatives considered:**

- *Refactor the scanner to use `walkdir`*: Unnecessary complexity for a known
  shallow directory structure.
- *Keep the unused dependency*: Violates the project's minimal-dependency
  principle.

### 7. Unit Test Suite — Built-in `#[cfg(test)]` modules

**Decision:** Add `#[cfg(test)] mod tests { ... }` blocks in `collector.rs` and
`config.rs`. Tests use Rust's built-in `#[test]` attribute and standard
assertion macros. No external test framework.

**Why:** Rust's built-in test support is fully sufficient for unit testing pure
functions. Tests run with `cargo test` — no additional dependencies or
configuration required.

**Test coverage targets:**

| Function | Cases |
|----------|-------|
| `parse_conventional_commit()` | Valid types (`feat`, `fix`, `docs`), scoped commits, breaking changes (`!`), non-matching subjects, empty input |
| Git log parsing | Mock `--shortstat` output, missing stat fields, zero insertions/deletions |
| `collect_proposals()` | Mock directory structures (valid archives, empty dirs, missing metadata) |
| `apply_filters()` | Date range filtering (inclusive boundaries), type filtering, empty filter set (pass-through), combined filters |
| `build_type_breakdown()` | Regression test for duplicate "其他" (Other) grouping |
| `build_timeline()` | Weekly aggregation, single entry, entries spanning year boundary |
| Date validation (D3) | Valid ISO dates, non-zero-padded dates, invalid dates (Feb 31), `since > until` |
| Path resolution (D4) | Relative paths, absolute paths, missing parent directory |
| JSON escaping (D1) | `</script>` in input, `&` in input, U+2028/U+2029 characters |

**Alternatives considered:**

- *`rstest` for parameterized tests*: Adds a proc-macro dependency; standard
  `#[test]` with helper functions achieves the same result.
- *`mockall` for mocking*: Overkill — the functions under test are pure or can
  be tested with mock data passed as arguments.
- *Integration tests only* (`tests/` directory): Insufficient for verifying
  parser edge cases; unit tests co-located with the code under test are more
  maintainable.

## Risks / Trade-offs

1. **Path resolution is a behavioral change.** Users who rely on CWD-relative
   path resolution in their config files will get different behavior after this
   change. Mitigation: this is a young tool with no external users yet, and the
   new behavior matches universal conventions. If needed, a `--basedir` escape
   hatch can be added later.

2. **The `branch` field adds complexity to git log command construction.** The
   git log argument builder must now conditionally include either `--all` or a
   branch name. Mitigation: the logic is a simple `if let Some / else` branch
   — trivial to read and test. The field is optional, so existing configs
   continue to work without modification.

3. **Test suite maintenance overhead.** Tests must be updated when parser
   behavior changes (e.g. new commit types, different stat formats).
   Mitigation: the test suite is the safety net that makes future changes
   *possible* — the cost of maintaining tests is far lower than the cost of
   undetected regressions.

4. **`LC_ALL=C` masks legitimate locale issues.** Forcing the POSIX locale on
   the git subprocess means we will never see locale-specific output, even if a
   future feature needs it. Mitigation: the override is scoped to the git
   subprocess only and does not affect the parent process. If locale-aware git
   output is ever needed, the override can be removed for that specific
   invocation.
