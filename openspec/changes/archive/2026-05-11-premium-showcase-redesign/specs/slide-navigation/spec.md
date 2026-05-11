# Slide Navigation

## Purpose

Provides full-viewport slide-based page navigation using CSS scroll-snap,
allowing users to browse the dashboard one section at a time. Includes a fixed
dot indicator rail and keyboard navigation support.

## Requirements

### Requirement: Scroll-Snap Page Layout

The system SHALL render each dashboard section as a full-viewport slide using
CSS `scroll-snap-type: y mandatory` on the scroll container and
`scroll-snap-align: start` on each section.

#### Scenario: User scrolls between sections
- **GIVEN** the dashboard is open in a browser
- **WHEN** the user scrolls past a section boundary
- **THEN** the viewport snaps to align exactly with the next section's top edge

### Requirement: Full-Viewport Section Height

Each slide section SHALL have a minimum height of `100vh` so that it fills the
entire viewport.

#### Scenario: Viewport is 900px tall
- **GIVEN** a browser viewport height of 900px
- **WHEN** the dashboard renders
- **THEN** each slide section is at least 900px tall

### Requirement: Navigation Dot Indicator

The system SHALL display a fixed navigation indicator on the right side of the
viewport with one dot per slide. The dot corresponding to the currently visible
slide SHALL be visually distinguished (larger or filled).

#### Scenario: User is on slide 3
- **GIVEN** the user has scrolled to the third slide
- **WHEN** the navigation indicator is visible
- **THEN** the third dot is visually active and other dots are inactive

### Requirement: Dot Click Navigation

Clicking a navigation dot SHALL smooth-scroll the viewport to the
corresponding slide.

#### Scenario: User clicks dot 5
- **GIVEN** the user is on slide 1
- **WHEN** the user clicks the fifth navigation dot
- **THEN** the viewport smooth-scrolls to slide 5

### Requirement: Keyboard Navigation

The system SHALL support keyboard navigation between slides using ArrowDown,
ArrowUp, PageDown, and PageUp keys.

#### Scenario: ArrowDown from slide 2
- **GIVEN** the user is viewing slide 2
- **WHEN** the user presses the ArrowDown key
- **THEN** the viewport navigates to slide 3

#### Scenario: ArrowUp from slide 1
- **GIVEN** the user is viewing slide 1
- **WHEN** the user presses the ArrowUp key
- **THEN** no navigation occurs (already at first slide)

### Requirement: Active Dot Tracking

The navigation indicator SHALL update the active dot automatically as the user
scrolls, using `IntersectionObserver` to detect which section is in view.

#### Scenario: User manually scrolls to slide 4
- **GIVEN** the user scrolls (not via dot click) to slide 4
- **WHEN** slide 4 enters the viewport
- **THEN** the fourth dot becomes active without user interaction

### Requirement: Keyboard Event Scope

The system SHALL only handle slide-navigation keyboard events (ArrowUp,
ArrowDown, PageUp, PageDown) when the event target is the document body, the
`<main>` scroll container, or a slide section. Events originating from
interactive elements (buttons, links, inputs) or nested scrollable containers
SHALL NOT trigger slide navigation.

#### Scenario: ArrowDown inside scrollable table
- **GIVEN** the user has focused inside the commit log slide's scrollable
  table container
- **WHEN** the user presses ArrowDown
- **THEN** the table container scrolls down and the slide does not change

#### Scenario: ArrowDown on body
- **GIVEN** focus is on the document body
- **WHEN** the user presses ArrowDown
- **THEN** the viewport navigates to the next slide

### Requirement: Navigation ARIA State

The active navigation dot SHALL carry `aria-current="true"`. When the active
slide changes, the previous dot's `aria-current` attribute SHALL be removed
and set on the new active dot.

#### Scenario: Slide changes from 2 to 3
- **GIVEN** the user navigates from slide 2 to slide 3
- **WHEN** the navigation indicator updates
- **THEN** dot 3 has `aria-current="true"` and dot 2 does not

### Requirement: Mobile Responsive Behaviour

On viewports narrower than 920px, scroll-snap SHALL be disabled and the
navigation dots SHALL be hidden. Sections become normal flowing content.

#### Scenario: Viewport is 768px wide
- **GIVEN** a browser viewport width of 768px
- **WHEN** the dashboard renders
- **THEN** scroll-snap is inactive and navigation dots are not visible
