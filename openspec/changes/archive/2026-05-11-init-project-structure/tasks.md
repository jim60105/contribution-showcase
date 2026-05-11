# Tasks: init-project-structure

## 1. Project Scaffolding

- [x] Create `Cargo.toml` with package name `contribution-showcase`, edition 2021, version 0.1.0
- [x] Add dependencies: `clap` 4 (feature `derive`), `serde` 1 (feature `derive`), `serde_json` 1, `toml` 0.8, `chrono` 0.4 (feature `serde`), `walkdir` 2, `anyhow` 1
- [x] Create `.gitignore` excluding `/target` and `/dist`
- [x] Initialize git repository

## 2. Configuration Module (`src/config.rs`)

- [x] Define `Config` struct with fields: `title` (`Option<String>`), `output` (`Option<OutputConfig>`), `projects` (`Vec<ProjectConfig>`), `filters` (`Option<FilterConfig>`); derive `Debug`, `Deserialize`
- [x] Define `OutputConfig` struct with field `path` (`Option<String>`); derive `Debug`, `Deserialize`
- [x] Define `ProjectConfig` struct with fields `name` (`String`), `path` (`String`), `description` (`Option<String>`); derive `Debug`, `Deserialize`
- [x] Define `FilterConfig` struct with fields `author`, `since`, `until` (all `Option<String>`), `types` (`Option<Vec<String>>`); derive `Debug`, `Deserialize`, `Default`
- [x] Implement `Config::load()` reading a TOML file via `std::fs::read_to_string` + `toml::from_str`, returning `anyhow::Result<Config>`
- [x] Implement `Config::output_path()` returning the configured path or default `"dist/index.html"`
- [x] Implement `Config::filters()` accessor that clones filter values with `Default` fallback

## 3. Data Model (`src/model.rs`)

- [x] Define `ShowcaseData` struct with fields: `title`, `author`, `date_range`, `generated_at` (`String`), `summary` (`Summary`), `timeline` (`Vec<TimelineEntry>`), `type_breakdown` (`Vec<TypeBreakdown>`), `projects` (`Vec<ProjectData>`), `proposals` (`Vec<ProposalEntry>`), `commits` (`Vec<CommitEntry>`); derive `Debug`, `Serialize`
- [x] Define `Summary` struct with fields: `total_commits`, `total_repos`, `total_proposals`, `lines_added`, `lines_removed` (all `usize`); derive `Debug`, `Serialize`
- [x] Define `TimelineEntry` struct with fields: `label` (`String`), `count` (`usize`), `height` (`f64`); derive `Debug`, `Serialize`
- [x] Define `TypeBreakdown` struct with fields: `commit_type` (`String`), `label` (`String`), `count` (`usize`), `percentage` (`f64`); derive `Debug`, `Serialize`
- [x] Define `ProjectData` struct with fields: `name`, `description` (`String`), `commit_count`, `proposal_count`, `lines_added`, `lines_removed` (`usize`), `top_types` (`Vec<TypeBreakdown>`); derive `Debug`, `Serialize`
- [x] Define `CommitEntry` struct with fields: `hash`, `date`, `commit_type`, `scope`, `subject`, `project` (`String`), `insertions`, `deletions` (`usize`); derive `Debug`, `Serialize`, `Clone`
- [x] Define `ProposalEntry` struct with fields: `slug`, `date`, `project`, `description` (`String`), `task_count` (`usize`); derive `Debug`, `Serialize`, `Clone`

## 4. Collector Module (`src/collector.rs`)

- [x] Implement `type_label()` mapping 10 conventional commit types to zh-TW labels: feat→新功能, fix→錯誤修復, docs→文件, refactor→重構, test→測試, chore→維護, ci→CI/CD, build→建構, style→格式, perf→效能, fallback→其他
- [x] Implement `parse_conventional_commit()` byte-level parser extracting type and optional scope from `type(scope):` pattern, returning `("other", "")` for non-matching subjects
- [x] Implement `collect_git_commits_filtered()` running `git log --all --format='COMMIT_DELIM%H|||%aN|||%aI|||%s' --shortstat` with optional `--author` flag, parsing delimited output into `Vec<CommitEntry>`
- [x] Implement `collect_proposals()` scanning `openspec/changes/archive/` directories via `std::fs::read_dir`, parsing `YYYY-MM-DD-slug` directory names, counting `- [x]` lines in `tasks.md`, reading description from `.openspec.yaml`
- [x] Implement `apply_filters()` filtering commits by `since`/`until` dates and commit types, filtering proposals by `since`/`until` dates
- [x] Implement `build_timeline()` aggregating commits into ISO week buckets (format `%G-W%V`), calculating relative bar heights as `count / max_count`
- [x] Implement `build_type_breakdown()` grouping by zh-TW label (via `type_label()`) to avoid duplicate "其他" entries, calculating percentages as `count / total * 100`
- [x] Implement `build_project_data()` computing per-repo statistics: commit count, proposal count, lines added/removed, top 5 commit types by count
- [x] Implement `collect()` main entry point orchestrating: per-project collection → filter application → chronological sorting → summary computation → date range calculation → timeline/breakdown building → `ShowcaseData` assembly

## 5. CLI Entry Point (`src/main.rs`)

- [x] Define `Cli` struct with `clap` derive: `--config` (default `"showcase.toml"`), `--output` (optional), `--author` (optional), `--since` (optional), `--until` (optional)
- [x] Implement config loading via `Config::load()` with CLI argument override merging for `author`, `since`, `until`, and `output` fields
- [x] Implement template injection: embed `templates/page.html` via `include_str!("../templates/page.html")`, serialize `ShowcaseData` to JSON, replace placeholder `"__SHOWCASE_DATA__"` in template with JSON data
- [x] Implement output file writing with parent directory creation via `std::fs::create_dir_all`
- [x] Add progress output to stderr: scanning message, summary line (commit/repo/proposal counts), output path

## 6. HTML Template (`templates/page.html`)

- [x] Create ~835-line self-contained HTML file with `lang="zh-TW"`, zero external dependencies (no CDN fonts, no external CSS/JS, no network requests)
- [x] Implement Soft Paper CSS theme with custom properties: `--bg: #f2f2f0`, `--surface: #fff`, `--ink: #0a0a0a`, `--secondary: #6b6b6b`
- [x] Define commit type color variables: `--feat: #2f7d4a`, `--fix: #b53a2a`, `--docs: #6366f1`, `--refactor: #d97706`, `--test: #0891b2`, `--chore: #6b7280`, `--ci: #7c3aed`, `--build: #ea580c`, `--style: #db2777`, `--perf: #059669`, `--other: #9a9a95`
- [x] Implement Hero section renderer displaying title, subtitle (author + date range), and generation timestamp
- [x] Implement KPI strip with 5 metric cards using CSS Grid (5 columns), formatted via `toLocaleString('zh-TW')`
- [x] Implement Timeline chart renderer with CSS flex bars, weekly aggregation, and bar labels on hover
- [x] Implement Type breakdown renderer with horizontal bars, percentage labels, and commit type color coding
- [x] Implement Project cards renderer with commit/proposal counts, line stats (added/removed), and top commit type badges
- [x] Implement Proposals table grouped by project, showing slug, date, description, and task count columns
- [x] Implement sortable Commit log table with type badges, initially showing 100 rows with toggle to show all
- [x] Add responsive breakpoints: 900px (charts stack vertically), 768px (KPI 3+2 grid), 480px (KPI single column)

## 7. Default Configuration (`showcase.toml`)

- [x] Configure 7 VMS repositories with relative paths and zh-TW descriptions: VMS-Platform-Core (後端整合平台), VMS-Frontend (監控儀表板前端), VMS-Custom-Function-Bridge (AI 分析橋接器), VMS-Template-AI-Module (AI 模組範本（上游）), VMS-Template-Downstream-Module (AI 模組範本（下游）), VMS (部署編排與腳本), VMS.wiki (專案文件與 Wiki)
- [x] Set author filter to `"Jim"` (passed to `git log --author=Jim`, which Git interprets as an author pattern matching both "Jim Chen" and "陳鈞 Jim")
- [x] Set output path to `dist/index.html`

## 8. Build and Verification

- [x] Build with `cargo build --release` — successful compilation, no warnings
- [x] Run test generation against workspace: produced 451 commits across 7 repos, 105 OpenSpec proposals
- [x] Fix `build_type_breakdown()` grouping bug — duplicate "其他" entries caused by grouping on raw `commit_type` instead of zh-TW `label`; changed grouping key to `label`
- [x] Verify generated `dist/index.html` (~66KB) renders correctly with all sections: Hero, KPI strip, Timeline, Type breakdown, Project cards, Proposals table, Commit log
