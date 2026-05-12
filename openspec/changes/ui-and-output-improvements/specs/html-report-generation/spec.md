## MODIFIED Requirements

### Requirement: Test Metrics Slide

The dashboard SHALL include a "測試" slide displaying test metrics for each project. The slide SHALL contain a KPI card row and a detail table. The KPI card row SHALL render ABOVE the detail table, showing totals: 總測試檔案 (sum of all test_file_count), 總測試案例 (sum of all test_case_count), 平均覆蓋率 (arithmetic mean of coverage_percent values from projects where coverage is not null; display "—" if no project has coverage data). Below the KPI card row, the detail table SHALL display columns: 專案, 框架, 測試檔案, 測試案例, 覆蓋率.

#### Scenario: KPI row renders above the detail table
- **GIVEN** test metrics data for multiple projects
- **WHEN** the test metrics slide is rendered
- **THEN** the KPI card row (總測試檔案, 總測試案例, 平均覆蓋率) SHALL appear above the per-project detail table in the DOM order

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

### Requirement: Project Cards Grid Layout

The project cards grid SHALL use a 3-column layout at desktop widths (≥901px), 2 columns at tablet widths (601–900px), and 1 column at mobile widths (≤600px). The CSS SHALL use a mobile-first `min-width` approach. The project cards section SHALL be wrapped in a scroll container (`.scroll-container`) to enable vertical scrolling when content exceeds viewport height, matching the behavior of the commit log, proposal, and test metrics sections. The section SHALL use the `slide--scrollable` class.

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

#### Scenario: Project cards scroll container
- **GIVEN** more project cards than fit within the viewport height
- **WHEN** the project cards section is rendered
- **THEN** the `#projectGrid` element SHALL be wrapped in a `.scroll-container` div that enables vertical scrolling

#### Scenario: Project slide uses scrollable class
- **GIVEN** the project cards slide is rendered
- **WHEN** inspecting the slide element's CSS classes
- **THEN** the slide element SHALL include the `slide--scrollable` class
