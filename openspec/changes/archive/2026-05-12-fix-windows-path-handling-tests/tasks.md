## 1. Config path resolution

- [x] 1.1 Update `Config::load` path resolution in `src/config.rs` to avoid joining paths that are already absolute or rooted.
- [x] 1.2 Add/adjust tests covering rooted path preservation across platforms.

## 2. Cross-platform test fixtures

- [x] 2.1 Update `src/main.rs` tests to write TOML path fields using literal-string-safe encoding for Windows paths.
- [x] 2.2 Update `src/collector.rs` path assertion to be path-separator-agnostic.

## 3. Validation

- [x] 3.1 Run `cargo fmt -- --check`.
- [x] 3.2 Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] 3.3 Run `cargo test`.
