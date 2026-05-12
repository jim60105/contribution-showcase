# TOML Config Loader — Delta

## MODIFIED Requirements

### Requirement: Relative Path Resolution

The system SHALL resolve project and output paths relative to the config file directory only when the configured path is neither absolute nor rooted.

#### Scenario: Rooted project path remains unchanged

- **GIVEN** a config project path that is rooted (for example `/opt/repos/foo` on Windows semantics)
- **WHEN** the config is loaded
- **THEN** the loader SHALL NOT prefix the config directory
- **AND** the original rooted value SHALL be preserved
