# HTML Report Generation — Delta

## MODIFIED Requirements

### Requirement: Project Cards Grid Layout

The project cards grid SHALL use a 3-column layout at desktop widths (≥901px), 2 columns at tablet widths (601–900px), and 1 column at mobile widths (≤600px). The CSS SHALL use a mobile-first `min-width` approach.

#### Scenario: Desktop viewport
- **GIVEN** a viewport width ≥901px
- **WHEN** the project cards section is rendered
- **THEN** the grid displays 3 columns

#### Scenario: Tablet viewport
- **GIVEN** a viewport width between 601px and 900px
- **WHEN** the project cards section is rendered
- **THEN** the grid displays 2 columns

#### Scenario: Mobile viewport
- **GIVEN** a viewport width ≤600px
- **WHEN** the project cards section is rendered
- **THEN** the grid displays 1 column

### Requirement: Proposal Table Column Order

The OpenSpec proposals table columns SHALL be ordered: 日期, 專案, 代稱, 工作項.

#### Scenario: Proposals table column order
- **GIVEN** the proposals table is rendered
- **WHEN** viewing the table headers
- **THEN** the column order is 日期, 專案, 代稱, 工作項

### Requirement: Date Column Width in Data Tables

The date cells in both the proposals table and the commit log table SHALL apply `white-space: nowrap` and `min-width: calc(10ch + var(--gap-sm) + var(--gap-sm))` via a shared `.date-cell` CSS class, so that `YYYY-MM-DD` dates always render on a single line without wrapping.

#### Scenario: Proposals date displays on one line
- **GIVEN** a proposals table with a date value "2026-05-08"
- **WHEN** the table is rendered at any supported viewport width
- **THEN** the date displays on a single line without wrapping

#### Scenario: Commits date displays on one line
- **GIVEN** a commit log table with a date value "2026-05-08"
- **WHEN** the table is rendered at any supported viewport width
- **THEN** the date displays on a single line without wrapping

#### Scenario: Date column minimum width
- **GIVEN** either the proposals or commit log table is rendered
- **WHEN** the table layout algorithm distributes column widths
- **THEN** the date column is at least `calc(10ch + var(--gap-sm) + var(--gap-sm))` wide

### Requirement: Commit Table Column Order

The commit log table columns SHALL be ordered: 日期, 專案, Hash, 類型, 說明, +/−.

#### Scenario: Commit table column order
- **GIVEN** the commit log table is rendered
- **WHEN** viewing the table headers
- **THEN** the column order is 日期, 專案, Hash, 類型, 說明, +/−

### Requirement: 8-Slide Dashboard Flow

The system SHALL render 8 dashboard slides in order: Cover (hero), KPI Dashboard, Timeline, Type Breakdown, Project Cards, Test Metrics, Proposals Table, and Commit Log.

#### Scenario: All slides present
- **GIVEN** a `ShowcaseData` instance with commits, proposals, projects, and test metrics
- **WHEN** the HTML report is generated
- **THEN** the output contains 8 slide sections in the specified order

## ADDED Requirements

### Requirement: Test Metrics Slide

The dashboard SHALL include a "測試" slide displaying test metrics for each project. The slide SHALL contain a table with columns: 專案, 框架, 測試檔案, 測試案例, 覆蓋率. Below the table, a KPI card row SHALL show totals: 總測試檔案 (sum of all test_file_count), 總測試案例 (sum of all test_case_count), 平均覆蓋率 (arithmetic mean of coverage_percent values from projects where coverage is not null; display "—" if no project has coverage data).

#### Scenario: Test metrics table rendered
- **GIVEN** test metrics data for 3 projects with varying coverage
- **WHEN** the test metrics slide is rendered
- **THEN** the table displays one row per project with framework, file count, case count, and coverage percentage

#### Scenario: Coverage progress bar colors
- **GIVEN** a project with coverage ≥80%
- **WHEN** the coverage cell is rendered
- **THEN** the progress bar uses the green color (#2f7d4a)

#### Scenario: Coverage unavailable
- **GIVEN** a project with no coverage data (coverage_percent is null)
- **WHEN** the coverage cell is rendered
- **THEN** the cell displays "—" with no progress bar

#### Scenario: No test metrics
- **GIVEN** zero test metrics entries
- **WHEN** the test metrics slide is rendered
- **THEN** the slide displays an empty-state message "無測試資料"

#### Scenario: Framework detected but zero test files
- **GIVEN** a project with framework "pytest" but no files matching test patterns
- **WHEN** the project row is rendered
- **THEN** test_file_count and test_case_count display 0

#### Scenario: Test files found but zero test cases
- **GIVEN** a project with 3 test files but no lines matching test function patterns
- **WHEN** the project row is rendered
- **THEN** test_file_count displays 3 and test_case_count displays 0
