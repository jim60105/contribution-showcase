# Tasks: wiki-docs-override

Override `commit_type` to `"docs"` for all commits from projects whose name ends
with `.wiki`.

---

## 1. Backend: Type Override Logic

- [ ] 1.1 In `src/collector.rs` `collect_git_commits_filtered()`, after `parse_conventional_commit()` returns, add: if `project_name.ends_with(".wiki")` then set `commit_type = "docs".to_string()`
- [ ] 1.2 Verify that the `scope` field is preserved (not cleared by the override)

## 2. Backend: Unit Tests

- [ ] 2.1 Add test: `.wiki` project with `feat(nav): add sidebar` → `commit_type == "docs"`, `scope == "nav"`
- [ ] 2.2 Add test: `.wiki` project with non-conventional subject → `commit_type == "docs"`, `scope == ""`
- [ ] 2.3 Add test: `.wiki` project with `docs: update readme` → `commit_type == "docs"` (still docs)
- [ ] 2.4 Add test: non-wiki project `VMS-Frontend` with `feat: add map` → `commit_type == "feat"` (not overridden)
- [ ] 2.5 Add test: project named `VMS.wiki-tools` with `feat: add parser` → `commit_type == "feat"` (not overridden — name contains `.wiki` but does not end with it)
- [ ] 2.6 Add test: project named `VMS.WIKI` with `feat: add page` → `commit_type == "feat"` (not overridden — case-sensitive match)
- [ ] 2.7 Run `cargo test` — all tests pass

## 3. Build and Verification

- [ ] 3.1 `cargo build --release` — successful compilation
- [ ] 3.2 Run the tool — verify VMS.wiki project card shows only "文件" in top_types
- [ ] 3.3 Verify Type Breakdown chart reflects increased "文件" proportion
- [ ] 3.4 Screenshot VMS.wiki project card and Type Breakdown slide
