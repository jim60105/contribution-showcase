## 1. Config Layer

- [x] 1.1 Define a `const DEFAULT_TIMELINE_MAX_BUCKETS: usize = 14` constant in `src/config.rs`
- [x] 1.2 Add `timeline_max_buckets: Option<usize>` field to `OutputConfig` in `src/config.rs` with `serde(default)` deserialization
- [x] 1.3 Add validation in `Config::load()`: if `timeline_max_buckets` is `Some(0)`, bail with a clear error message ("timeline_max_buckets must be ≥ 1")
- [x] 1.4 Add a helper method `Config::timeline_max_buckets(&self) -> usize` that returns the configured value or `DEFAULT_TIMELINE_MAX_BUCKETS`
- [x] 1.5 Add unit tests for config loading with `timeline_max_buckets` present, absent, and invalid (0)

## 2. CLI Layer

- [x] 2.1 Add `--timeline-max-buckets` optional argument to the `Generate` struct in `src/main.rs` with `clap(long, value_parser = clap::value_parser!(usize).range(1..))` for ≥ 1 validation
- [x] 2.2 In `run_generate()`, apply CLI override: if `args.timeline_max_buckets` is `Some`, write it into `config.output.timeline_max_buckets`
- [x] 2.3 Add unit tests for CLI argument parsing and override behavior

## 3. Collector Integration

- [x] 3.1 Change `build_timeline()` signature from `fn build_timeline(commits: &[CommitEntry]) -> Vec<TimelineEntry>` to `fn build_timeline(commits: &[CommitEntry], max_buckets: usize) -> Vec<TimelineEntry>`
- [x] 3.2 Replace all five hardcoded `14` comparisons in the granularity cascade with the `max_buckets` parameter
- [x] 3.3 Update `collect()` to read `config.timeline_max_buckets()` and pass it to `build_timeline()`
- [x] 3.4 Update all existing `build_timeline()` call sites in tests to pass `14` as the second argument (mechanical, no assertion changes)

## 4. Tests for Non-Default Thresholds

- [x] 4.1 Add test: threshold of 7 escalates daily to weekly sooner (e.g., 10-day span uses weekly instead of daily)
- [x] 4.2 Add test: threshold of 21 keeps daily granularity longer (e.g., 20-day span stays daily)
- [x] 4.3 Add test: threshold of 1 forces yearly for most spans
- [x] 4.4 Add test: threshold of 7 escalates weekly to monthly sooner

## 5. Example Config and Spec Update

- [x] 5.1 Add a commented-out `timeline_max_buckets` entry to `showcase.example.toml` under the `[output]` section with explanatory comment
- [x] 5.2 Update `openspec/specs/adaptive-timeline-granularity/spec.md` to replace hardcoded `14` references with `timeline_max_buckets` parameter language

## 6. Verification

- [x] 6.1 Run `cargo fmt -- --check` to verify formatting
- [x] 6.2 Run `cargo clippy` to verify no new warnings
- [x] 6.3 Run `cargo test` to verify all tests pass (existing + new)
