# Delta: html-report-generation — lines-charts-and-type-floor

## MODIFIED Requirements

### Requirement: Typography Craft System

The report SHALL enforce a minimum font size floor of 18px across all text
elements, with a proportionally scaled hierarchy. Headlines use negative
letter-spacing and a multi-weight hierarchy (400 body / 500–600 labels / 700
headlines).

The CSS custom properties SHALL be defined as follows:

| Variable | Value |
|---|---|
| `--fs-xs` | 18px |
| `--fs-small` | 20px |
| `--fs-body` | 22px |
| `--fs-h2` | clamp(28px, 3vw, 40px) |
| `--fs-h1` | clamp(36px, 4vw, 52px) |
| `--fs-display` | clamp(52px, 6vw, 84px) |

The following component-level font sizes SHALL apply:

| Selector | Value |
|---|---|
| `.metric` | clamp(40px, 5vw, 60px) |
| `.kpi-value` | clamp(32px, 4vw, 48px) |
| `.hero-metric .metric` | clamp(60px, 8vw, 100px) |
| `.sort-arrow` | 18px |
| `.scroll-indicator` | 28px |

#### Scenario: No text element renders below 18px

Given the report is rendered at any viewport width,
then no visible text element SHALL have a computed font-size below 18px.

#### Scenario: Proportional hierarchy is preserved

Given the typography scale,
then `--fs-xs` < `--fs-small` < `--fs-body` < `--fs-h2` (min) < `--fs-h1` (min) < `--fs-display` (min) MUST hold.

---

## ADDED Requirements

### Requirement: Lines-Based Timeline Chart

The Timeline chart JavaScript SHALL read `entry.lines` (total lines changed)
for value annotations and peak identification, while continuing to use
`entry.height` (backend-normalized 0–100) for bar rendering dimensions.

The peak annotation label SHALL use the format:

```
高峰：{label}（{lines.toLocaleString('zh-TW')}行）
```

#### Scenario: Timeline chart renders lines changed

Given a timeline data entry with `lines: 1500`, `height: 100`, and `label: "2024-W03"`,
when the timeline chart is rendered,
then the bar height MUST be proportional to `entry.height`
and the peak annotation SHALL read "高峰：2024-W03（1,500行）".

### Requirement: Lines-Based Type Breakdown Chart

The Type Breakdown chart JavaScript SHALL read `entry.lines` (total lines
changed) for value labels, while continuing to use `entry.percentage`
(backend-normalized) for bar width rendering.

The breakdown label SHALL use the format:

```
{label}（{lines.toLocaleString('zh-TW')}行）
```

#### Scenario: Type breakdown chart renders lines changed

Given a type breakdown entry with `label: "feat"`, `lines: 12345`, and `percentage: 45.2`,
when the breakdown chart is rendered,
then the bar width MUST be proportional to `entry.percentage`
and the label SHALL read "feat（12,345行）".

#### Scenario: Breakdown unit consistency

Given the type breakdown chart is rendered,
then all segment labels SHALL use "行" as the unit suffix
and values MUST be formatted with `toLocaleString('zh-TW')`.
