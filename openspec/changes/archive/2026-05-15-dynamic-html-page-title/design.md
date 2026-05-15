## Context

The HTML template `templates/page.html` has a hardcoded
`<title>貢獻總覽</title>` element. The user-configured title from
`showcase.toml` is only applied at runtime by JavaScript (line 791:
`document.getElementById('coverTitle').textContent = DATA.title`), meaning
the browser tab, bookmark label, and SEO snippet still read the static
fallback regardless of configuration.

The existing generation pipeline in `run_generate()` already performs one
placeholder replacement (`"__SHOWCASE_DATA__"` → JSON payload). The title
is available in `ShowcaseData.title` (populated from `Config.title`).

## Goals / Non-Goals

**Goals:**

- The `<title>` element in the generated HTML reflects the user-configured
  title.
- Special characters in the title are safely escaped for HTML context.
- The change is minimal — one new placeholder, one new `.replace()` call.

**Non-Goals:**

- Making other `<head>` metadata dynamic (description, og:tags) — future work.
- Changing the visible cover heading logic (already handled by JS).
- Adding a separate fallback mechanism — `collector::collect()` already
  resolves `Config.title` to `"貢獻總覽"` when absent, so
  `ShowcaseData.title` always has a value by the time generation runs.

## Decisions

### Decision 1: New placeholder `__PAGE_TITLE__` in the template

Introduce a dedicated placeholder `__PAGE_TITLE__` in the `<title>` tag
rather than reusing `__SHOWCASE_DATA__`.

**Rationale**: The data placeholder is JSON-escaped (for `<script>` context).
HTML `<title>` requires HTML-entity escaping (e.g., `&` → `&amp;`), which
is a different escaping domain. A separate placeholder keeps the two
replacement passes independent and correct.

**Alternative considered**: Parse the JSON after injection and patch the
`<title>` via string manipulation — rejected because it's fragile and
unnecessarily complex.

### Decision 2: HTML-entity escaping for the title

Apply a minimal HTML-entity escape (`&`, `<`, `>`, `"`) when injecting the
title into the `<title>` tag. This prevents malformed HTML if the title
contains special characters.

**Rationale**: The `<title>` element is in a raw text context per the HTML
spec; only `&` and `<` are strictly dangerous, but escaping `>` and `"` is
standard defensive practice.

### Decision 3: Replacement order

Perform the `__PAGE_TITLE__` replacement **before** the `__SHOWCASE_DATA__`
replacement. This avoids any theoretical risk of the title text containing
the data placeholder pattern.

**Rationale**: Readability and a predictable pipeline — title first, then
data. Rust's `str::replace` does not recursively replace inside
replacement text, so the order is not strictly required for correctness,
but a consistent left-to-right pass is easier to reason about.

## Risks / Trade-offs

- **[Risk]** Title containing `__PAGE_TITLE__` literally → would be consumed
  by the replacement. → **Mitigation**: Extremely unlikely for a display title;
  acceptable risk.
- **[Risk]** Compile-time template change requires rebuild. →
  **Mitigation**: Already the case for any template change; no new burden.
