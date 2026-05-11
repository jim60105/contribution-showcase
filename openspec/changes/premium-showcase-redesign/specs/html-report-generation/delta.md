# html-report-generation — Delta

## Added Requirements

### Requirement: Editorial Paper Design System

The system SHALL apply the "Editorial Paper" visual theme with warm paper
background `#f8f7f4`, surface color `#ffffff`, foreground `#1a1916`, muted
text `#6b6964`, border `#e8e5df`, and accent `#2563eb`. The theme uses serif
fonts for display headings and sans-serif for body text.

**Supersedes**: "Soft Paper Theme" requirement (canvas `#f2f2f0`, surface
`#fff`, ink `#0a0a0a`). The new palette is warmer and introduces structured
design tokens including accent, muted, and border roles.

#### Scenario: Theme colours applied
- **GIVEN** the generated HTML file
- **WHEN** inspecting the CSS `:root` custom properties
- **THEN** `--bg` is `#f8f7f4`, `--surface` is `#ffffff`, `--fg` is
  `#1a1916`, `--muted` is `#6b6964`, `--border` is `#e8e5df`, and
  `--accent` is `#2563eb`

### Requirement: Slide-Based Section Layout

The system SHALL render dashboard sections as full-viewport slides using CSS
scroll-snap, replacing the previous continuous scroll layout.

**Supersedes**: The implicit continuous scroll layout from the previous
"Dashboard Sections" requirement. Sections are now slides, not stacked cards.

#### Scenario: Dashboard sections render as slides
- **GIVEN** the generated HTML file
- **WHEN** opened in a desktop browser
- **THEN** each section occupies the full viewport height and the browser
  snaps between sections on scroll

### Requirement: 7-Slide Dashboard Flow

The system SHALL render the following 7 dashboard slides in order: Cover
(hero), KPI Dashboard (6 metric cards), Timeline (weekly bar chart), Type
Breakdown, Project Cards, Proposals Table, and Commit Log.

**Supersedes**: "Dashboard Sections" requirement that specified "Hero banner,
KPI strip (5 cards), timeline bar chart, type breakdown, project cards,
proposals table, and commit log." The new flow adds a dedicated Cover slide,
expands KPI from 5 to 6 cards (adding the average daily lines metric), and
restructures the Hero as a standalone slide.

#### Scenario: All slides present
- **GIVEN** a `ShowcaseData` instance with commits, proposals, and projects
- **WHEN** the HTML report is generated
- **THEN** the output contains 7 slide sections: Cover, KPI Dashboard,
  Timeline, Type Breakdown, Projects, Proposals, and Commit Log

### Requirement: KPI Dashboard with 6 Metrics

The system SHALL display 6 KPI metric cards: Total Commits, Active
Repositories, OpenSpec Proposals, Lines Added, Lines Removed, and Average
Daily Lines Changed.

**Supersedes**: "KPI strip (5 cards)" from the Dashboard Sections requirement.
The sixth card is the new average daily lines metric.

#### Scenario: KPI cards rendered
- **GIVEN** a `ShowcaseData` instance
- **WHEN** the HTML report is generated
- **THEN** the KPI section contains 6 metric cards including one for average
  daily lines changed

### Requirement: Typography Craft System

The system SHALL use a structured typography system with serif display font
(`'Iowan Old Style', 'Charter', Georgia, serif`), sans-serif body font
(system font stack), and monospace for data numbers
(`font-variant-numeric: tabular-nums`). Headlines use negative letter-spacing
and a 3-weight hierarchy (400 body / 550 labels / 600 headlines).

#### Scenario: Typography applied
- **GIVEN** the generated HTML file
- **WHEN** inspecting CSS
- **THEN** display headings use serif with negative tracking, body text uses
  sans-serif, and numeric values use monospace with tabular-nums

### Requirement: Empty Data State

When a slide's data set is empty (zero commits, zero proposals, zero timeline
entries), the slide SHALL still render with a centred message indicating no
data is available (e.g. "無提交記錄", "無提案紀錄"). Slides are never hidden.

#### Scenario: No commits after filtering
- **GIVEN** a `ShowcaseData` instance with zero commits
- **WHEN** the HTML report is generated
- **THEN** the Timeline, Type Breakdown, and Commit Log slides each display
  an empty-state message instead of charts or tables

## Removed Requirements

### Soft Paper Theme

The "Soft Paper Theme" requirement (canvas `#f2f2f0`, surface `#fff`, ink
`#0a0a0a`) is superseded by the "Editorial Paper Design System" above.

### Dashboard Sections

The "Dashboard Sections" requirement (5-card KPI strip + continuous scroll
layout) is superseded by "7-Slide Dashboard Flow" and "KPI Dashboard with 6
Metrics" above.
