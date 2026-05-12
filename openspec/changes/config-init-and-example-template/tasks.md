# Tasks — config-init-and-example-template

## 1. Repository Housekeeping

- [x] 1.1 Create `showcase.example.toml` with placeholder projects, inline comments documenting every field (including optional `coverage_command`, `coverage_result_path`), and commented-out optional filters (`author`, `since`, `until`, `types`, `exclude_hashes`)
- [x] 1.2 Add `/showcase.toml` to `.gitignore`
- [x] 1.3 Remove `showcase.toml` from version control (`git rm --cached showcase.toml`)

## 2. CLI Restructuring

- [x] 2.1 Refactor clap structs: create `Subcommand` enum with `Generate` and `Init` variants using clap derive `Subcommand`
- [x] 2.2 Move current flags (`--config`, `--output`, `--author`, `--since`, `--until`) under the `Generate` variant
- [x] 2.3 Add `Init` variant with optional `--output` flag (default `showcase.toml`)
- [x] 2.4 Configure bare invocation (no subcommand) to print help text
- [x] 2.5 Update `main()` to match on subcommand and dispatch to `generate` / `init` handlers

## 3. Init Subcommand Implementation

- [x] 3.1 Embed template content via `include_str!("../showcase.example.toml")`
- [x] 3.2 Implement overwrite guard using atomic `OpenOptions::create_new(true)` — print error to stderr and exit non-zero if file exists
- [x] 3.3 Create parent directories if needed, then write template content to target path on success
- [x] 3.4 Print success message to stderr with written file path and "edit before running generate" hint

## 4. Test Updates

- [x] 4.1 Update existing CLI tests to use `generate` subcommand form
- [x] 4.2 Add tests for `init` subcommand: default output path, custom `--output`, overwrite guard behaviour
- [x] 4.3 Add test for bare invocation printing help
- [x] 4.4 Verify embedded template content contains required sections (`title`, `output`, `filters`, `projects`) and parses as valid `Config`
