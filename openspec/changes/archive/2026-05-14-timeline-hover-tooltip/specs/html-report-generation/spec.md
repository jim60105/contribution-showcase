## MODIFIED Requirements

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

When the user hovers the cursor over a bar, a tooltip SHALL appear displaying the bucket label, total lines, and a per-type breakdown of lines changed (from `type_lines`). Each type row SHALL show a colored indicator matching the type's CSS custom property, the type name, and the line count formatted with `toLocaleString('zh-TW')` followed by `行`. Type rows SHALL be ordered consistently with the bar segment stacking order (descending by total lines, `other` last). The tooltip SHALL disappear when the cursor leaves the bar.

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

#### Scenario: Tooltip displays per-type lines on hover

- **WHEN** the user hovers over a bar with `label: "2024-W03"`, `lines: 1500`, and `type_lines: {"feat": 900, "fix": 600}`
- **THEN** a tooltip SHALL appear showing "2024-W03（1,500行）" followed by rows "feat: 900行" and "fix: 600行" with colored indicators

#### Scenario: Tooltip type rows use consistent ordering

- **WHEN** the user hovers over a bar and the global type order is [feat, fix, docs, other]
- **THEN** the tooltip rows SHALL appear in the same order: feat, fix, docs, other

#### Scenario: Tooltip colored indicators match bar segment colors

- **WHEN** the tooltip displays a row for type `feat`
- **THEN** the colored indicator SHALL use the CSS custom property `var(--feat)`

#### Scenario: Tooltip disappears on mouse leave

- **WHEN** the user moves the cursor away from a bar
- **THEN** the tooltip SHALL be hidden

#### Scenario: Empty bucket tooltip shows only label and zero lines

- **WHEN** the user hovers over a gap bucket with `lines: 0` and empty `type_lines`
- **THEN** the tooltip SHALL show "{label}（0行）" with no type rows

#### Scenario: Tooltip is positioned above the hovered bar

- **WHEN** the user hovers over a bar
- **THEN** the tooltip SHALL be positioned centered above the bar, clamped to prevent overflow outside the chart container
