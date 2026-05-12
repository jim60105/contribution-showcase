# Git Log Collection

## Purpose

Collects commit history from Git repositories by invoking the `git log`
subprocess. Produces structured records containing hash, author, date, subject,
and change-size statistics for downstream aggregation and reporting.

## Requirements

### Requirement: Git Log Invocation Format

The system SHALL invoke `git log` with the output format
`COMMIT_DELIM%H|||%aN|||%aI|||%s` combined with `--shortstat` so that each
commit produces a predictable, machine-parseable block.

#### Scenario: Standard invocation without filters
- **GIVEN** a valid Git repository path
- **WHEN** the system collects commit history with no author filter and no branch configured
- **THEN** `git log` is invoked with `--all`, the specified format string, and `--shortstat`

### Requirement: Author Filtering

The system SHALL pass an `--author` flag to `git log` when an author filter is
provided, restricting output to commits whose author matches the filter.

#### Scenario: Author filter supplied
- **GIVEN** a valid Git repository and an author filter value `"Alice"`
- **WHEN** the system collects commit history
- **THEN** `git log` is invoked with `--author=Alice`

#### Scenario: No author filter
- **GIVEN** a valid Git repository and no author filter
- **WHEN** the system collects commit history
- **THEN** `git log` is invoked without any `--author` flag

### Requirement: Locale Pinning

The system SHALL set `LC_ALL=C` in the environment of the `git log` subprocess
so that `--shortstat` output uses locale-independent English wording
(`insertions`, `deletions`) regardless of the host locale.

#### Scenario: Host locale is non-English
- **GIVEN** the host `LC_ALL` is set to `zh_TW.UTF-8`
- **WHEN** the system spawns `git log`
- **THEN** the subprocess environment contains `LC_ALL=C` and shortstat output uses English tokens

### Requirement: Configurable Ref Scope

The system SHALL accept an optional `branch` field per project. When `branch`
is set, it is passed as the revision argument to `git log` instead of `--all`.
When `branch` is absent, the system defaults to `--all`.

#### Scenario: Branch field configured
- **GIVEN** a project configuration with `branch` set to `"main"`
- **WHEN** the system collects commit history
- **THEN** `git log` is invoked with `main` as the revision argument and without `--all`

#### Scenario: Branch field absent
- **GIVEN** a project configuration with no `branch` field
- **WHEN** the system collects commit history
- **THEN** `git log` is invoked with `--all`

### Requirement: Shortstat Parsing

The system SHALL parse the `--shortstat` line to extract the number of
insertions and deletions. It must handle lines that contain only insertions,
only deletions, or both.

#### Scenario: Both insertions and deletions present
- **GIVEN** a shortstat line `" 3 files changed, 42 insertions(+), 7 deletions(-)"`
- **WHEN** the system parses the line
- **THEN** insertions is `42` and deletions is `7`

#### Scenario: Insertions only
- **GIVEN** a shortstat line `" 1 file changed, 10 insertions(+)"`
- **WHEN** the system parses the line
- **THEN** insertions is `10` and deletions is `0`

#### Scenario: Deletions only
- **GIVEN** a shortstat line `" 2 files changed, 5 deletions(-)"`
- **WHEN** the system parses the line
- **THEN** insertions is `0` and deletions is `5`

### Requirement: Unique Commit Date Collection

After collecting and filtering commits, the system SHALL count the number of
distinct calendar dates (YYYY-MM-DD) on which at least one commit was authored.

#### Scenario: Commits span multiple dates
- **GIVEN** filtered commits on dates 2025-01-01, 2025-01-01, 2025-01-03
- **WHEN** unique dates are counted
- **THEN** the count is 2

### Requirement: Average Daily Lines Computation

The system SHALL compute the average daily lines changed as
`(total_insertions + total_deletions) / unique_date_count`, guarding against
division by zero (returning `0.0` when no commits exist).

#### Scenario: Normal computation
- **GIVEN** 2 unique dates, total insertions = 100, total deletions = 50
- **WHEN** the average is computed
- **THEN** `avg_daily_lines` equals `75.0`

#### Scenario: No commits after filtering
- **GIVEN** 0 commits after applying filters
- **WHEN** the average is computed
- **THEN** `avg_daily_lines` equals `0.0`

### Requirement: Non-Git Directory Handling

The system SHALL gracefully skip directories that are not Git repositories and
emit a warning rather than terminating.

#### Scenario: Path is not a Git repository
- **GIVEN** a configured project path that does not contain a `.git` directory
- **WHEN** the system attempts to collect commit history
- **THEN** a warning is emitted and the project is skipped without aborting the run

### Requirement: Error Message Context

The system SHALL include the project name and the ref description (branch name
or `--all`) in any error message produced during git log collection.

#### Scenario: Git log fails for a specific project and branch
- **GIVEN** a project named `"platform-core"` with `branch` set to `"release/2.0"`
- **WHEN** `git log` exits with a non-zero status
- **THEN** the error message contains `"platform-core"` and `"release/2.0"`

#### Scenario: Git log fails with --all
- **GIVEN** a project named `"frontend"` with no branch configured
- **WHEN** `git log` exits with a non-zero status
- **THEN** the error message contains `"frontend"` and `"--all"`

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

### Requirement: Wiki Project Type Override

After parsing the conventional-commit type from a commit subject, the system
SHALL override `commit_type` to `"docs"` when the project name ends with
`.wiki`. The original `scope` field SHALL be preserved.

#### Scenario: Commit in a .wiki project with a non-docs prefix

- **GIVEN** a project named `"my-project.wiki"` and a commit subject `"feat(nav): add sidebar"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"docs"` and `scope` is `"nav"`

#### Scenario: Commit in a .wiki project with no conventional prefix

- **GIVEN** a project named `"my-project.wiki"` and a commit subject `"Update architecture page"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"docs"` and `scope` is `""`

#### Scenario: Commit in a .wiki project that is already docs

- **GIVEN** a project named `"my-project.wiki"` and a commit subject `"docs: update readme"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"docs"` (unchanged) and `scope` is `""`

#### Scenario: Commit in a non-wiki project is not overridden

- **GIVEN** a project named `"my-frontend"` and a commit subject `"feat: add map"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"feat"` (not overridden)

#### Scenario: Project name that contains .wiki but does not end with it

- **GIVEN** a project named `"my-project.wiki-tools"` and a commit subject `"feat: add parser"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"feat"` (not overridden)

#### Scenario: Case-sensitive suffix match

- **GIVEN** a project named `"my-project.WIKI"` and a commit subject `"feat: add page"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"feat"` (not overridden — match is case-sensitive)

### Requirement: Commit Hash Exclusion

After collecting commits, the system SHALL exclude any commit whose `hash` is present in the `exclude_hashes` set. The exclusion SHALL occur before any aggregation steps (timeline, type breakdown, project data, and summary counts).

#### Scenario: Commit hash is in the exclude list
- **GIVEN** a collected commit with hash `"dd33ee63950bb49a284de835528343561f1a70d5"` and an `exclude_hashes` set containing that hash
- **WHEN** exclusion filtering runs
- **THEN** that commit is removed and does not appear in any aggregation output

#### Scenario: Exclude list is empty
- **GIVEN** a set of collected commits and an empty `exclude_hashes` set
- **WHEN** exclusion filtering runs
- **THEN** all commits are retained and included in aggregation

#### Scenario: Exclude list contains a hash not present in commits
- **GIVEN** a set of collected commits and an `exclude_hashes` set containing `"0000000000000000000000000000000000000000"` which does not match any commit
- **WHEN** exclusion filtering runs
- **THEN** all commits are retained and no error is raised

#### Scenario: Exclude list contains a short or malformed hash
- **GIVEN** a collected commit with hash `"dd33ee63950bb49a284de835528343561f1a70d5"` and an `exclude_hashes` set containing only `"dd33ee63"` (a short prefix)
- **WHEN** exclusion filtering runs
- **THEN** the commit is NOT excluded (exact full-string match is required) and no error is raised
