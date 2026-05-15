## Context

The `generate` subcommand currently exposes six CLI flags: `--config`,
`--output`, `--author`, `--since`, `--until`, and `--timeline-max-buckets`.
Three config-only settings have no CLI counterpart: `title`,
`filters.types`, and `filters.exclude_hashes`. The existing override pattern
in `run_generate()` is straightforward: if the CLI `Option<T>` is `Some`,
it replaces the corresponding config field before validation runs.

## Goals / Non-Goals

**Goals:**

- Add `--title <TITLE>` flag that overrides `config.title`.
- Add `--types <TYPES>` flag (comma-separated) that overrides
  `config.filters.types`.
- Add `--exclude-hashes <HASHES>` flag (comma-separated) that overrides
  `config.filters.exclude_hashes`.
- Follow the identical override pattern used by existing flags.
- Add tests for each new flag.

**Non-Goals:**

- Adding CLI flags for project-level fields (`projects[].name`, `.path`,
  `.branch`, etc.). These are structurally complex and belong in the config.
- Changing how filters are applied in the collector.
- Changing config file format or defaults.

## Decisions

### 1. Comma-separated strings for list flags

**Decision**: Accept `--types` and `--exclude-hashes` as single
comma-separated strings (e.g., `--types feat,fix,docs`).

**Rationale**: This is the simplest UX for short lists. Clap supports
`num_args(1..)` for space-separated multiple values, but comma-separated
is more conventional for filter lists and avoids shell quoting issues.

**Alternative**: `--types feat --types fix` (clap `append`). Rejected as
more verbose for no benefit.

### 2. Parse comma-separated values into `Vec<String>` in `run_generate()`

**Decision**: Split the raw CLI string on commas, trim whitespace, filter
empty segments, and assign to the config field. If the resulting vector is
empty, treat the flag as absent (do not override the config value).

**Rationale**: Keeps the `Generate` struct simple (`Option<String>`) and
parsing co-located with the other override logic.

### 3. `--title` is a plain string flag

**Decision**: `--title` takes a single string value, same as `--author`.

**Rationale**: Title is always a single value. No special parsing needed.

## Risks / Trade-offs

- **[Comma in values]** → Commit hashes never contain commas. Commit types
  are single lowercase words. No risk of accidental splitting.
- **[Empty string]** → `--title ""` sets an empty title, which falls through
  to the default "貢獻總覽" in the collector. This is consistent behaviour.
