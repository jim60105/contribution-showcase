# Tasks: wiki-docs-override

Override `commit_type` to `"docs"` for all commits from projects whose name ends
with `.wiki`.

---

## 1. Backend: Type Override Logic

- [x] 1.1 In `src/collector.rs` `collect_git_commits_filtered()`, after `parse_conventional_commit()` returns, add: if `project_name.ends_with(".wiki")` then set `commit_type = "docs".to_string()`
- [x] 1.2 Verify that the `scope` field is preserved (not cleared by the override)

## 2. Backend: Unit Tests

- [x] 2.1 Add test: `.wiki` project with `feat(nav): add sidebar` → `commit_type == "docs"`, `scope == "nav"`
- [x] 2.2 Add test: `.wiki` project with non-conventional subject → `commit_type == "docs"`, `scope == ""`
- [x] 2.3 Add test: `.wiki` project with `docs: update readme` → `commit_type == "docs"` (still docs)
- [x] 2.4 Add test: non-wiki project `my-frontend` with `feat: add map` → `commit_type == "feat"` (not overridden)
- [x] 2.5 Add test: project named `my-project.wiki-tools` with `feat: add parser` → `commit_type == "feat"` (not overridden — name contains `.wiki` but does not end with it)
- [x] 2.6 Add test: project named `my-project.WIKI` with `feat: add page` → `commit_type == "feat"` (not overridden — case-sensitive match)
- [x] 2.7 Run `cargo test` — all tests pass

## 3. Build and Verification

- [x] 3.1 `cargo build --release` — successful compilation
- [x] 3.2 Run the tool — verify my-project.wiki project card shows only "文件" in top_types
- [x] 3.3 Verify Type Breakdown chart reflects increased "文件" proportion
- [x] 3.4 Screenshot my-project.wiki project card and Type Breakdown slide
