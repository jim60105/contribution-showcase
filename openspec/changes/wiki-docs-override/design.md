## Context

The `contribution-showcase` CLI collects git commit history from multiple
repositories configured in `showcase.toml`. Each commit's subject line is parsed
by `parse_conventional_commit()` to extract a commit type (feat, fix, docs,
etc.). Non-matching subjects fall back to `"other"`.

One of the configured projects is `VMS.wiki` — the Azure DevOps wiki repository
containing authoritative design and requirements documents. Its commits are
purely documentation work but do not consistently use the `docs:` prefix.

Currently, many VMS.wiki commits end up classified as "other", which:
1. Inflates the "其他" slice in the Type Breakdown chart.
2. Under-reports documentation effort in global and per-project views.
3. Creates misleading `top_types` on the VMS.wiki project card.

## Goals / Non-Goals

**Goals:**
- G1: All commits from projects whose name ends with `.wiki` are classified as
  `commit_type = "docs"`, regardless of the parsed conventional-commit type.
- G2: The override preserves the original `scope` field (only the type changes).
- G3: Unit tests verify the override fires for `.wiki` projects and does not
  fire for other projects.

**Non-Goals:**
- NG1: A generic per-project type-override config field (too heavy for a
  single convention).
- NG2: Changing the proposal scanner behaviour (proposals are already
  project-scoped and unaffected by commit type).
- NG3: Retroactively fixing existing test data — tests use fresh mock data.

## Decisions

### D1 — Convention-based override, not config-driven

Override is keyed to the `.wiki` project-name suffix. A config field like
`override_type = "docs"` was considered but rejected: only wiki repos need
this treatment, and a naming convention avoids schema changes.

**Rationale**: YAGNI. If a second override pattern emerges later, promote to
config then.

### D2 — Override point: inside `collect_git_commits_filtered()`

The override is applied immediately after `parse_conventional_commit()` returns,
inside the commit-parsing loop. This keeps the logic co-located with type
extraction and ensures all downstream consumers (timeline, breakdown, project
data) see the corrected type.

**Alternative rejected**: A post-collection `filter_map` pass — adds an extra
loop and separates the override from its cause.

### D3 — Scope preservation

The `scope` field parsed from the commit subject is preserved even when the
type is overridden. For example, `feat(auth): add login` on a `.wiki` project
becomes `commit_type = "docs"`, `scope = "auth"`. The scope has no downstream
effect today but preserving it avoids information loss.

### D4 — Case-sensitive suffix match

The `.wiki` suffix check uses a case-sensitive `ends_with(".wiki")`. Azure
DevOps wiki repos consistently use lowercase `.wiki`; accepting uppercase
variants (`.WIKI`, `.Wiki`) would widen the blast radius without benefit.

**Rationale**: The convention is well-defined; case-insensitive matching would
risk false positives on hypothetical project names that happen to end in an
uppercase variant.

## Risks / Trade-offs

### R1 — False positives on non-wiki `.wiki` projects (Low)

Any project whose name ends in `.wiki` triggers the override. In practice, only
Azure DevOps wiki repos follow this naming pattern. If a non-wiki project
accidentally matches, its types would be incorrectly overridden.

**Mitigation**: The convention is well-established in the Azure DevOps ecosystem;
the risk is negligible for this workspace.
