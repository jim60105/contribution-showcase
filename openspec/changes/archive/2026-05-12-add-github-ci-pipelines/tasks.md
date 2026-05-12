## 1. Cargo.toml Metadata

- [x] 1.1 Update `Cargo.toml` with required crates.io metadata: add `description`, `license` (GPL-3.0-only, matching the existing LICENSE file), `repository` (GitHub URL), and optionally `homepage`, `keywords`, `categories` fields

## 2. Project Setup

- [x] 2.1 Create `.github/workflows/` directory structure in the repository root

## 3. Changelog

- [x] 3.1 Create `CHANGELOG.md` in the repository root using Keep a Changelog format, with an initial `## [Unreleased]` section and a header linking to https://keepachangelog.com

## 4. CI Workflow

- [x] 4.1 Create `.github/workflows/build-test-audit.yml` with three parallel jobs: **check** (ubuntu-latest — `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings` using `dtolnay/rust-toolchain@stable` with `rustfmt,clippy` components), **test** (matrix: ubuntu/windows/macos, stable Rust — `cargo test` with `actions/cache@v5` keyed on OS + `Cargo.lock` hash for `~/.cargo/registry`, `~/.cargo/git`, and `target/`), and **audit** (ubuntu-latest — `actions-rust-lang/audit@v1.2.7`). Triggers: push to `master`/`dev`, PRs targeting `master`/`dev`, with `paths-ignore: '**/*.md'`

## 5. Release Workflow

- [x] 5.1 Create `.github/workflows/release.yml` triggered on push tags matching `v*` with `permissions: contents: write` and four sequential stages: **validate-tag** (verify tag matches SemVer regex `^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$`, extract version from tag and compare against `version` in `Cargo.toml` to ensure they match), **create-release** (extract changelog section for the tag version from `CHANGELOG.md` using `awk`, write extracted notes to a file, fallback to `"Release $VERSION"`, create GitHub Release via `softprops/action-gh-release@v3` using `body_path` to supply release notes, mark pre-release if tag contains `-`), **build** (matrix over 6 targets — `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` with cross-compilation setup, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`, `x86_64-apple-darwin`, `aarch64-apple-darwin` — build with `cargo build --release --target`, rename binary to `contribution-showcase-<target>[.exe]`, upload via `softprops/action-gh-release@v3`), and **publish-crates** (conditional on tag not containing `-`, run `cargo publish --token ${{ secrets.CARGO_REGISTRY_TOKEN }}`)

## 6. Validation

- [x] 6.1 Verify CI workflow: confirm YAML is valid, triggers are correct for push/PR on `master`/`dev`, all three jobs are defined with correct steps and caching config
- [x] 6.2 Verify Release workflow: confirm YAML is valid, tag trigger pattern is correct, SemVer tag validation and tag-vs-Cargo.toml version matching logic are correct, changelog extraction writes to a file and `body_path` is used, all 6 build targets are present with correct OS/linker settings, and publish-crates conditional is correct
- [x] 6.3 Verify `cargo publish --dry-run` succeeds with the updated `Cargo.toml` metadata
