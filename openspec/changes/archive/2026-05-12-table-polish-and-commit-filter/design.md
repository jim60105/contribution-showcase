# Table Polish and Commit Filter

## Context

The contribution-showcase tool generates a self-contained HTML dashboard from
Git history and OpenSpec archives. The current report has four cosmetic and
functional gaps:

1. A specific commit (`dd33ee63950bb49a284de835528343561f1a70d5`) needs to be
   excluded from all metrics and tables, and there is no mechanism to exclude
   commits by hash.
2. The commit log table shows date, project, type, subject, and +/− but not
   the commit hash — making it hard to cross-reference entries with `git log`.
3. The proposals table includes a "說明" (description) column that duplicates
   information already available in the slug and adds clutter.
4. The HTML page has no favicon, leaving the browser tab with a generic icon.

The existing `apply_filters()` function in `collector.rs` already handles
date-range and type filtering for commits. The `FilterConfig` struct in
`config.rs` is deserialized from the `[filters]` section of `showcase.toml`.
The `CommitEntry` model already stores the full `hash` field. The HTML template
in `templates/page.html` renders both tables via JavaScript functions
`renderCommits()` and `renderProposals()`.

## Goals / Non-Goals

**Goals:**

- Provide a declarative mechanism to exclude specific commits by hash via
  config, without requiring code changes for each exclusion.
- Show the short commit hash in the commit log table for traceability.
- Simplify the proposals table by removing the low-value description column.
- Give the page a distinctive favicon for tab identification.

**Non-Goals:**

- Regex-based or pattern-based commit filtering (exact match is sufficient).
- Making the proposals table columns configurable via config.
- Supporting multiple favicon formats or theme-aware favicons.
- Adding hash-based exclusion to proposals (only commits are affected).

## Decisions

### D1 — `exclude_hashes` stores full 40-character hex strings

**Decision:** The `exclude_hashes` field in `[filters]` accepts a list of full
40-character SHA-1 hex strings. Matching is exact string comparison.

```toml
[filters]
author = "Jim"
exclude_hashes = [
    "dd33ee63950bb49a284de835528343561f1a70d5",
]
```

```rust
// In FilterConfig
pub exclude_hashes: Option<Vec<String>>,
```

**Why:** Full hashes are unambiguous and avoid the collision risk of short
hashes. The `CommitEntry.hash` field already stores the full `%H` value from
`git log`, so comparison is a simple `==`. Asking users to paste full hashes is
a minor inconvenience offset by zero ambiguity — and they can always copy from
`git log --format=%H`.

**Alternatives considered:**

- *Short hash prefix matching (`starts_with`)*: Introduces collision risk and
  makes the config file ambiguous about which commit is targeted. Not worth the
  convenience trade-off for a list that will rarely exceed a handful of entries.
- *Regex pattern*: Overkill for exact hash exclusion. Adds regex compilation
  cost and complexity.

### D2 — Exclusion point: inside `apply_filters()`

**Decision:** Add the hash exclusion predicate to the existing
`apply_filters()` function in `collector.rs`, alongside the date and type
filters. The predicate runs early in the filter chain (before date checks) to
skip excluded commits as quickly as possible.

```rust
// Inside apply_filters(), within the commit filter closure:
if let Some(ref hashes) = filters.exclude_hashes {
    if hashes.iter().any(|h| h == &c.hash) {
        return false;
    }
}
```

**Why:** `apply_filters()` is the single, well-tested chokepoint for all
commit filtering. Adding the check here means every downstream consumer
(timeline, breakdown, counts, tables) automatically respects the exclusion
without any additional code. It also keeps the git log collection clean —
`collect_git_commits_filtered()` remains a pure data collector.

**Alternatives considered:**

- *Filter during `collect_git_commits_filtered()`*: Would require passing
  filter config into the collection function, coupling collection with
  filtering. The current architecture intentionally separates these concerns.
- *Filter in JavaScript (client-side)*: The excluded commit's data would still
  be serialized into the HTML payload, wasting bytes and leaking data the user
  explicitly wanted hidden.

### D3 — Short hash display: first 8 characters, monospace

**Decision:** The "Hash" column in the commit log table displays
`commit.hash.substring(0, 8)` rendered in monospace font (reusing the existing
`.mono` CSS class). The column is the first column, before "日期".

```javascript
// In renderCommits():
html += '<th>Hash</th>';  // first column, not sortable
// ...
html += '<td class="mono">' + escapeHtml(c.hash.substring(0, 8)) + '</td>';
```

**Why:** 8 characters provide sufficient uniqueness for human identification
(Git's default short hash is 7–10 chars). Placing it first makes visual
scanning natural — hash → date → project → type → subject → changes. Monospace
ensures consistent column width. The column header uses the English "Hash"
since it is a technical identifier, consistent with Git terminology used
universally.

**Alternatives considered:**

- *7 characters (Git's minimum)*: Slightly higher collision risk in large
  repos; 8 is a safe middle ground widely used by GitHub and GitLab.
- *Full 40-character hash*: Wastes horizontal space for no practical benefit
  in a visual report.
- *Linkable hash (to a remote URL)*: Requires knowing the remote URL, which
  is not in the config. Out of scope.
- *Making the column sortable*: Sorting by hash has no semantic meaning. Kept
  as a static column.

### D4 — Favicon emoji: 📊 (bar chart)

**Decision:** Use the 📊 (bar chart / chart with upwards trend) emoji as the
favicon, rendered via an inline SVG data URI in the `<head>`:

```html
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'%3E%3Ctext y='.9em' font-size='90'%3E📊%3C/text%3E%3C/svg%3E">
```

**Why:** 📊 directly represents data visualization and analytics, which is the
core purpose of a contribution showcase dashboard. It is universally supported
in modern browsers (including Chromium 77), requires zero external files
(air-gapped compatible), and adds only ~150 bytes to the HTML. The
percent-encoded SVG data-URI approach avoids raw `<` / `>` inside an HTML
attribute, ensuring robust parsing across all HTML parsers.

**Alternatives considered:**

- *🗺️ (map)*: Represents the project specifically but not the tool's function
  (contribution reporting).
- *🏗️ (building)*: Too generic; could mean anything under construction.
- *🔬 (microscope)*: Implies research rather than reporting.
- *External `.ico` file*: Violates the air-gapped self-contained HTML
  constraint (the file would need to be bundled or base64-encoded separately).

### D5 — No configuration toggle for proposals table columns

**Decision:** The "說明" column is removed unconditionally from the proposals
table. No config option is added to control which columns are shown.

**Why:** The description column duplicates information already conveyed by the
slug (which is the OpenSpec change directory name and is inherently
descriptive). Adding a column-visibility config would increase complexity for
a feature nobody has requested. The tool has zero external users, so there is
no backward-compatibility concern. If column configurability is ever needed,
it can be added as a separate change with its own proposal.

**Alternatives considered:**

- *Make columns configurable via `[table.proposals.columns]`*: Over-engineered
  for a single-user tool. Adds config parsing, validation, and template logic
  for a feature with no demand.
- *Keep the column but make it optional*: Still adds config complexity for
  no benefit.

## Risks / Trade-offs

### R1 — `exclude_hashes` is an exact-match-only mechanism

If a user enters a short hash or a hash with incorrect casing, the exclusion
silently does nothing — no validation error is raised. Values are treated as
opaque strings and compared with `==` against `CommitEntry.hash`. **Mitigation:**
The field name `exclude_hashes` (plural, full word) and TOML examples in
`showcase.toml` make it clear that full hashes are expected.

### R2 — Hash column increases table width

Adding a column to an already-wide table may cause horizontal scrolling on
narrow viewports. **Mitigation:** The hash column is only 8 monospace
characters wide (~80px). A `overflow-x: auto` rule on the table container
ensures graceful horizontal scroll when content overflows.

### R3 — Removing 說明 from proposals loses information

Users who relied on the description column will no longer see it.
**Mitigation:** The tool has zero external users. The slug already conveys
the change's identity (e.g., `table-polish-and-commit-filter`), and the full
description is available in each proposal's `proposal.md` file. The column
removal is a simplification, not a data loss.

### R4 — Emoji favicon rendering varies across platforms

The 📊 emoji may render differently on Windows, macOS, and Linux, and some
very old browsers may not support SVG data-URI favicons at all.
**Mitigation:** The target browser is Chromium 77+, which fully supports SVG
data-URI favicons. The emoji is a cosmetic enhancement — if it fails to render,
the browser simply shows its default favicon with no functional impact.
