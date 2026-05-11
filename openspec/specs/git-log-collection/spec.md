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
