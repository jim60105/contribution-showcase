## Why

The timeline granularity cascade in `build_timeline()` uses a hardcoded threshold of `14` to decide when to escalate from one time granularity level to the next (e.g., daily → weekly → monthly). This threshold directly controls how many bars appear in the 提交趨勢 chart, but changing it requires modifying source code and recompiling. Exposing it as a configurable parameter lets users tune chart density to their preference — more bars for detailed views or fewer bars for overview — without touching the codebase.

## What Changes

- Add a new optional `timeline_max_buckets` field to the `[output]` section in the TOML config (`OutputConfig` struct), defaulting to `14` when omitted.
- Add a new `--timeline-max-buckets` CLI argument to the `generate` subcommand that overrides the config value.
- Thread this parameter through from `collect()` → `build_timeline()`, replacing every hardcoded `14` comparison in the granularity cascade.
- Update `showcase.example.toml` with a commented-out example of the new field.
- Update the existing `adaptive-timeline-granularity` spec to reflect that the threshold is now a parameter rather than a fixed constant.

## Capabilities

### New Capabilities

- `timeline-bucket-config`: Exposes the timeline granularity threshold as a configurable parameter via TOML config and CLI argument, with a default of 14 that preserves current behavior.

### Modified Capabilities

- `adaptive-timeline-granularity`: The threshold value `14` used in the five-level granularity cascade changes from a hardcoded constant to a runtime parameter. The cascade logic itself and all label formats remain unchanged; only the source of the threshold value changes.

## Impact

- **Code**: `src/config.rs` (new field on `OutputConfig`), `src/main.rs` (new CLI arg, threading to collector), `src/collector.rs` (`build_timeline` signature change to accept threshold, all `14` literals replaced with parameter).
- **Config**: `showcase.example.toml` gains a new commented-out `timeline_max_buckets` field under `[output]`.
- **Tests**: Existing timeline tests in `collector.rs` will need updating to pass the threshold parameter; most tests keep using `14` so assertions stay the same. New tests should verify non-default threshold values.
- **Model**: No changes to `model.rs` — the output data shape is unaffected.
- **Breaking changes**: None. The default value of `14` preserves identical behavior for existing users.
