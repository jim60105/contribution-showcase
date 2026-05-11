## MODIFIED Requirements

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
