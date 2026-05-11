# Delta: git-log-collection — lines-charts-and-type-floor

## ADDED Requirements

### Requirement: build_timeline lines accumulation

The `build_timeline()` function SHALL aggregate lines changed rather than commit counts.

#### Scenario: Timeline entry accumulates insertions and deletions

- Given a set of commits grouped by ISO week
- When `build_timeline()` processes each commit
- Then it MUST accumulate `commit.insertions + commit.deletions` into the weekly total
- And the resulting `TimelineEntry.lines` field SHALL hold the sum of insertions and deletions for that week

#### Scenario: Timeline height normalization

- Given the collection of `TimelineEntry` values produced by `build_timeline()`
- When computing `TimelineEntry.height`
- Then the system SHALL normalize as `lines / max_lines * 100` where `max_lines` is the maximum `lines` value across all entries

#### Scenario: All-zero timeline entries

- Given all commits in the filtered dataset have `insertions = 0` and `deletions = 0`
- When `build_timeline()` computes height normalisation
- Then all `TimelineEntry.height` values SHALL be `0.0` (no division by zero)

### Requirement: build_type_breakdown lines accumulation

The `build_type_breakdown()` function SHALL aggregate lines changed rather than commit counts.
Results SHALL be sorted in descending order by `lines`.

#### Scenario: Type breakdown accumulates insertions and deletions

- Given a set of commits grouped by conventional commit type
- When `build_type_breakdown()` processes each commit
- Then it MUST accumulate `commit.insertions + commit.deletions` into the per-type total
- And the resulting `TypeBreakdown.lines` field SHALL hold the sum of insertions and deletions for that type

#### Scenario: Type breakdown percentage calculation

- Given the collection of `TypeBreakdown` values produced by `build_type_breakdown()`
- When computing `TypeBreakdown.percentage`
- Then the system SHALL calculate `type_lines / total_lines * 100` where `total_lines` is the sum of all `TypeBreakdown.lines` values

#### Scenario: All-zero type breakdown entries

- Given all commits have `insertions = 0` and `deletions = 0`
- When `build_type_breakdown()` computes percentages
- Then all `TypeBreakdown.percentage` values SHALL be `0.0` (no division by zero)

#### Scenario: Type breakdown ordering

- Given multiple commit types with different line totals
- When `build_type_breakdown()` produces the result vector
- Then entries SHALL be sorted in descending order by `lines`

### Requirement: Project top_types lines semantics

The `build_project_data()` function SHALL compute per-project `top_types`
using lines changed (not commit count). Each project's `top_types` vector
SHALL be sorted by descending `lines` and truncated to at most 5 entries.

#### Scenario: Project top types use lines

- Given a project with commits of types feat (200 lines), fix (500 lines), docs (100 lines)
- When `build_project_data()` produces `top_types`
- Then the entries SHALL be ordered [fix, feat, docs] by descending `lines`

### Requirement: Model field renaming

#### Scenario: TimelineEntry uses lines field

- The `TimelineEntry` struct MUST expose a field named `lines` of type `usize`
- The previous `count` field SHALL NOT exist

#### Scenario: TypeBreakdown uses lines field

- The `TypeBreakdown` struct MUST expose a field named `lines` of type `usize`
- The previous `count` field SHALL NOT exist
