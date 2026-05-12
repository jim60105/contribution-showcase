# Premium Showcase Redesign

## Context

The `contribution-showcase` Rust CLI generates a self-contained HTML dashboard
summarising a contributor's Git activity across repositories. The current
template produces a crude continuous-scrolling page — functional, but visually
unremarkable and lacking the polish expected of a portfolio-grade deliverable.

This proposal redesigns the HTML template into a **premium, slide-style
presentation** inspired by the *open-design* project's design philosophy:
seed-based architecture, anti-AI-slop rules, typography craft, animation
discipline, and component patterns. The result is a 7-slide editorial experience
that treats contributor data as a story rather than a dump.

### Constraints

| Constraint | Detail |
|---|---|
| **Modern browsers** | Standalone HTML tool opened in a regular browser. Target: Chrome 90+, Firefox 90+, Safari 15+. Not embedded in the desktop client's Chromium 77 webview (that constraint applies to my-frontend only). |
| **Self-contained** | Single HTML file, no external network requests. All CSS and JS inline. |
| **Air-gapped** | No CDN fonts, no analytics, no external resources of any kind. |
| **Rust template** | The HTML is generated from Rust string templates — changes affect `templates/page.html` (loaded via `include_str!("../templates/page.html")`). |
| **JS style** | JavaScript uses ES5-style `var`/`function` for readability and simplicity in template code. Not a compatibility requirement — a stylistic choice for maintainability. |

### Key Browser Features Used

| Feature | Notes |
|---|---|
| `IntersectionObserver` | Widely supported in all target browsers |
| CSS `scroll-snap-type` | Widely supported in all target browsers |
| CSS Custom Properties | Widely supported in all target browsers |
| CSS `clamp()` | Supported in Chrome 79+, Firefox 75+, Safari 13.1+ |
| `requestAnimationFrame` | Used for numeric counter animations |

---

## Goals / Non-Goals

### Goals

1. **Slide-based layout** — replace continuous scroll with a 7-slide,
   full-viewport presentation using CSS scroll-snap.
2. **Editorial design language** — establish an "Editorial Paper" design system
   with intentional typography, restrained colour palette, and craft-level
   attention to spacing.
3. **Scroll-triggered animations** — elements animate into view as slides are
   entered, using `IntersectionObserver`.
4. **Navigation indicator** — fixed dot nav on the right edge for spatial
   orientation and direct slide access.
5. **New metric** — compute and display "Average Daily Lines" in the KPI
   dashboard (requires a small Rust backend change).
6. **Responsive behaviour** — graceful degradation from desktop slides to mobile
   single-column flow.
7. **Anti-AI-slop quality bar** — enforce design rules that prevent the template
   from looking like generic AI-generated output.

### Non-Goals

- **Interactive filtering / search** — the showcase is a static snapshot, not an
  app. No client-side filtering of commits or repos.
- **Dark mode** — out of scope for this change. The "Editorial Paper" palette is
  light-only by design. A future proposal may add a dark variant.
- **External font loading** — system font stacks only, to satisfy air-gap and
  self-contained constraints.
- **Accessibility audit** — basic semantic HTML (headings, sections, ARIA labels
  on nav) is included, but a full WCAG audit is not in scope.
- **Printing** — no print stylesheet. The HTML is designed for screen viewing.

---

## Decisions

### 1. Slide-Based Page Layout

**Decision:** Use CSS `scroll-snap-type: y mandatory` on the `<main>` container
to create a slide-style presentation where each `<section>` occupies the full
viewport.

**Mechanism:**

```css
main {
  height: 100vh;
  overflow-y: auto;
  scroll-snap-type: y mandatory;
  scroll-behavior: smooth;
}

section.slide {
  min-height: 100vh;
  scroll-snap-align: start;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: var(--gap-xl) var(--gap-lg);
  box-sizing: border-box;
}
```

Each slide is a flex container that vertically and horizontally centres its
content. The `min-height: 100vh` ensures every slide fills the viewport; content
taller than the viewport (slides 6 and 7) scrolls internally via a nested
scrollable container.

**Internal scroll for long-content slides:**

```css
section.slide--scrollable {
  justify-content: flex-start;
  align-items: stretch;
}

section.slide--scrollable .slide-inner {
  flex: 1;
  overflow-y: auto;
  max-height: calc(100vh - var(--gap-xl) * 2);
  padding: var(--gap-md);
}
```

Slides 6 (Proposals) and 7 (Commit Log) use the `.slide--scrollable` variant.
Their table content is wrapped in `.slide-inner` which provides its own vertical
scroll, keeping the outer snap behaviour intact.

**Alternative considered:** Continuous scroll (the current design). Rejected
because the explicit design goal is a slide-style presentation that treats each
data category as a distinct "page" in a story.

**Alternative considered:** JavaScript-driven slide transitions (e.g.
`scrollTo` on wheel events). Rejected because CSS scroll-snap is natively
supported in all target browsers and provides a smoother, more reliable
experience without fighting the browser's scroll model.

---

### 2. Design System — "Editorial Paper" Theme

**Decision:** Establish a cohesive design token system via CSS custom properties,
inspired by open-design's seed-based architecture. Every colour, font, spacing
value, and radius is defined once in `:root` and referenced throughout.

```css
:root {
  /* ── Palette ── */
  --bg: #f8f7f4;          /* warm off-white paper */
  --surface: #ffffff;      /* card / raised surface */
  --fg: #1a1916;           /* primary text — near-black, warm */
  --muted: #6b6964;        /* secondary text */
  --border: #e8e5df;       /* dividers, card borders */
  --accent: #2563eb;       /* interactive / highlight */

  /* ── Commit Type Colours ── */
  --feat: #2f7d4a;
  --fix: #b53a2a;
  --docs: #6366f1;
  --refactor: #d97706;
  --test: #0891b2;
  --chore: #6b7280;
  --ci: #7c3aed;
  --build: #ea580c;
  --style: #db2777;
  --perf: #059669;
  --other: #9a9a95;

  /* ── Typography ── */
  --font-display: 'Iowan Old Style', 'Charter', Georgia, serif;
  --font-body: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  --font-mono: ui-monospace, 'SF Mono', Menlo, monospace;

  /* ── Type Scale ── */
  --fs-display: clamp(48px, 6vw, 76px);
  --fs-h1: clamp(32px, 4vw, 48px);
  --fs-h2: clamp(24px, 3vw, 36px);
  --fs-body: 16px;
  --fs-small: 14px;
  --fs-xs: 12px;

  /* ── Spacing (8-point grid) ── */
  --gap-xs: 8px;
  --gap-sm: 12px;
  --gap-md: 20px;
  --gap-lg: 32px;
  --gap-xl: 56px;
  --gap-2xl: 96px;

  /* ── Layout ── */
  --container: 1120px;
  --radius: 12px;
}
```

**Palette rationale:**

- `--bg: #f8f7f4` is a warm off-white that evokes quality paper stock, avoiding
  the sterile `#ffffff` background common in generic dashboards.
- `--fg: #1a1916` is a warm near-black that pairs with the paper tone, avoiding
  harsh pure-black text.
- `--accent: #2563eb` is a deliberate blue chosen for its readability against
  warm backgrounds. It is **not** Tailwind's default indigo — it is a custom
  token.
- Commit type colours are carried forward from the current template to maintain
  familiarity, but are now defined as tokens rather than inline styles.

**Anti-AI-slop rules** (enforced during implementation):

| Rule | Rationale |
|---|---|
| No default Tailwind indigo | Signals lazy generation; use `--accent` instead |
| No emoji as icons | Emoji render inconsistently and signal low craft |
| No lorem ipsum or placeholder copy | Every string must be real data or a meaningful label |
| Max 2 typefaces | Serif for display, sans for body — more is noise |
| `font-variant-numeric: tabular-nums` on data | Numbers in columns must align; proportional figures break grids |
| ALL CAPS text gets `letter-spacing: 0.08em` | Caps without tracking looks amateurish |
| No gratuitous gradients or shadows | Shadows only on raised surfaces (cards); no decorative gradients |
| No border-radius > `--radius` | Consistency; pill shapes are prohibited |

---

### 3. Typography System

**Decision:** A disciplined type scale with exactly two typeface families and
strict rules for weight, spacing, and line-height at each level.

| Level | Size | Family | Weight | Letter-spacing | Line-height | Usage |
|---|---|---|---|---|---|---|
| Display | `clamp(48px, 6vw, 76px)` | `--font-display` (serif) | 600 | `-0.03em` | 1.1 | Cover slide title |
| H1 | `clamp(32px, 4vw, 48px)` | `--font-display` (serif) | 600 | `-0.02em` | 1.2 | Slide section headings |
| H2 | `clamp(24px, 3vw, 36px)` | `--font-body` (sans) | 600 | `-0.01em` | 1.3 | Sub-headings, card titles |
| Body | `16px` | `--font-body` (sans) | 400 | `normal` | 1.6 | Paragraphs, descriptions |
| Small | `14px` | `--font-body` (sans) | 500 | `0.01em` | 1.5 | Labels, captions, metadata |
| XS | `12px` | `--font-body` (sans) | 500 | `0.02em` | 1.4 | Axis labels, footnotes |
| Metric | varies | `--font-mono` (mono) | 600 | `normal` | 1.0 | KPI numbers, counters |

**CSS implementation:**

```css
.display { font-family: var(--font-display); font-size: var(--fs-display); font-weight: 600; letter-spacing: -0.03em; line-height: 1.1; }
.h1      { font-family: var(--font-display); font-size: var(--fs-h1); font-weight: 600; letter-spacing: -0.02em; line-height: 1.2; }
.h2      { font-family: var(--font-body); font-size: var(--fs-h2); font-weight: 600; letter-spacing: -0.01em; line-height: 1.3; }
.body    { font-family: var(--font-body); font-size: var(--fs-body); font-weight: 400; line-height: 1.6; }
.small   { font-family: var(--font-body); font-size: var(--fs-small); font-weight: 500; letter-spacing: 0.01em; }
.xs      { font-family: var(--font-body); font-size: var(--fs-xs); font-weight: 500; letter-spacing: 0.02em; }
.metric  { font-family: var(--font-mono); font-weight: 600; font-variant-numeric: tabular-nums; }
.caps    { text-transform: uppercase; letter-spacing: 0.08em; }
```

**Negative letter-spacing on large type** is critical for display-quality
headings — large serif text at default tracking looks loose and unprofessional.
The values (`-0.03em`, `-0.02em`, `-0.01em`) tighten progressively less as size
decreases.

**`font-variant-numeric: tabular-nums`** on `.metric` ensures that KPI numbers,
counter animations, and table columns align vertically. Without this, digits
like "1" and "0" have different widths in proportional fonts, causing jitter
during counter animations and misaligned columns.

---

### 4. Scroll-Triggered Animations

**Decision:** Use `IntersectionObserver` to trigger CSS-class-based animations
when slides enter the viewport. Layout motion uses CSS transitions and
`@keyframes`. The exception is numeric counter animations, which use
`requestAnimationFrame` for smooth interpolation.

**Observer setup:**

```javascript
// All animation-ready elements start hidden
// CSS: .animate { opacity: 0; transform: translateY(24px); transition: opacity 0.6s ease, transform 0.6s ease; }
// CSS: .animate.in-view { opacity: 1; transform: translateY(0); }

var observer = new IntersectionObserver(function(entries) {
  entries.forEach(function(entry) {
    if (entry.isIntersecting) {
      entry.target.classList.add('in-view');
      observer.unobserve(entry.target);
    }
  });
}, { threshold: 0.2 });

document.querySelectorAll('.animate').forEach(function(el) {
  observer.observe(el);
});
```

**Animation types:**

#### Fade-up (sections, cards)

```css
.animate {
  opacity: 0;
  transform: translateY(24px);
  transition: opacity 0.6s cubic-bezier(0.22, 1, 0.36, 1),
              transform 0.6s cubic-bezier(0.22, 1, 0.36, 1);
}
.animate.in-view {
  opacity: 1;
  transform: translateY(0);
}
```

The `cubic-bezier(0.22, 1, 0.36, 1)` easing produces a quick start with a
gentle settle — more refined than linear or `ease-out`.

#### Stagger (card grids, list items)

```css
.stagger > .animate:nth-child(1) { transition-delay: 0ms; }
.stagger > .animate:nth-child(2) { transition-delay: 100ms; }
.stagger > .animate:nth-child(3) { transition-delay: 200ms; }
.stagger > .animate:nth-child(4) { transition-delay: 300ms; }
.stagger > .animate:nth-child(5) { transition-delay: 400ms; }
.stagger > .animate:nth-child(6) { transition-delay: 500ms; }
```

The Rust template generates `nth-child` rules up to the maximum number of
stagger children needed (6 for the KPI grid, variable for project cards).

#### Counter animation (KPI numbers)

```javascript
function animateCounter(el) {
  var target = parseInt(el.getAttribute('data-target'), 10);
  var duration = 1200;
  var start = null;

  function step(timestamp) {
    if (!start) start = timestamp;
    var progress = Math.min((timestamp - start) / duration, 1);
    // Ease-out cubic
    var eased = 1 - Math.pow(1 - progress, 3);
    el.textContent = Math.floor(eased * target).toLocaleString();
    if (progress < 1) {
      requestAnimationFrame(step);
    } else {
      el.textContent = target.toLocaleString();
    }
  }

  requestAnimationFrame(step);
}
```

Counter animation is triggered when the KPI slide's `IntersectionObserver`
fires. Each `.counter` element stores its final value in `data-target` and
displays "0" until animated.

#### Bar growth (timeline, type breakdown)

```css
.bar {
  width: 0;
  transition: width 0.6s cubic-bezier(0.22, 1, 0.36, 1);
}
.bar.in-view {
  /* width set via inline style by Rust template, e.g. style="--bar-w: 72%" */
  width: var(--bar-w);
}
```

Vertical bars (timeline) use `height` instead of `width`, with the same
transition. Stagger delays are applied via inline `transition-delay` set by the
Rust template based on bar index.

**Animation discipline (from open-design philosophy):**

- Every animation must **confirm a state change** (element entering viewport).
  No looping decorative animations.
- Duration budget: 600ms for transitions, 1200ms for counters. Nothing longer.
- No animation on elements already in the initial viewport at page load — the
  observer fires immediately, so cover slide content appears without delay.
- `prefers-reduced-motion: reduce` media query disables all transitions and
  skips counter animation (counters display final values immediately):

```css
@media (prefers-reduced-motion: reduce) {
  .animate { opacity: 1; transform: none; transition: none; }
  .bar { transition: none; }
  .counter { /* JS checks this query and sets final value without animation */ }
}
```

The counter JavaScript checks `window.matchMedia('(prefers-reduced-motion: reduce)').matches` and, if true, sets `el.textContent` to the target value immediately without calling `requestAnimationFrame`.

---

### 5. Navigation Indicator

**Decision:** A fixed vertical dot indicator on the right edge of the viewport,
providing spatial orientation and direct slide access.

**Structure:**

```html
<nav class="slide-nav" aria-label="Slide navigation">
  <button class="slide-nav-dot active" data-slide="0" aria-label="Cover">
    <span class="slide-nav-label">Cover</span>
  </button>
  <button class="slide-nav-dot" data-slide="1" aria-label="KPI Dashboard">
    <span class="slide-nav-label">KPIs</span>
  </button>
  <!-- ... 5 more dots ... -->
</nav>
```

**CSS:**

```css
.slide-nav {
  position: fixed;
  right: var(--gap-md);
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
  z-index: 100;
}

.slide-nav-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid var(--muted);
  background: transparent;
  cursor: pointer;
  padding: 0;
  transition: all 0.3s ease;
  position: relative;
}

.slide-nav-dot.active {
  width: 12px;
  height: 12px;
  background: var(--fg);
  border-color: var(--fg);
}

.slide-nav-label {
  position: absolute;
  right: 24px;
  top: 50%;
  transform: translateY(-50%);
  font-family: var(--font-body);
  font-size: var(--fs-xs);
  color: var(--muted);
  white-space: nowrap;
  opacity: 0;
  transition: opacity 0.2s ease;
  pointer-events: none;
}

.slide-nav-dot:hover .slide-nav-label {
  opacity: 1;
}
```

**Behaviour (JavaScript):**

```javascript
// Update active dot on scroll
var slides = document.querySelectorAll('section.slide');
var dots = document.querySelectorAll('.slide-nav-dot');

var navObserver = new IntersectionObserver(function(entries) {
  entries.forEach(function(entry) {
    if (entry.isIntersecting) {
      var index = parseInt(entry.target.getAttribute('data-slide-index'), 10);
      dots.forEach(function(dot, i) {
        dot.classList.toggle('active', i === index);
      });
    }
  });
}, { threshold: 0.5 });

slides.forEach(function(slide) {
  navObserver.observe(slide);
});

// Click to navigate
dots.forEach(function(dot) {
  dot.addEventListener('click', function() {
    var index = parseInt(dot.getAttribute('data-slide'), 10);
    slides[index].scrollIntoView({ behavior: 'smooth' });
  });
});
```

The nav uses a separate `IntersectionObserver` instance with `threshold: 0.5`
(the slide must be at least half-visible to be considered "active"), distinct
from the animation observer which uses `threshold: 0.2`.

---

### 6. 7-Slide UX Flow

**Decision:** The presentation is structured as exactly 7 slides, each with a
clear narrative purpose. The order follows a journalistic inverted pyramid:
headline → summary metrics → temporal view → categorical view → detail → raw
data.

#### Slide 1: Cover (Hero)

```
┌─────────────────────────────────────────────┐
│                                             │
│                                             │
│        Contribution Showcase                │  ← .display, serif
│        ───────────────────                  │
│        Author Name                          │  ← .small .caps
│        2024-01-01 — 2024-12-31              │  ← .small .muted
│                                             │
│              1,247                           │  ← .metric, counter animation
│           total commits                     │  ← .xs .caps .muted
│                                             │
│               ↓                             │  ← scroll indicator, pulsing
│                                             │
└─────────────────────────────────────────────┘
```

- The cover is intentionally sparse — a single bold metric (total commits) acts
  as the hook.
- The scroll indicator is a small chevron or arrow character with a subtle CSS
  `translateY` pulse animation (the **one** exception to "no decorative
  animations" — it serves a functional purpose of indicating scrollability).
- No card, no border, no background shape — just type on paper.

#### Slide 2: KPI Dashboard

```
┌─────────────────────────────────────────────┐
│                                             │
│    Overview                                 │  ← .h1, serif
│                                             │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│   │  1,247   │  │    12    │  │    34    │ │
│   │ Commits  │  │  Repos   │  │Proposals │ │
│   └──────────┘  └──────────┘  └──────────┘ │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│   │ +45,892  │  │ -12,340  │  │   158    │ │
│   │  Added   │  │ Removed  │  │Avg Daily │ │  ← NEW metric
│   └──────────┘  └──────────┘  └──────────┘ │
│                                             │
└─────────────────────────────────────────────┘
```

- 6 cards in a 3×2 grid, each with a large `.metric` number and a `.small .caps`
  label.
- "Lines Added" number is coloured `--feat` (green); "Lines Removed" is coloured
  `--fix` (red); "Avg Daily Lines" uses `--accent`.
- Cards fade in with 100ms stagger.
- All numbers use counter animation triggered by the slide's intersection.

**Card CSS:**

```css
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--gap-md);
  max-width: 720px;
  width: 100%;
}

.kpi-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: var(--gap-lg) var(--gap-md);
  text-align: center;
}

.kpi-card .metric {
  font-size: var(--fs-h1);
}

.kpi-card .label {
  margin-top: var(--gap-xs);
  color: var(--muted);
}
```

#### Slide 3: Timeline

```
┌─────────────────────────────────────────────┐
│                                             │
│    Weekly Activity                          │  ← .h1, serif
│                                             │
│    ▐                                        │
│    ▐  ▐                          ▐          │
│    ▐  ▐     ▐        ▐     ▐    ▐    ▐     │  ← vertical bars, CSS flex
│    ▐  ▐  ▐  ▐  ▐  ▐  ▐  ▐  ▐   ▐  ▐ ▐  ▐  │
│    ▐  ▐  ▐  ▐  ▐  ▐  ▐  ▐  ▐   ▐  ▐ ▐  ▐  │
│   ─────────────────────────────────────────  │
│   W1 W2 W3 W4 W5 W6 W7 W8 W9 ...          │  ← .xs labels, rotated
│                                             │
│   Peak: W12 (87 commits)                   │  ← .small .muted
│                                             │
└─────────────────────────────────────────────┘
```

- Vertical bar chart built with CSS flexbox (`align-items: flex-end`).
- Each bar's height is set via `--bar-h` custom property, calculated as a
  percentage of the maximum week's commits.
- The peak week's bar is highlighted with `--accent` colour; others use
  `--border` with a darker fill.
- Bars animate from `height: 0` to their target height with stagger delays.
- Week labels on the x-axis are rotated 45° for space efficiency when there are
  many weeks.

**Peak annotation behaviour:**

- The bar with the highest commit count is highlighted with `--accent` colour
  and a label annotation showing the week identifier and commit count (e.g.
  "W12 — 87 commits").
- If multiple bars tie for the highest count, the **earliest** (leftmost /
  chronologically first) bar is highlighted.
- If the timeline has zero entries (no commit data), no peak annotation is shown.

#### Slide 4: Type Breakdown

```
┌─────────────────────────────────────────────┐
│                                             │
│    Commit Types                             │  ← .h1, serif
│                                             │
│    feat ████████████████████░░░  62%  (774) │
│    fix  ████████░░░░░░░░░░░░░░  18%  (225) │
│    docs ████░░░░░░░░░░░░░░░░░░   8%  (100) │
│    refactor ███░░░░░░░░░░░░░░░   5%   (62) │
│    test ██░░░░░░░░░░░░░░░░░░░░   3%   (37) │
│    chore █░░░░░░░░░░░░░░░░░░░░   2%   (25) │
│    other █░░░░░░░░░░░░░░░░░░░░   2%   (24) │
│                                             │
└─────────────────────────────────────────────┘
```

- Horizontal bar chart with colour-coded bars matching `--feat`, `--fix`, etc.
- Each row: type label (zh-TW, e.g. "功能", "修復"), coloured bar, percentage,
  and absolute count.
- Bars animate width from 0 to target percentage on scroll entry.
- Labels use `.small` size, counts use `.metric` style with `tabular-nums`.

**Bar CSS:**

```css
.type-bar-track {
  background: var(--border);
  border-radius: 4px;
  height: 28px;
  flex: 1;
  overflow: hidden;
}

.type-bar-fill {
  height: 100%;
  border-radius: 4px;
  width: 0;
  transition: width 0.6s cubic-bezier(0.22, 1, 0.36, 1);
}

.type-bar-fill.in-view {
  width: var(--bar-w);
}
```

#### Slide 5: Projects

```
┌─────────────────────────────────────────────┐
│                                             │
│    Projects                                 │  ← .h1, serif
│                                             │
│   ┌─────────────────┐  ┌─────────────────┐  │
│   │ my-project-Platform-    │  │ my-frontend    │  │
│   │ Core             │  │                 │  │
│   │                  │  │                 │  │
│   │ 423 commits      │  │ 312 commits     │  │
│   │ 8 proposals      │  │ 5 proposals     │  │
│   │ +12,340 / -3,201 │  │ +8,920 / -2,100│  │
│   │ [feat] [fix]     │  │ [feat] [docs]   │  │
│   └─────────────────┘  └─────────────────┘  │
│                                             │
│   ┌─────────────────┐  ┌─────────────────┐  │
│   │ ...             │  │ ...             │  │
│   └─────────────────┘  └─────────────────┘  │
│                                             │
└─────────────────────────────────────────────┘
```

- Card grid: 2 columns (desktop and tablet), single column on mobile.
- Each card shows: project name (`.h2`), commit count, proposal count, line
  stats (`+added` in green, `-removed` in red), and top commit type badges.
- Cards use `--surface` background, `--border` border, `--radius` corners.
- Fade-in with stagger animation.

#### Slide 6: Proposals (Scrollable)

```
┌─────────────────────────────────────────────┐
│    OpenSpec Proposals                       │  ← .h1, serif
│   ┌─────────────────────────────────────┐   │
│   │ Project     Slug      Date   Tasks  │   │  ← scrollable table
│   │─────────────────────────────────────│   │
│   │ Platform-   auth-     01-15    5    │   │
│   │ Core        redesign                │   │
│   │ Platform-   event-    02-03    3    │   │
│   │ Core        router                  │   │
│   │ Frontend    e-map-    01-20    8    │   │
│   │             perf                    │   │
│   │ ...                                 │   │
│   │                                     │   │  ← internal scroll
│   └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

- Uses `.slide--scrollable` variant with internal overflow.
- Table grouped by project with subtle group headers.
- Clean row design: alternating row background (`--bg` / `--surface`),
  hover highlight.
- Columns: Project, Slug, Date, Description (truncated), Task Count.

**Table CSS:**

```css
.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--fs-small);
}

.data-table th {
  text-align: left;
  padding: var(--gap-xs) var(--gap-sm);
  border-bottom: 2px solid var(--fg);
  font-weight: 600;
}

.data-table td {
  padding: var(--gap-xs) var(--gap-sm);
  border-bottom: 1px solid var(--border);
  vertical-align: top;
}

.data-table tr:hover td {
  background: var(--bg);
}
```

#### Slide 7: Commit Log (Scrollable)

```
┌─────────────────────────────────────────────┐
│    Commit Log                               │  ← .h1, serif
│   ┌─────────────────────────────────────┐   │
│   │ Date ▾   Type   Repo     Message    │   │  ← sortable headers
│   │─────────────────────────────────────│   │
│   │ 12-31  [feat]  Frontend  Add map... │   │
│   │ 12-30  [fix]   Platform  Fix auth.. │   │
│   │ 12-30  [docs]  Wiki      Update...  │   │
│   │ ...                                 │   │
│   │                                     │   │  ← internal scroll
│   │ Showing 100 of 1,247               │   │
│   │ [Show all]                          │   │
│   └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

- Same `.slide--scrollable` variant as Slide 6.
- **Sortable columns**: clicking a column header sorts the table by that column.
  Sort state is tracked in a simple JavaScript object; no external library.
- **Type badges**: inline `<span>` with commit-type colour as background
  (low-opacity) and text colour. E.g.:

```css
.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: var(--fs-xs);
  font-weight: 600;
  letter-spacing: 0.02em;
}

.badge--feat { background: rgba(47, 125, 74, 0.12); color: var(--feat); }
.badge--fix  { background: rgba(181, 58, 42, 0.12); color: var(--fix); }
/* ... other types ... */
```

- **100-row limit** with a "Show all" toggle button that reveals remaining rows.
  Rows beyond 100 are rendered in the HTML but hidden with `display: none` and
  toggled via a simple `classList.toggle` click handler.
- Compact row height, monospace date column, truncated commit messages with
  `text-overflow: ellipsis`.

**Sort implementation:**

```javascript
function sortTable(table, colIndex, direction) {
  var tbody = table.querySelector('tbody');
  var rows = Array.from(tbody.querySelectorAll('tr'));

  rows.sort(function(a, b) {
    var aText = a.children[colIndex].textContent.trim();
    var bText = b.children[colIndex].textContent.trim();
    if (direction === 'asc') return aText.localeCompare(bText);
    return bText.localeCompare(aText);
  });

  rows.forEach(function(row) {
    tbody.appendChild(row);
  });
}

document.querySelectorAll('.sortable-header').forEach(function(header) {
  header.addEventListener('click', function() {
    var table = header.closest('table');
    var colIndex = parseInt(header.getAttribute('data-col'), 10);
    var currentDir = header.getAttribute('data-sort-dir') || 'desc';
    var newDir = currentDir === 'asc' ? 'desc' : 'asc';
    header.setAttribute('data-sort-dir', newDir);

    // Reset other headers
    table.querySelectorAll('.sortable-header').forEach(function(h) {
      if (h !== header) h.removeAttribute('data-sort-dir');
    });

    sortTable(table, colIndex, newDir);
  });
});
```

---

### 7. Average Daily Lines Metric

**Decision:** Add a new KPI — "Average Daily Lines" — that shows the average
number of lines changed (insertions + deletions) per active day.

**Definition:**

```
avg_daily_lines = (total_insertions + total_deletions) / count(unique_commit_dates)
```

Where `unique_commit_dates` is the set of distinct calendar dates (YYYY-MM-DD)
on which at least one commit was made. This measures intensity on active days,
not calendar days (which would dilute the metric over weekends and holidays).

**Rust backend change — `src/model.rs`:**

```rust
// Add to the Summary struct:
pub struct Summary {
    // ... existing fields ...
    pub avg_daily_lines: f64,
}
```

**Rust backend change — `src/collector.rs`:**

```rust
use std::collections::HashSet;

// After collecting all commits:
let unique_dates: HashSet<&str> = commits
    .iter()
    .map(|c| c.date.as_str())  // assuming date is stored as "YYYY-MM-DD"
    .collect();

let total_lines = summary.total_insertions + summary.total_deletions;
let avg_daily_lines = if unique_dates.is_empty() {
    0.0
} else {
    total_lines as f64 / unique_dates.len() as f64
};
```

**Template rendering:**

The metric is serialized to JSON and rendered in the KPI dashboard (Slide 2) as
the 6th card. The counter animation rounds to the nearest integer for display:

```html
<div class="kpi-card animate">
  <div class="metric counter" data-target="158" style="color: var(--accent)">0</div>
  <div class="label small caps">Avg Daily Lines</div>
</div>
```

**Display precision:** The `avg_daily_lines` value is `f64` in JSON but displayed
as a rounded integer (`Math.round()`) in the KPI card. The "行/天" unit label
follows the number in the card's `.label` element.

**Alternative considered:** Average per calendar day (including non-active days).
Rejected because it penalises contributors who don't commit on weekends, making
the metric less meaningful. Active-day average better reflects work intensity.

**Alternative considered:** Median instead of mean. Rejected for simplicity —
the mean is easier to compute in the Rust pipeline and is more intuitive as a
dashboard KPI. Outlier days (massive refactors) are acceptable in a personal
showcase context.

---

### 8. Responsive Behavior

**Decision:** Three breakpoints with progressive simplification. Scroll-snap is
disabled on narrow viewports where slide-height content would overflow.

#### ≥ 1200px — Full Slide Layout

- Scroll-snap active, full 7-slide experience.
- KPI grid: 3 columns (2 rows).
- Project cards: 2 columns.
- Timeline: full-width bar chart.
- Navigation dots visible.

#### 921px – 1199px — Compact Slides

```css
@media (max-width: 1199px) {
  .kpi-grid { grid-template-columns: repeat(2, 1fr); }
  .project-grid { grid-template-columns: repeat(2, 1fr); }
  :root {
    --gap-xl: 40px;
    --gap-2xl: 64px;
  }
}
```

- KPI grid: 2 columns (3 rows).
- Project cards: 2 columns.
- Slightly reduced spacing.
- Scroll-snap still active (content fits in viewport at this size).

#### ≤ 920px — Mobile Flow

```css
@media (max-width: 920px) {
  main {
    scroll-snap-type: none;  /* disable snap on mobile */
  }
  section.slide {
    min-height: auto;  /* natural content height */
    scroll-snap-align: none;
    padding: var(--gap-lg) var(--gap-md);
  }
  .slide-nav {
    display: none;  /* hide dot navigation */
  }
  .kpi-grid { grid-template-columns: 1fr; }
  .project-grid { grid-template-columns: 1fr; }
  section.slide--scrollable .slide-inner {
    max-height: none;
    overflow-y: visible;
  }
}
```

- Scroll-snap disabled — page becomes a normal scrolling document.
- All grids collapse to single column.
- Navigation dots hidden (no slides to navigate).
- Slides become natural-height sections with generous vertical padding.
- Internal scroll containers (slides 6–7) expand to full content height.
- Typography still scales via `clamp()` — no additional font-size overrides
  needed.

**Breakpoint matrix summary:**

| Viewport | KPI Grid | Projects | Scroll-snap | Nav dots |
|---|---|---|---|---|
| ≥ 1200px | 3×2 | 2 columns | ✅ Active | ✅ Visible |
| 921–1199px | 2×3 | 2 columns | ✅ Active | ✅ Visible |
| ≤ 920px | 1 column (stacked) | 1 column | ❌ Disabled | ❌ Hidden |

**Rationale for disabling snap on mobile:** `scroll-snap-type: y mandatory` with
`min-height: 100vh` sections works well on desktop where content fits within the
viewport. On mobile, content (especially grids that stack to single column)
frequently exceeds viewport height, making mandatory snap jarring and
potentially trapping users between slides. Converting to a flowing layout is the
standard responsive pattern for slide-style pages.

**Overflow policy:**
- Project names and proposal slugs: `overflow: hidden; text-overflow: ellipsis; white-space: nowrap` with `title` attribute for full text on hover.
- Commit messages in tables: single-line ellipsis with `max-width: 400px`.
- Timeline week labels: if more than ~26 weeks, show every other label to avoid overlap.

---

### 9. Empty-Data Handling

**Decision:** All 7 slides always render regardless of data content. When data
is absent or zero after filtering, slides display a graceful empty state rather
than being hidden.

**Behaviour per slide:**

| Slide | Empty-data behaviour |
|---|---|
| Cover (Hero) | Displays "0" for the hero total-commits metric. |
| KPI Dashboard | All KPI cards show 0 values; counter animation still runs (animates to 0). |
| Timeline | Shows a centred message: "無提交記錄" (no commit data). No bars rendered. No peak annotation. |
| Type Breakdown | Shows a centred empty-state message. No bars rendered. |
| Projects | Each project card shows 0 commits / 0 proposals. If the project list itself is empty, a "無專案資料" message is shown. |
| Proposals | Shows a centred message: "無提案紀錄" (no proposal data). Table header row is still rendered. |
| Commit Log | Shows "無提交記錄" message. Table header row is still rendered. |

**Rationale:** Hiding slides when data is empty would break the 7-slide
navigation model (dot nav expects exactly 7 slides) and surprise users with a
shorter-than-expected presentation. Showing zero/empty states is honest and
consistent.

---

## Risks / Trade-offs

### 1. Scroll-snap can feel jarring

**Risk:** Users unfamiliar with slide-style scrolling may be confused by the
"sticky" snap behaviour, especially with trackpad momentum scrolling.

**Mitigation:**
- The cover slide includes a visible scroll indicator (animated chevron) to
  signal that scrolling is the primary interaction.
- `scroll-behavior: smooth` on the `<main>` container provides eased snapping
  rather than abrupt jumps.
- Scroll-snap is disabled entirely on mobile (≤ 920px), where it causes the
  most confusion.
- The navigation dots provide an escape hatch — users can click directly to any
  slide.

### 2. 100vh slides don't work for variable-height content

**Risk:** Slides 6 (Proposals) and 7 (Commit Log) contain tables of arbitrary
length that will almost always exceed viewport height.

**Mitigation:**
- These two slides use the `.slide--scrollable` variant with an internal
  `overflow-y: auto` container.
- The slide itself still snaps correctly; only its inner content scrolls.
- On mobile, the internal scroll is disabled and content flows naturally.
- Slide 7 additionally limits initial display to 100 rows with a "Show all"
  toggle, preventing extreme scroll depths.

### 3. JavaScript readability constraint

**Risk:** Template JavaScript uses ES5-style `var`/`function` for readability.
Future contributors might introduce modern syntax inconsistently.

**Mitigation:**
- The JS style choice (ES5-style) is documented in the Context section's
  constraint table.
- All JavaScript examples in this document use `var`, `function(){}`, and
  `Array.from()` / `forEach()` consistently.
- Code review should enforce stylistic consistency within the template.

### 4. Template file size

**Risk:** The redesigned template will be significantly larger than the current
version — estimated at 1000–1500 lines of combined HTML/CSS/JS in the Rust
template string.

**Mitigation:**
- The template remains a single self-contained file with no external
  dependencies, which is a project constraint regardless of size.
- CSS and JS sections are clearly delineated with comment headers for
  maintainability.
- The design system's token-based approach (CSS custom properties) reduces
  repetition — changing a colour or spacing value requires editing one line.
- Future proposal: if the template grows beyond ~2000 lines, consider splitting
  the Rust template into logical sections (CSS, JS, HTML partials) that are
  concatenated at compile time. This is out of scope for this change.

### 5. No serif fonts guaranteed on all systems

**Risk:** The display typeface stack (`'Iowan Old Style', 'Charter', Georgia,
serif`) relies on system fonts that may not be present on all operating systems.
Iowan Old Style is macOS-only; Charter may not be installed on Windows or Linux.

**Mitigation:**
- Georgia is the third fallback and is available on virtually all systems
  (Windows, macOS, most Linux desktop distributions).
- The generic `serif` catch-all ensures *some* serif font is always used.
- The design's visual quality is not dependent on Iowan Old Style specifically —
  the tight letter-spacing and weight choices look good with any quality serif.
- Web font embedding was considered and rejected due to the self-contained /
  air-gapped constraint (no external resources, and base64-encoding a font would
  add 50–200KB to the file).
