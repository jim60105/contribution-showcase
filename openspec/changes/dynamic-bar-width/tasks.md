## 1. CSS Changes

- [ ] 1.1 Remove `gap: 10px` from `.timeline-chart` in `templates/page.html`
- [ ] 1.2 Add `justify-content: space-evenly` to `.timeline-chart`
- [ ] 1.3 Remove `flex: 1` from `.bar-wrapper`

## 2. JavaScript Changes

- [ ] 2.1 Compute bar width percentage `(100 / (timeline.length * 2))` in the timeline rendering function and apply it as an inline `width` style on each `.bar-wrapper` element

## 3. Verification

- [ ] 3.1 Run `cargo test` to ensure all existing tests pass
- [ ] 3.2 Run `cargo run -- generate` and visually verify the bar chart shows evenly spaced bars occupying half the chart width
