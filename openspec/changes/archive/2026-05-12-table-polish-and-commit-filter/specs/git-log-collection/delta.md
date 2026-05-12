# Git Log Collection — Delta

## ADDED Requirements

### Requirement: Commit Hash Exclusion

After collecting commits, the system SHALL exclude any commit whose `hash` is present in the `exclude_hashes` set. The exclusion SHALL occur before any aggregation steps (timeline, type breakdown, project data, and summary counts).

#### Scenario: Commit hash is in the exclude list

- **GIVEN** a collected commit with hash `"dd33ee63950bb49a284de835528343561f1a70d5"` and an `exclude_hashes` set containing that hash
- **WHEN** exclusion filtering runs
- **THEN** that commit is removed and does not appear in any aggregation output (timeline, type breakdown, project data, or summary counts)

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
