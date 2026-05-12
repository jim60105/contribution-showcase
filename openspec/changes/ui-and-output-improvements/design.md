## Context

This change addresses four usability gaps in the generated HTML showcase report. The tool is pre-release with no external users, so backward compatibility and migration are non-concerns.

The four improvements span two layers:

- **Backend (Rust)**: Dynamic output filename derivation and adaptive timeline granularity selection.
- **Frontend (HTML/JS)**: Test metrics KPI reorder and project grid scroll container.

All changes are additive or structural—no data model fields are added or removed, and the `ShowcaseData` JSON schema remains unchanged. The `TimelineEntry.label` field already holds a string; only its format varies.

## Goals / Non-Goals

### Goals

- Derive a meaningful default output filename from the configured `title`, so multiple showcase reports can coexist in the same output directory.
- Select daily, weekly, or monthly timeline granularity automatically based on commit date range, improving chart readability for both short sprints and long project histories.
- Surface test KPI cards (total files, total cases, average coverage) above the detail table for faster scanning.
- Make the project grid scrollable to prevent viewport overflow, consistent with other scrollable sections.

### Non-Goals

- Custom user-specified granularity (e.g., `granularity = "monthly"` in config). Automatic detection is sufficient for now; a config override can be added later if needed.
- Output format negotiation (e.g., PDF, Markdown). The tool outputs HTML only.
- Responsive / mobile layout improvements. The report targets desktop viewing.
- Internationalization of timeline labels. Labels use ISO date formats (`YYYY-MM-DD`, `YYYY-Www`, `YYYY-MM`) which are locale-independent.

## Decisions

### 1. Filename sanitization strategy

When `output` path is not explicitly configured, the default filename derives from `config.title`:

1. Replace any character that is not ASCII alphanumeric, hyphen, or underscore with a hyphen.
2. Collapse consecutive hyphens into one.
3. Trim leading/trailing hyphens.
4. Lowercase the result.
5. Append `.html`.
6. Fallback to `index.html` if `title` is `None`, empty, or sanitizes to an empty string.
7. The output directory remains `dist/` by default, producing `dist/{sanitized-title}.html`.

The `output_path()` method signature changes from `&str` to `String` because the title-derived filename is computed at call time and cannot be returned as a borrowed reference. All call sites (`main.rs`) are updated accordingly.

**Rationale**: This keeps filenames portable across OS filesystems (no Unicode, no special chars, no spaces). The sanitization lives in `config.rs` alongside `output_path()`, keeping path logic centralized. No new dependencies are needed—this is simple byte-level string manipulation. Returning `String` is the simplest approach; `Cow<'_, str>` would add complexity without benefit since the method is called once per run.

### 2. Adaptive timeline granularity

Granularity is selected in `build_timeline()` in `src/collector.rs` based on the calendar-day span between the earliest and latest commit dates:

| Day span | Granularity | Label format | chrono format |
|----------|-------------|--------------|---------------|
| 0–13     | Daily       | `2025-01-15` | `%Y-%m-%d`    |
| 14–60    | Weekly      | `2025-W03`   | `%G-W%V`      |
| 61+      | Monthly     | `2025-01`    | `%Y-%m`        |

The span is computed as `(max_date - min_date).num_days()` using calendar day difference.

**Rationale**: The thresholds (14 and 60 days) are chosen so that the chart produces roughly 14–60 bars in most cases, which fits well visually. The decision is made in the backend because timeline aggregation (summing insertions + deletions per bucket) already happens in Rust. The frontend renders whatever label strings and values arrive in the JSON—no frontend logic change is needed for this feature.

**Edge cases**: If there are 0 or 1 commits, `build_timeline()` already returns an empty or single-entry vec. The granularity selection is only meaningful with ≥ 2 commits and degrades gracefully.

### 3. Test metrics KPI reorder

In `renderTestMetrics()` within `templates/page.html`, the construction order changes to emit the `.test-kpi-row` block before the `.data-table` block. This is a pure reorder of two sibling HTML construction blocks within the same function.

**Rationale**: KPI summary cards provide at-a-glance insight and should appear first, following the information hierarchy pattern used elsewhere in the report (e.g., commit stats summary above commit log). No data model or CSS changes are required—both elements already have correct styling.

### 4. Project grid scroll container

The project section (`#projectGrid`) is wrapped in a `scroll-container` div, and the section element gains the `slide--scrollable` class. This matches the pattern already used by the commit log, proposal, and test metrics sections.

**Rationale**: Consistency with existing scrollable sections. The CSS for `scroll-container` and `slide--scrollable` already exists in the template; no new styles are needed.

### 5. No new dependencies

All four changes use existing crate capabilities (`chrono` formatting, standard string operations) and existing CSS classes. No new Rust crates or JS libraries are introduced.

## Risks / Trade-offs

### Filename collisions

Two configs with titles that sanitize to the same string (e.g., "My Project!" and "my-project") will produce the same filename. This is acceptable because a single user is unlikely to have colliding titles, and explicit `output.path` config remains available as an escape hatch.

### Granularity threshold rigidity

Fixed thresholds (14/60 days) may produce suboptimal bucket counts for unusual date ranges (e.g., exactly 14 days yields 2 weekly bars). This is a minor cosmetic issue. If problematic, thresholds can be tuned later or a config override added—but adding config surface area now is premature.

### Timeline label sorting

Daily and monthly labels sort lexicographically in the correct chronological order (ISO 8601). Weekly labels (`YYYY-Www`) also sort correctly. No special sort logic is needed.

### Template maintainability

The HTML template is a single large file with embedded JS. Reordering blocks in `renderTestMetrics()` and adding a wrapper div are small, low-risk changes. The lack of a component framework means changes must be carefully placed, but the scope here is minimal.
