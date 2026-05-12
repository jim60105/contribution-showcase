## Why

Projects whose name ends in `.wiki` are pure documentation repositories (e.g.
`my-project.wiki`). Their commits overwhelmingly describe documentation work, yet
`parse_conventional_commit()` still classifies them by their subject-line prefix
— which in a wiki repo is inconsistent or absent, causing many commits to land
in the "其他" (other) bucket. This distorts the Type Breakdown chart and
per-project `top_types`, under-representing the "文件" (docs) category.

Overriding the commit type to `"docs"` for `.wiki` projects produces an accurate
picture of contribution effort.

## What Changes

- **Type override for `.wiki` projects**: After parsing each commit, if the
  project name ends with `.wiki`, set `commit_type` to `"docs"` regardless of
  the parsed conventional-commit prefix.
- **Scope preservation**: The original scope (if any) is kept, only the type
  is overridden.
- **Test coverage**: Unit tests confirm the override fires for `.wiki` suffix
  and does not fire for non-wiki project names.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `git-log-collection`: Add a post-parse type-override rule that forces
  `commit_type = "docs"` when the project name ends with `.wiki`.

## Impact

- **`src/collector.rs`** — `collect_git_commits_filtered()` gains a one-line
  override after `parse_conventional_commit()`. Existing tests unaffected; new
  tests added for the override behaviour.
- **No template changes** — the HTML template already renders whatever
  `commit_type` the backend provides.
- **No config changes** — the override is convention-based (`.wiki` suffix), not
  a new config field.
