## Context

Windows CI failures are caused by a combination of rooted-path detection differences and Windows path escaping in TOML basic strings used by tests.

## Goals / Non-Goals

### Goals

- Ensure config path resolution does not rewrite rooted paths as config-relative paths.
- Ensure tests generate valid TOML on Windows when embedding filesystem paths.
- Ensure path assertions do not assume `/` separators.

### Non-Goals

- No changes to report rendering behavior.
- No changes to CLI argument surface.

## Decisions

1. Treat a path as already anchored when either `is_absolute()` or `has_root()` is true before applying config-relative resolution.
2. Use TOML literal strings (`'...'`) in tests for path values built from filesystem paths.
3. Replace separator-specific assertions with `Path`-based suffix checks.

## Risks / Trade-offs

- `has_root()` broadens the set of paths treated as anchored; this is intentional for Windows compatibility.
- TOML literal strings require single quotes in values to be escaped as `''`; test fixtures apply this escaping.
