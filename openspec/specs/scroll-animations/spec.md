# Scroll Animations

## Purpose

Provides scroll-triggered animations for dashboard elements, including fade-in
entrances, numeric counter animations, and bar chart growth effects. All
animations are driven by `IntersectionObserver` and respect the user's
reduced-motion preference.

## Requirements

### Requirement: Fade-In Entrance Animation

Elements marked for animation SHALL transition from invisible
(`opacity: 0; transform: translateY(24px)`) to visible
(`opacity: 1; transform: translateY(0)`) when they scroll into view. The
24px vertical offset has been validated against the 18px minimum typography
floor and remains proportionally appropriate.

#### Scenario: Section enters viewport
- **GIVEN** a section with the animation marker class
- **WHEN** the section scrolls into the viewport (threshold ≥ 0.2)
- **THEN** the element fades in and slides up over approximately 400ms

### Requirement: Counter Animation

Numeric metric values SHALL animate from zero to their final value when their
container scrolls into view, using eased interpolation over approximately
1200ms.

#### Scenario: KPI card enters viewport
- **GIVEN** a KPI card displaying "451"
- **WHEN** the card's section scrolls into view
- **THEN** the displayed number animates from 0 to 451 with easing

### Requirement: Locale-Aware Number Formatting

Counter animations SHALL format numbers using `toLocaleString('zh-TW')` so
that large values include appropriate thousands separators.

#### Scenario: Counter target is 12345
- **GIVEN** a counter target value of 12345
- **WHEN** the counter animation completes
- **THEN** the displayed text is `"12,345"` (or equivalent zh-TW formatted string)

### Requirement: Bar Growth Animation

Bar chart elements SHALL animate their width or height from zero to their
target dimension when they scroll into view.

#### Scenario: Timeline bars enter viewport
- **GIVEN** a set of timeline bars with varying heights
- **WHEN** the timeline section scrolls into view
- **THEN** each bar grows from zero height to its target height

### Requirement: Stagger Effect

When multiple sibling elements animate in, each successive element SHALL begin
its animation approximately 80–100ms after the previous one.

#### Scenario: Six KPI cards animate in
- **GIVEN** six KPI metric cards in a grid
- **WHEN** their section scrolls into view
- **THEN** card 1 starts first, card 2 starts ~100ms later, and so on

### Requirement: One-Shot Trigger

Each animated element SHALL trigger its animation only once. After the initial
play, the observer SHALL stop observing that element.

#### Scenario: User scrolls past and back
- **GIVEN** a section that has already played its entrance animation
- **WHEN** the user scrolls away and then returns
- **THEN** the section remains in its final animated state without replaying

### Requirement: Reduced-Motion Preference

When the user's operating system requests reduced motion
(`prefers-reduced-motion: reduce`), all animations SHALL be skipped and
elements SHALL appear in their final state immediately.

#### Scenario: Reduced motion enabled
- **GIVEN** the user has enabled reduced-motion in OS settings
- **WHEN** the dashboard loads
- **THEN** all elements appear instantly without animation or transition

#### Scenario: Counter with reduced motion
- **GIVEN** the user has enabled reduced-motion in OS settings
- **WHEN** a KPI card enters the viewport
- **THEN** the final numeric value appears immediately without animation
