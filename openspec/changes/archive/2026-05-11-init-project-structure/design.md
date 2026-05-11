## Context

The MapSight VMS workspace contains 7+ independently versioned Git repositories. During project reviews, stakeholders need a consolidated view of development contributions — commit counts, conventional commit breakdowns, OpenSpec proposal history, and line-change statistics — across all repos, presented as a single portfolio-style HTML dashboard.

No existing tool in the workspace addresses this. Generic tools like `git-stats` or GitHub contribution graphs are unsuitable: the platform runs in air-gapped environments (no GitHub/GitLab access), reports must be fully self-contained HTML (no CDN, no external fonts), and the output must understand workspace-specific conventions (Conventional Commits, OpenSpec archive structure).

`contribution-showcase` is a standalone Rust CLI developer tool. It is NOT part of the runtime surveillance platform — it runs on developer machines to generate these portfolio reports. This design document is a retrospective record of the architectural decisions made during initial implementation.

## Goals / Non-Goals

### Goals
- Produce a single self-contained HTML file with no external dependencies (air-gapped compatible).
- Scan multiple Git repositories in a single run, driven by a TOML configuration file.
- Parse Conventional Commit subjects and produce type-level breakdowns per project.
- Discover completed OpenSpec proposals from archive directories and surface them in the report.
- Build quickly, have minimal dependencies, and run without installation (single native executable, no language runtime required).

### Non-Goals
- Real-time or incremental dashboard updates (batch tool, runs on demand).
- Integration with any CI/CD pipeline or automated publishing.
- Replacing the VMS.wiki as the authoritative project documentation source.
- Supporting non-Git version control systems.
- Providing interactive editing or annotation of the generated report.

## Decisions

### 1. Language Choice — Rust

**Decision:** Implement the tool in Rust, producing a single native executable with no runtime dependencies beyond the `git` CLI. Note: the default Rust toolchain on Linux links dynamically against system libc; a truly static binary would require building with the `x86_64-unknown-linux-musl` target.

**Why:** Rust compiles to a self-contained executable that runs on any Linux workstation without installing a language runtime (Python, Node, etc.). This is critical for air-gapped developer environments where installing toolchains may not be straightforward. Strong typing catches data-model mismatches at compile time, and startup is near-instant compared to interpreted alternatives.

**Alternatives considered:**
- **Python**: Available in the workspace and familiar to the team, but requires a Python runtime and `uv` setup on every developer machine. Distributing a CLI tool as a Python package adds friction.
- **TypeScript/Node**: Same runtime-installation concern. Bundling via `pkg` or `bun compile` is possible but adds build complexity.
- **Go**: Similar single-binary story, but the team has no existing Go expertise in this workspace.

### 2. Data Flow — Pipeline Pattern

**Decision:** Structure the application as a linear pipeline: `showcase.toml → Config::load() → collector::collect() → ShowcaseData → serde_json → template injection → dist/index.html`.

**Why:** A linear pipeline makes the data flow explicit and testable at each stage. Each phase has a single responsibility: configuration parsing, data collection, serialization, and rendering. There are no circular dependencies or shared mutable state between stages.

**Alternatives considered:**
- **Event-driven / streaming architecture**: Overkill for a batch tool that processes a bounded set of repos sequentially.
- **Monolithic main function**: Would work for the current scope but resists decomposition as features are added.

### 3. Git Log Parsing — Custom Delimiter Format

**Decision:** Invoke `git log --all --format='COMMIT_DELIM%H|||%aN|||%aI|||%s' --shortstat` as a subprocess with `LC_ALL=C` environment variable (to ensure English-language shortstat output) and parse the output with a custom delimiter protocol. `COMMIT_DELIM` marks commit boundaries; `|||` separates fields (hash, author name, ISO date, subject). `--shortstat` lines are parsed separately for insertion/deletion counts.

**Why:** The `COMMIT_DELIM` prefix solves the fundamental problem of `--shortstat` output: stat lines appear on separate lines after the format string, with no guaranteed delimiter between them and the next commit. A chosen sentinel prefix provides unambiguous record boundaries. The `|||` separator is chosen because it is extremely unlikely to appear in author names or subjects, unlike commas or tabs. Note that `--all` scans all local refs (branches, tags, stashes), which means results may vary across machines depending on local branch state.

**Alternatives considered:**
- **`git log --format='%H,%aN,%aI,%s'` (CSV-style)**: Commas and pipes appear in commit subjects; quoting rules are fragile.
- **`git log` with `--numstat`**: Produces per-file stats requiring aggregation, significantly more output to parse.
- **libgit2 / git2-rs crate**: Eliminates the subprocess but adds a heavy native dependency (~2MB), complicates cross-compilation, and requires reimplementing shortstat aggregation manually.

### 4. Conventional Commit Parsing — Hand-written Parser

**Decision:** Implement `parse_conventional_commit()` as a hand-written byte-level parser that scans for type, optional `(scope)`, optional `!`, required `:` delimiter, and description. Non-matching subjects are classified as type `"other"`. Type labels are mapped to Traditional Chinese strings (e.g., `feat` → `新功能`, `fix` → `錯誤修復`).

**Why:** The Conventional Commit grammar is simple enough that a hand-written parser is shorter and clearer than pulling in a regex crate. Avoiding the `regex` dependency saves ~1MB of compile-time overhead and keeps the dependency tree minimal. The zh-TW labels match the workspace's documentation language convention.

**Alternatives considered:**
- **`regex` crate**: Well-tested but adds a significant dependency for a ~20-line parser. The regex pattern itself would be less readable than explicit character scanning.
- **`nom` parser combinator**: Powerful but extreme overkill for this grammar; adds learning curve for contributors.
- **Treat all commits as untyped**: Loses the primary analytical value of the tool.

### 5. OpenSpec Scanning — Filesystem-based

**Decision:** Scan `openspec/changes/archive/` directories in each configured repository using `std::fs::read_dir`. Parse directory names matching the `YYYY-MM-DD-slug` convention to extract dates. Count completed tasks from `- [x]` checkbox lines in `tasks.md`. Read proposal descriptions from `.openspec.yaml` via simple line-level string parsing (no YAML parser). Note: the author filter does not apply to proposals — all proposals in the scanned date range are included regardless of author, since OpenSpec archives do not record author metadata.

**Why:** OpenSpec archives follow a strict directory naming convention that is already enforced by the workflow. Parsing the date from the directory name is more reliable than parsing file metadata. Avoiding a YAML parser dependency (for reading a single `description:` field) keeps the dependency count minimal — simple `starts_with` / `trim` operations suffice.

**Alternatives considered:**
- **Full YAML parser (`serde_yaml`)**: Correct for complex YAML but overkill for extracting one field. Adds a transitive dependency on `unsafe-libyaml` or `yaml-rust2`.
- **Parsing `design.md` instead of `.openspec.yaml`**: Less structured; description extraction would require Markdown heading parsing.
- **Ignoring OpenSpec data entirely**: Misses a key contribution dimension that is unique to this workspace.

### 6. HTML Template — JSON Data Injection

**Decision:** Embed the HTML template at compile time via `include_str!("../templates/page.html")`. At runtime, serialize `ShowcaseData` to JSON, escape HTML-sensitive characters (`<`, `>`, `&`, U+2028, U+2029) to prevent `</script>` injection, and inject it by replacing the placeholder `"__SHOWCASE_DATA__"` in the template string. The template contains `const DATA = "__SHOWCASE_DATA__"` which becomes `const DATA = {...actual JSON...}` after replacement.

**Why:** This approach requires zero template engine dependencies. The HTML/CSS/JS is a single self-contained file that designers can edit with standard web tools. JSON serialization is handled entirely by `serde_json`, which is already a dependency for the data model. The placeholder replacement is a single `str::replace` call.

**Alternatives considered:**
- **Tera / Handlebars / Askama template engine**: Adds a dependency and a custom template syntax. Designers cannot preview the template in a browser without a build step.
- **Server-side rendering (serve the HTML dynamically)**: Contradicts the "single static file" requirement.
- **Generating HTML programmatically in Rust**: Extremely verbose; loses the ability to iterate on design in a browser.

### 7. Design Theme — Soft Paper

**Decision:** Use a warm neutral "Soft Paper" color palette: canvas `#f2f2f0`, surface `#fff`, ink `#0a0a0a`. System font stack (`-apple-system, BlinkMacSystemFont, "Segoe UI", ...`) for zero external font dependencies. Commit type colors follow a carefully chosen palette (e.g., `feat=#2f7d4a`, `fix=#b53a2a`, `docs=#6366f1`). All styling uses CSS custom properties for easy theming. Responsive breakpoints at 900px, 768px, and 480px.

**Why:** The air-gapped constraint prohibits loading Google Fonts or any CDN-hosted assets. System fonts render instantly and look native on every platform. The warm neutral palette provides comfortable reading for what is essentially a data-heavy report document. CSS custom properties centralize the palette for future theme variants.

**Alternatives considered:**
- **Dark theme**: Harder to read for dense tabular data; less suitable for printing.
- **Bundled web fonts (e.g., Noto Sans TC)**: Adds 2-5MB to the HTML file for marginal visual improvement over system fonts.
- **CSS framework (Tailwind, Bootstrap)**: Adds build complexity for a single-page template that is hand-authored once.

### 8. Dashboard Layout — Seven Sections

**Decision:** Structure the dashboard as seven distinct sections: (1) Hero with title, author, date range, generation timestamp; (2) KPI strip with 5 metric cards (commits, repos with matching commits, proposals, lines added, lines removed); (3) Charts row with a 2-column grid containing a weekly timeline bar chart and a horizontal type breakdown chart; (4) Project cards showing per-repo statistics; (5) Proposals table grouped by project; (6) Sortable commit log table with type badges, initially showing 100 rows and expandable to show all; (7) Footer with generation metadata.

**Why:** The seven sections follow a top-down information hierarchy: summary → trends → per-project detail → raw data. This matches how stakeholders consume contribution reports — executives scan KPIs, managers review project breakdowns, and developers drill into commit logs. The progressive-disclosure pattern (50 initial rows, expand by 100) keeps the page responsive even with thousands of commits.

**Alternatives considered:**
- **Tabbed interface**: Hides information behind clicks; less suitable for a static report that may be printed or shared as a file.
- **Single scrolling list**: No visual hierarchy; KPIs would be buried below the fold.
- **Interactive charts (D3.js, Chart.js)**: Adds JavaScript library dependencies; CSS-only bar charts suffice for the current data complexity.

### 9. Configuration — TOML with CLI Overrides

**Decision:** Use a TOML configuration file (`showcase.toml`) as the primary configuration source, with CLI flags (`--output`, `--author`, `--since`, `--until`) that override specific fields. The `--config` flag (default `"showcase.toml"`) specifies the config file path. Projects are defined as `[[projects]]` array entries with `name`, `path`, and optional `description`.

**Why:** TOML is human-readable, supports nested structures, and is a natural fit for Rust (the `toml` crate is well-maintained and lightweight). A config file is essential because the tool scans multiple repositories — listing 7+ repo paths as CLI arguments would be unwieldy. CLI overrides provide flexibility for one-off runs (e.g., filtering by a different author or date range) without editing the config file.

**Alternatives considered:**
- **YAML**: More feature-rich but the `serde_yaml` crate adds a heavier dependency chain. YAML's indentation sensitivity is error-prone for configuration files.
- **JSON**: No comments, verbose syntax — poor UX for a hand-edited config file.
- **CLI-only (no config file)**: Impractical for specifying multiple project paths with descriptions.

### 10. Dependency Strategy — Minimal Footprint

**Decision:** Limit direct dependencies to 7 crates: `clap` (CLI parsing), `serde` + `serde_json` (serialization), `toml` (config parsing), `chrono` (date handling), `walkdir` (filesystem traversal, currently unused but reserved for future recursive scanning), and `anyhow` (error handling). No runtime dependencies beyond the `git` CLI. Note: transitive dependencies are larger; audit with `cargo tree` for supply-chain review.

**Why:** Every added crate increases compile time, binary size, and supply-chain attack surface. For a developer tool that will be built from source in potentially restricted environments, a minimal dependency tree means faster builds and easier auditing. The 7 chosen crates cover the essential needs without overlap — each serves a distinct purpose that would require significant hand-written code to replace.

**Alternatives considered:**
- **`git2-rs` (libgit2 bindings)**: Would eliminate the `git` CLI dependency but adds ~2MB of native code, complicates cross-compilation, and requires `cmake` at build time.
- **`regex` crate**: Useful for commit parsing but avoidable with hand-written parsing (see Decision 4).
- **`reqwest` / `ureq`**: Not needed — the tool has no network requirements by design.

## Risks / Trade-offs

1. **Template changes require recompilation.** Because the HTML template is embedded via `include_str!` at compile time, any change to `templates/page.html` — even a CSS color tweak — requires a full `cargo build`. This is acceptable for a developer tool with infrequent template changes, but would be problematic if rapid design iteration were needed. Mitigation: the template can be previewed in a browser with mock data independently of the Rust build.

2. **Git CLI dependency.** The tool assumes `git` is installed and available on `$PATH`. While this is a safe assumption for developer machines, it means the tool cannot run in minimal container images or environments where Git is not installed. Mitigation: the error message when `git` is not found is explicit, and the alternative (bundling libgit2) would significantly increase build complexity.

3. **Fragile OpenSpec directory parsing.** The scanner relies on the `YYYY-MM-DD-slug` naming convention for archive directories and simple string parsing for `.openspec.yaml` fields. If the OpenSpec workflow changes its conventions (e.g., different date format, nested archives, or YAML structure changes), the scanner will silently skip or misparse proposals. Mitigation: the naming convention is documented in each repo's `AGENTS.md` and has been stable since project inception.

4. **No incremental or cached builds.** Every run re-scans all configured repositories from scratch by invoking `git log` on the full history. For large repositories with tens of thousands of commits, this may take several seconds. Mitigation: the tool is designed for on-demand batch use, not continuous monitoring; a few seconds of scan time is acceptable for the expected usage pattern.

5. **Line count inflation from initial scaffolding commits.** Scaffolding commits (e.g., `npm init`, `uv init`, `cargo init`, checked-in lockfiles) can contribute thousands of added lines that inflate contribution metrics. The tool does not distinguish generated or scaffolding code from hand-written code. Mitigation: users can set `since` / `until` date filters to exclude initial scaffolding periods, and the per-project breakdown helps contextualize outlier numbers.

6. **JSON injection in `<script>` context.** Commit subjects, OpenSpec descriptions, or other user-controlled fields could contain `</script>` or similar HTML-breaking content. Raw JSON from `serde_json` is not automatically safe for embedding inside an HTML `<script>` tag. Mitigation: escape `<`, `>`, `&`, U+2028, and U+2029 after JSON serialization before template injection.

7. **Git shortstat locale sensitivity.** The parser matches English substrings (`"insertion"`, `"deletion"`). If a developer's Git installation emits localized shortstat output in a non-English locale, line counts will silently become zero. Mitigation: set `LC_ALL=C` on the git subprocess to force English output.

8. **`git log --all` includes all local refs.** Results may vary across developer machines depending on local branch state (stale feature branches, remote-tracking branches, unmerged work). Mitigation: document this behavior; future enhancement could allow configuring target refs per project.

9. **Date filter inputs are not validated.** The `since` / `until` values are compared as strings. Malformed dates (e.g., `2026-5-1`, `2026/05/01`) will produce silent incorrect filtering. Mitigation: add `chrono::NaiveDate` parsing validation in a future iteration.

## Operational Assumptions

- `git` CLI is installed and available on `$PATH`. Missing git produces an explicit error.
- Missing or non-Git project paths are skipped with a warning to stderr; one failed repo does not abort the entire run.
- Relative project paths in `showcase.toml` are resolved relative to the **current working directory**, not relative to the config file location.
- The author filter (`--author` / `filters.author`) applies only to Git commits via `git log --author`. OpenSpec proposals are not author-filtered since archive directories do not contain author metadata.
- The "repos" KPI counts repositories with at least one matching commit, not the total number of configured repositories.
- `git log --author=<pattern>` uses Git's regex-like pattern matching, not plain substring matching. Simple names like "Jim" work as expected, but special regex characters in author values could behave unexpectedly.
