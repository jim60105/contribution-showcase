## 1. Add CLI flag fields to Generate struct

- [x] 1.1 Add `title: Option<String>` field with `--title` long flag and help text to the `Generate` struct in `src/main.rs`
- [x] 1.2 Add `types: Option<String>` field with `--types` long flag and help text describing comma-separated format
- [x] 1.3 Add `exclude_hashes: Option<String>` field with `--exclude-hashes` long flag and help text describing comma-separated format

## 2. Add override logic in run_generate()

- [x] 2.1 Add title override: if `args.title` is `Some`, set `config.title = Some(value)`
- [x] 2.2 Add types override: if `args.types` is `Some`, split on commas, trim whitespace, filter empties, lowercase, and set `config.filters.types = Some(vec)`
- [x] 2.3 Add exclude_hashes override: if `args.exclude_hashes` is `Some`, split on commas, trim whitespace, filter empties, lowercase, and set `config.filters.exclude_hashes = Some(vec)`

## 3. Add tests

- [x] 3.1 Add unit test: `--title` overrides config title and appears in generated HTML `<title>`
- [x] 3.2 Add unit test: `--types` filters commits to only matching types
- [x] 3.3 Add unit test: `--exclude-hashes` excludes commits with matching full hashes
- [x] 3.4 Add unit test: comma-separated parsing handles whitespace and trailing commas; empty result does not override config
- [x] 3.5 Add unit test: `--title` affects default output filename when `[output].path` is not set
- [x] 3.6 Add unit test: `parse_csv_list` lowercases and trims correctly

## 4. Verification

- [x] 4.1 Run `cargo test` — all 220 tests pass
- [x] 4.2 Run `cargo fmt -- --check` and `cargo clippy` — clean
- [x] 4.3 Verify `cargo run -- generate --help` shows all three new flags
