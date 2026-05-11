# OpenSpec Archive Scanner

## Purpose

Scans the `openspec/changes/archive/` directory tree to discover completed
OpenSpec proposals, extracts metadata (date, slug, task completion count,
description) from each archived proposal directory, and returns a sorted
collection for inclusion in the contribution showcase report.

## Requirements

### Requirement: Directory Scanning

The system SHALL scan `openspec/changes/archive/` using `std::fs::read_dir`
to discover archived proposal directories.

#### Scenario: Archive directory contains proposals
- **GIVEN** the `openspec/changes/archive/` directory exists and contains
  subdirectories
- **WHEN** the scanner is invoked
- **THEN** each immediate subdirectory is evaluated as a potential proposal entry

#### Scenario: Archive directory does not exist
- **GIVEN** the `openspec/changes/archive/` directory does not exist
- **WHEN** the scanner is invoked
- **THEN** the scanner returns an empty `Vec<ProposalEntry>` without error

### Requirement: Date and Slug Parsing

The system SHALL parse directory names matching the `YYYY-MM-DD-slug` pattern
to extract a date and a slug for each proposal.

#### Scenario: Valid directory name
- **GIVEN** a subdirectory named `2025-06-15-add-feature`
- **WHEN** the scanner parses the directory name
- **THEN** the date `2025-06-15` and slug `add-feature` are extracted

#### Scenario: Directory name does not match pattern
- **GIVEN** a subdirectory whose name does not match `YYYY-MM-DD-slug`
- **WHEN** the scanner parses the directory name
- **THEN** the directory is silently skipped

### Requirement: Task Counting

The system SHALL count completed tasks by matching `- [x]` checkboxes in the
proposal's `tasks.md` file.

#### Scenario: tasks.md with completed tasks
- **GIVEN** a proposal directory containing a `tasks.md` with three `- [x]`
  lines and two `- [ ]` lines
- **WHEN** the scanner counts completed tasks
- **THEN** the completed task count is `3`

#### Scenario: tasks.md missing
- **GIVEN** a proposal directory without a `tasks.md` file
- **WHEN** the scanner attempts to count tasks
- **THEN** the completed task count defaults to `0`

### Requirement: Description Reading

The system SHALL read the proposal description from the `description` field
in the proposal's `.openspec.yaml` file.

#### Scenario: .openspec.yaml contains a description
- **GIVEN** a proposal directory with a `.openspec.yaml` that has a
  `description` field
- **WHEN** the scanner reads the description
- **THEN** the description value from the YAML file is used

#### Scenario: .openspec.yaml missing or lacks description
- **GIVEN** a proposal directory without a `.openspec.yaml` or with a YAML
  file that has no `description` field
- **WHEN** the scanner reads the description
- **THEN** the description defaults to an empty string

### Requirement: Sort Order

The system SHALL return `Vec<ProposalEntry>` sorted by date in descending
order (most recent first).

#### Scenario: Multiple proposals with different dates
- **GIVEN** archived proposals dated `2025-01-10`, `2025-06-15`, and
  `2025-03-22`
- **WHEN** the scanner returns results
- **THEN** the entries are ordered `2025-06-15`, `2025-03-22`, `2025-01-10`

### Requirement: Author Filter Non-Applicability

The system SHALL NOT apply author-based filtering to proposals. Proposals are
always included regardless of any active author filter.

#### Scenario: Author filter is active
- **GIVEN** an author filter is configured to show only contributions by a
  specific author
- **WHEN** the scanner collects proposals
- **THEN** all archived proposals are included in the results, ignoring the
  author filter
