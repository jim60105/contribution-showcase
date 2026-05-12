## Why

The Windows CI job fails on tests that assume POSIX path behavior and path separators. The failures block the Build, Test & Audit workflow for commit `9052197aee30d90c192aeac629181d1ec14bdf6e`.

## What Changes

- Preserve rooted project and output paths during config loading to avoid incorrectly treating rooted paths as relative on Windows.
- Make test TOML fixtures Windows-safe by avoiding invalid backslash escapes in generated path strings.
- Make path assertions separator-agnostic so tests pass on all supported CI platforms.

## Capabilities

### Modified Capabilities

- `toml-config-loader`: Path resolution behavior is tightened for cross-platform rooted path handling.
- `unit-test-suite`: Test fixtures and assertions become cross-platform safe.

## Impact

- `src/config.rs`
- `src/main.rs` tests
- `src/collector.rs` tests
