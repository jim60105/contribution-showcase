# Delta: HTML Report Generation

**Modifies**: `openspec/specs/html-report-generation/spec.md`

## MODIFIED Requirements

### Requirement: 8-Slide Dashboard Flow

The system SHALL render 8 dashboard slides in order: Cover (hero), KPI Dashboard (6 metric cards), Commit Trends (stacked type bar chart), Type Breakdown, Project Cards, Test Metrics, Proposals Table, and Commit Log.

#### Scenario: All slides present
- **GIVEN** a `ShowcaseData` instance with commits, proposals, projects, and test metrics
- **WHEN** the HTML report is generated
- **THEN** the output contains 8 slide sections in the specified order

### Requirement: Stacked Type Timeline Chart

The Timeline chart JavaScript SHALL read `entry.lines` (total lines changed)
for value annotations and peak identification, while continuing to use
`entry.height` (backend-normalized 0–100) for bar rendering dimensions.

Each bar SHALL be rendered as a vertically stacked column of colored segments.
Each segment represents a conventional commit type and is colored using the
corresponding CSS custom property (`--feat`, `--fix`, `--docs`, `--refactor`,
`--test`, `--chore`, `--ci`, `--build`, `--style`, `--perf`, `--other`).
Within each bar, segments SHALL be stacked in a fixed order: descending by
total lines across all buckets, with `other` always last. Each segment's height
is proportional to its type's lines relative to the bucket total, and the
overall bar height uses the existing `height` (0–100) normalization.

The peak annotation label SHALL use the format:

```
高峰：{label}（{lines.toLocaleString('zh-TW')}行）
```

#### Scenario: Timeline chart renders stacked bars

- **GIVEN** a timeline data entry with `lines: 1500`, `height: 100`, `label: "2024-W03"`, and `type_lines: {"feat": 900, "fix": 600}`
- **WHEN** the timeline chart is rendered
- **THEN** the bar height MUST be proportional to `entry.height`, the bar SHALL contain two colored segments (`feat` at 60% and `fix` at 40% of the bar height), and the peak annotation SHALL read "高峰：2024-W03（1,500行）"

#### Scenario: Segment ordering is fixed across bars

- **GIVEN** timeline data where `feat` has the most total lines across all buckets, followed by `fix`, then `docs`
- **WHEN** the stacked bars are rendered
- **THEN** within every bar, `feat` segments SHALL appear below `fix` segments, and `fix` below `docs`, regardless of per-bucket proportions

#### Scenario: Other type is always last in stack order

- **GIVEN** timeline data where `other` has more total lines than `docs`
- **WHEN** the stacked bars are rendered
- **THEN** `other` segments SHALL appear at the top of each bar (last in stacking order), above all named types

#### Scenario: Single-type bucket renders as a single segment

- **GIVEN** a timeline entry with `type_lines: {"feat": 500}` and `lines: 500`
- **WHEN** the bar is rendered
- **THEN** the bar SHALL contain a single segment colored with `var(--feat)` occupying the full bar height

#### Scenario: Empty timeline renders empty-state message

- **WHEN** the timeline data contains zero entries
- **THEN** the timeline slide SHALL display "無提交記錄"

#### Scenario: Bar segment colors match CSS custom properties

- **GIVEN** a timeline entry with `type_lines: {"feat": 100, "fix": 50}`
- **WHEN** the bar is rendered
- **THEN** the `feat` segment SHALL use `var(--feat)` (`#2f7d4a`) and the `fix` segment SHALL use `var(--fix)` (`#b53a2a`)

#### Scenario: Segment heights are proportional within a bar

- **GIVEN** a timeline entry with `height: 80`, `lines: 1000`, and `type_lines: {"feat": 700, "fix": 300}`
- **WHEN** the bar is rendered
- **THEN** the overall bar height SHALL be 80% of maximum, the `feat` segment SHALL occupy 70% of the bar height, and the `fix` segment SHALL occupy 30%

## ADDED Requirements

### Requirement: Timeline Section Renamed to Commit Trends

The timeline slide heading SHALL read "提交趨勢" (instead of "時間軸"). The navigation dot label for the timeline slide SHALL also read "提交趨勢". The `NAV_LABELS` array entry for this slide SHALL be "提交趨勢".

#### Scenario: Timeline slide heading displays new name

- **WHEN** the HTML report is generated
- **THEN** the timeline slide heading SHALL read "提交趨勢"

#### Scenario: Navigation label updated

- **WHEN** the dashboard navigation dots are rendered
- **THEN** the label for the timeline slide SHALL read "提交趨勢" instead of "時間軸"

#### Scenario: NAV_LABELS array updated

- **WHEN** inspecting the JavaScript `NAV_LABELS` array
- **THEN** the entry corresponding to the timeline slide SHALL be "提交趨勢"
