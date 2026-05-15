# Design: Configurable Timeline Bucket Threshold

## Context

`build_timeline()` in `src/collector.rs` implements a five-level granularity cascade (daily → weekly → monthly → quarterly → yearly) that decides how many bars appear in the timeline chart. Every level transition compares the distinct bucket count against a hardcoded literal `14` — five occurrences total. This constant controls chart density but is buried in the implementation, requiring a recompile to change. The goal is to surface it as a runtime parameter through both the TOML config and a CLI argument while preserving the current default.

`OutputConfig` currently holds only `path: Option<String>`. The `collect()` function receives the entire `Config` by reference and delegates to `build_timeline()`, which today takes only `&[CommitEntry]`. The CLI `Generate` struct already follows the pattern of optional override fields (`--output`, `--author`, `--since`, `--until`) that layer on top of config values.

## Goals / Non-Goals

### Goals

- Expose the granularity threshold as `timeline_max_buckets` in the `[output]` TOML section and as `--timeline-max-buckets` in the `generate` CLI.
- CLI argument overrides the config value; omitting both yields `14`.
- Replace all five hardcoded `14` comparisons in `build_timeline()` with the parameter.
- Update `showcase.example.toml` with a commented-out example.
- Update the `adaptive-timeline-granularity` spec to reflect parameterisation.

### Non-Goals

- Changing the cascade logic itself (level order, label formats, bucket aggregation).
- Per-project threshold overrides — the threshold applies globally to the merged timeline.
- Exposing other chart tuning knobs (bar width, colour thresholds, etc.).
- Supporting dynamic/auto-tuning algorithms that pick the threshold at runtime.

## Decisions

### 1. Config placement: `[output]` section, not a new `[timeline]` section

Place `timeline_max_buckets` on `OutputConfig` under the existing `[output]` table.

**Rationale**: The timeline is a rendering concern — it controls how the generated HTML chart looks, not what data is collected. `[output]` already governs the output artifact (`path`), so this is the natural home. Introducing a dedicated `[timeline]` top-level section for a single field would add structural complexity without proportional benefit. If more timeline knobs are added later, a `[timeline]` sub-table can be introduced as a non-breaking extension at that point.

**Alternative considered**: A `[timeline]` top-level section. Rejected because one-field sections create config sprawl and there is no concrete second field on the horizon. YAGNI applies.

### 2. Threading: add parameter to `build_timeline()`, resolve before calling

Add `max_buckets: usize` as a parameter to `build_timeline()`. The resolved value (CLI override > config > default 14) is computed in `run_generate()` and stored on `Config` or `OutputConfig`, then read in `collect()` and passed to `build_timeline()`.

**Rationale**: `build_timeline()` is a pure function that transforms commits into timeline entries — passing the threshold directly keeps it self-contained and easy to test in isolation with different values. `collect()` already receives `&Config`, so it simply reads `config.output.timeline_max_buckets` and forwards it. This avoids threading an unrelated concern through intermediate layers and keeps the function signature explicit about its inputs.

**Alternative considered**: Making `build_timeline()` accept the full `Config` reference. Rejected because it couples a focused function to the entire config surface, making tests needlessly verbose and obscuring which config fields actually matter.

### 3. Validation: minimum of 1, enforced at config load and CLI parse time

Accept any `usize` value ≥ 1. Values of 0 are rejected with a clear error message at config load time (in `Config::load()`) and, for the CLI path, via clap's `value_parser!(clap::value_parser!(usize)).range(1..)`.

**Rationale**: A threshold of 0 would mean "no buckets fit at any granularity", which is meaningless — the cascade would always escalate past yearly and produce no output or degenerate results. A minimum of 1 guarantees at least one bucket is possible, so the cascade always terminates at a meaningful level. There is no practical upper bound worth enforcing; a value of 1000 simply means the cascade stays at daily granularity, which is valid if unusual.

**Alternative considered**: Accepting 0 and treating it as "use default". Rejected because silent fallback behaviour is surprising and harder to debug. Explicit validation with an early error is more helpful.

### 4. Naming: `timeline_max_buckets`

Use `timeline_max_buckets` for the TOML field and `--timeline-max-buckets` for the CLI flag.

**Rationale**: The name describes what the parameter controls — the maximum number of buckets before the granularity escalates. The `timeline_` prefix scopes it clearly within `[output]`, avoiding ambiguity if other max-bucket concepts are added later. It matches the mental model of the cascade: "if the number of buckets exceeds this max, try the next coarser granularity".

**Alternatives considered**:
- `max_timeline_entries`: Could be confused with limiting the output array length rather than controlling granularity selection.
- `granularity_threshold`: Accurate but less intuitive — "threshold" doesn't convey that it's about bucket count.
- `timeline_bucket_limit`: Reasonable synonym but "max" is more conventional in Rust APIs (e.g., `max_connections`, `max_retries`).

### 5. Default resolution order: CLI > config > hardcoded 14

Follow the existing override pattern established by `--output`, `--author`, `--since`, `--until`: the CLI value, if provided, takes precedence over the config file value. If neither is set, the hardcoded default of `14` applies. This is implemented by using `Option<usize>` in both `OutputConfig` and `Generate`, merging in `run_generate()` with:

```rust
let output = config.output.get_or_insert_with(Default::default);
if let Some(n) = args.timeline_max_buckets {
    output.timeline_max_buckets = Some(n);
}
```

The helper `Config::timeline_max_buckets()` then resolves via `.unwrap_or(DEFAULT_TIMELINE_MAX_BUCKETS)` where `DEFAULT_TIMELINE_MAX_BUCKETS` is a named constant set to `14`.

## Risks / Trade-offs

**[Risk: Extreme values produce unhelpful charts]** → The tool accepts any value ≥ 1 without warning. A threshold of 1 forces yearly granularity on almost any dataset, while 10000 forces daily granularity across decades. This is intentional — the parameter is a power-user knob, and constraining it further would impose opinionated chart aesthetics. The default of 14 remains the recommended value in the example config comment.

**[Risk: Signature change breaks existing tests]** → `build_timeline()` gains a new parameter, so every existing call site in tests must be updated to pass a value. Mitigation: all existing tests pass `14` as the threshold, making the change mechanical with no assertion changes required.

**[Risk: Type mismatch in cascade comparisons]** → Existing counts use mixed types (`day_count: i64`, `month_count: i32`, `quarter_count: i32`, `week_set.len(): usize`). The new `max_buckets: usize` parameter requires casts at each comparison site. Mitigation: cast `max_buckets` to the appropriate type at each comparison point (e.g., `day_count <= max_buckets as i64`, `month_count <= max_buckets as i32`). Since the threshold is validated ≥ 1 and any practical value fits comfortably in all target types, overflow is not a concern.

**[Risk: Future config expansion may outgrow `[output]`]** → If several more timeline-related fields are added, `[output]` could become a grab-bag. Mitigation: a future change can introduce a `[output.timeline]` nested table as a non-breaking addition, moving `timeline_max_buckets` under it with a deprecation alias on the old path. This is not worth doing preemptively for one field.
