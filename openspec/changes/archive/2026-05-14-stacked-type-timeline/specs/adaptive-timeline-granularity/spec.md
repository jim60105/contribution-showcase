# Delta: Adaptive Timeline Granularity

**Modifies**: `openspec/specs/adaptive-timeline-granularity/spec.md`

## ADDED Requirements

### Requirement: Per-Type Lines Breakdown in Timeline Entries

Each `TimelineEntry` SHALL include a `type_lines` field (a map from commit type string to `usize`) recording lines changed per conventional commit type within that bucket. The `lines` field remains the aggregate total. Types with zero lines in a bucket SHALL be omitted from the map. The `type_lines` values SHALL sum to `lines`.

#### Scenario: Bucket with multiple commit types records per-type lines

- **WHEN** a weekly bucket contains commits of type `feat` adding 300 lines and type `fix` adding 120 lines
- **THEN** the `TimelineEntry` SHALL have `lines: 420`, `type_lines: {"feat": 300, "fix": 120}`, and `type_lines` values SHALL sum to 420

#### Scenario: Bucket with a single commit type

- **WHEN** a daily bucket contains only `docs` commits totalling 50 lines
- **THEN** the `TimelineEntry` SHALL have `lines: 50` and `type_lines: {"docs": 50}`

#### Scenario: Zero-line types are omitted from the map

- **WHEN** a bucket contains `feat` commits with 200 lines and no `fix` commits
- **THEN** the `type_lines` map SHALL contain `{"feat": 200}` and SHALL NOT contain a `"fix"` key

#### Scenario: Empty bucket has an empty type_lines map

- **WHEN** a contiguous gap bucket is generated with zero commits
- **THEN** the `TimelineEntry` SHALL have `lines: 0` and `type_lines` SHALL be an empty map
