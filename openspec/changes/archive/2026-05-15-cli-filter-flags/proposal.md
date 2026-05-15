## Why

Three config-only settings—`title`, `filters.types`, and
`filters.exclude_hashes`—have no CLI counterpart. Users who want a quick
one-off run with a different title or type filter must edit the TOML file
instead of passing a flag. Adding CLI flags for these settings brings feature
parity and supports scripting / CI workflows where config files are shared but
run-time parameters differ.

## What Changes

- Add `--title` flag to the `generate` subcommand. When provided, it overrides
  the TOML `title` value. This affects both the HTML page title and the
  default output filename.
- Add `--types` flag (comma-separated list, e.g. `--types feat,fix,docs`) to
  override `filters.types`. Only commits matching the listed types are
  included.
- Add `--exclude-hashes` flag (comma-separated list) to override
  `filters.exclude_hashes`. Commits whose hash matches any listed value
  are excluded.
- All three follow the existing override pattern: CLI value replaces config
  value when present. List flags that parse to an empty list after
  trimming are treated as absent (do not override the config value).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `rust-cli-scaffold`: Add three new CLI flags (`--title`, `--types`,
  `--exclude-hashes`) to the `generate` subcommand definition and override
  logic.
- `toml-config-loader`: Document that the three new CLI flags override their
  respective config fields using the same semantics as existing flags.

## Impact

- `src/main.rs` — `Generate` struct gains three fields; `run_generate()` merge
  logic extended with three additional override branches.
- No changes to `src/config.rs`, `src/collector.rs`, or `src/model.rs`. The
  existing `FilterConfig` and `Config` structs already support these fields;
  only the CLI-to-config bridging logic changes.
- `showcase.example.toml` — add comments noting CLI override availability.
