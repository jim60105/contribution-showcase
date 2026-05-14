## 1. CSS Label Positioning Fix

- [x] 1.1 Increase `.timeline-chart` `padding-bottom` from `40px` to `80px` in `templates/page.html`
- [x] 1.2 Change `.bar-label` positioning from `bottom: -36px; transform-origin: top left` to `top: 100%; right: 50%; margin-top: 4px; transform-origin: top right` in `templates/page.html`

## 2. CSS Variable Audit

- [x] 2.1 Verify all static `var(--X)` references resolve to `:root`-defined custom properties (already fixed: `--card-bg` → `--surface`, `--fs-sm` → `--fs-xs`)

## 3. Tests and Verification

- [x] 3.1 Run full test suite (`cargo test`) to verify all existing tests pass
- [x] 3.2 Visual verification: screenshot the timeline chart to confirm labels no longer overlap bars

## 4. Regenerate Docs

- [x] 4.1 Run `cargo run -- generate` and copy `dist/index.html` to `docs/contribution-showcase.html`
