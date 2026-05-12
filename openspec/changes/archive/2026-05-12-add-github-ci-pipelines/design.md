# Design — add-github-ci-pipelines

## Context

The project has no CI/CD automation. Every quality check — formatting, linting,
testing, security auditing — is run manually by the developer. Releases are
entirely manual: build binaries on each platform, create a GitHub Release, and
publish to crates.io by hand. This is error-prone (easy to forget a platform or
skip a lint step) and does not scale as the project grows or accepts
contributions.

The project is a pure Rust binary with no native/C dependencies, no feature
flags, and no complex build requirements. This simplicity means the CI/CD setup
can be straightforward — no special toolchains, no vendored libraries, no
conditional compilation matrices.

The project follows Gitflow: `master` for releases, `dev` for integration,
with `feature/*` and `fix/*` branches merging into `dev`. CI triggers should
align with this branching model.

## Goals / Non-Goals

**Goals:**

- Automate quality gates (fmt, clippy, test, security audit) on every push to
  `master`/`dev` and on pull requests targeting those branches.
- Automate cross-platform release binary builds for six targets: Linux x86_64,
  Linux aarch64, Windows x86_64, Windows aarch64, macOS x86_64, macOS aarch64.
- Automate GitHub Release creation with changelog-based release notes.
- Automate crates.io publishing for stable releases.
- Support pre-release detection for tags containing `-` (e.g., `v0.2.0-rc.1`).
- Establish a `CHANGELOG.md` as the single source of truth for release notes.

**Non-Goals:**

- Code coverage collection or coverage gates — no coverage infrastructure
  exists and adding it increases complexity without immediate value at this
  stage.
- Docker image builds — the tool is distributed as a standalone binary, not a
  container image.
- Nightly or scheduled builds — the project is not large enough to benefit from
  nightly regression detection.
- cargo-nextest or other alternative test runners — `cargo test` is sufficient
  for the current test suite (~74 tests).
- Quality check scripts or custom linting wrappers — the standard
  `cargo fmt`/`cargo clippy` commands are used directly.
- Smoke tests for cross-compiled binaries (e.g., QEMU for aarch64) — the
  binary is pure Rust with no native dependencies, so cross-compilation
  correctness is high-confidence without runtime validation.

## Decisions

### 1. Two separate workflow files

Split CI and Release into separate files: `build-test-audit.yml` and
`release.yml`.

**Rationale:** The two workflows have different triggers (push/PR vs. tag),
different job structures (matrix testing vs. matrix building + publishing), and
different failure semantics (CI blocks merges; release is a deployment action).
Keeping them separate makes each file easier to reason about and modify
independently.

**Alternative considered:** A single workflow with conditional jobs. Rejected
because it creates a monolithic file where CI changes risk breaking release
logic and vice versa.

### 2. CI trigger scope: master, dev, and PRs targeting them

The CI workflow triggers on:
- `push` to `master` and `dev`
- `pull_request` targeting `master` and `dev`

**Rationale:** This aligns with the project's Gitflow model. Feature branches
get CI feedback when they open PRs against `dev`. Direct pushes to `master` and
`dev` are validated as a safety net. There is no need to trigger CI on every
feature branch push — the PR trigger covers the integration point.

**Alternative considered:** Triggering on all pushes to all branches. Rejected
because it wastes CI minutes on work-in-progress branches where the developer
hasn't requested feedback yet.

### 3. OS matrix: ubuntu-latest, windows-latest, macos-latest

Run `cargo test` across all three major platforms using GitHub-hosted runners
with the `latest` runner image tags.

**Rationale:** The tool uses `std::fs`, `std::path`, and `std::process::Command`
to interact with the filesystem and invoke Git. Path handling and process
spawning behave differently across OSes. Testing on all three catches
platform-specific bugs (e.g., path separators, line endings, command quoting).

Using `-latest` tags keeps the CI on current runner images without manual
maintenance. The trade-off (occasionally breaking due to runner updates) is
acceptable for a project of this size.

**Alternative considered:** Pinning specific runner versions (e.g.,
`ubuntu-24.04`). Rejected because the maintenance burden of tracking runner
deprecations outweighs the stability benefit at this scale.

### 4. Stable Rust toolchain only

Use `dtolnay/rust-toolchain@stable` with no MSRV policy or nightly builds.

**Rationale:** The project has no downstream dependents that need MSRV
guarantees. Building on stable is sufficient to ensure the code compiles and
tests pass. Nightly features are not used.

**Alternative considered:** Adding an MSRV check with a pinned Rust version.
Rejected as premature — there are no users who need to pin their Rust version.
This can be added later if the crate gains adoption.

### 5. Cargo caching with actions/cache

Cache `~/.cargo/registry/index`, `~/.cargo/registry/cache`, and `target/` using
`actions/cache@v5` with a cache key derived from the OS and `Cargo.lock` hash.

**Rationale:** Caching significantly reduces CI build times (from minutes to
seconds for incremental builds). The `Cargo.lock` hash ensures the cache
invalidates when dependencies change.

**Alternative considered:** `Swatinem/rust-cache` (a Rust-specific caching
action). While it handles more edge cases (e.g., cleaning stale artifacts), the
generic `actions/cache` is sufficient for this project's simple dependency tree
and avoids adding another third-party action dependency.

### 6. Security audit with actions-rust-lang/audit

Run `actions-rust-lang/audit@v1.2.7` as a separate job in the CI workflow.

**Rationale:** `cargo audit` checks dependencies against the RustSec Advisory
Database for known vulnerabilities. Running it in CI ensures that vulnerable
dependencies are caught before merging. Using a pinned version (`v1.2.7`)
prevents unexpected breakage from upstream changes.

**Alternative considered:** Running `cargo audit` as a step within the test
job. Rejected because the audit is independent of build/test and should not
block the test matrix if RustSec's database is temporarily unavailable. A
separate job allows the test matrix to succeed even if the audit has transient
failures.

### 7. Lint and format checks as a separate job from testing

Structure the CI workflow with three jobs:
1. **check** — `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings`
   (runs on ubuntu-latest only)
2. **test** — `cargo test` (runs on the OS matrix)
3. **audit** — security audit (runs on ubuntu-latest only)

**Rationale:** Formatting and linting are platform-independent — running them
on one OS is sufficient and avoids wasting CI minutes. Using `--all-targets`
ensures clippy also lints test and example targets, not just the main library
and binary targets. Separating them from tests means developers see lint
failures fast (before the longer test matrix completes). The three jobs run in
parallel, minimizing total wall-clock time.

**Alternative considered:** Running fmt/clippy as steps before `cargo test` in
the same job. Rejected because it serializes feedback (lint errors only appear
after checkout + toolchain setup, and test results are delayed by lint checks).

### 8. Release trigger: tag pattern `v*`

The release workflow triggers on tags matching `v*` (e.g., `v0.1.0`,
`v0.2.0-rc.1`).

**Rationale:** This is the conventional Rust/GitHub pattern for version tags.
The `v` prefix distinguishes version tags from other tags. The wildcard captures
both stable and pre-release versions.

### 9. Pre-release detection via tag name

Tags containing `-` (e.g., `v0.2.0-rc.1`, `v0.3.0-beta.2`) are marked as
pre-releases in the GitHub Release. Stable tags (e.g., `v0.1.0`) are marked as
latest releases.

**Rationale:** This follows semver conventions where pre-release versions
include a hyphen-separated identifier. The detection uses a simple
`contains(github.ref_name, '-')` check, which is readable and requires no
external tooling.

### 10. Changelog extraction with awk

Extract the release notes for the current version from `CHANGELOG.md` using
`awk`, selecting the content between the matching `## [version]` header and the
next `## [` header.

**Rationale:** `awk` is available on all GitHub-hosted runners, requires no
installation, and the extraction logic is a one-liner. The `CHANGELOG.md`
follows the [Keep a Changelog](https://keepachangelog.com/) format, which uses
`## [version]` headers — making awk-based extraction reliable.

The extracted content SHALL be written to a file (`release_notes.md`) and
passed to the release action via `body_path` (not `body`) to handle multiline
content safely.

**Alternative considered:** Using a dedicated changelog parsing tool (e.g.,
`changelog-rs`, a custom script). Rejected because it adds a dependency for a
trivial text extraction task.

### 11. Binary builds: six-target matrix

Build release binaries for:
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `aarch64-unknown-linux-gnu` (Linux aarch64)
- `x86_64-pc-windows-msvc` (Windows x86_64)
- `aarch64-pc-windows-msvc` (Windows aarch64)
- `x86_64-apple-darwin` (macOS x86_64)
- `aarch64-apple-darwin` (macOS aarch64)

Each matrix entry specifies the runner OS and target triple. Native targets
(where the runner OS matches the target) use the default toolchain. The Linux
aarch64 cross-compilation target uses `cross-rs/cross` or the
`gcc-aarch64-linux-gnu` cross-compiler and linker.

**Rationale:** These six targets cover the vast majority of developer
workstations and CI environments. ARM64 Linux is increasingly common (AWS
Graviton, Raspberry Pi, Docker on Apple Silicon). Windows ARM is included
since pure Rust cross-compilation to `aarch64-pc-windows-msvc` requires no
additional toolchain setup on `windows-latest`.

**Alternative considered:** Using `cross` for all targets. Rejected for native
targets because it adds Docker overhead unnecessarily — native compilation is
faster and simpler. `cross` or the system cross-compilation toolchain is only
needed for Linux aarch64 where the runner (x86_64) differs from the target.

### 12. Release asset naming convention

Name release binaries as
`contribution-showcase-<target-triple>[.exe]` (e.g.,
`contribution-showcase-x86_64-unknown-linux-gnu`,
`contribution-showcase-x86_64-pc-windows-msvc.exe`). The naming convention
does **not** include the tag version — the version is already captured by the
GitHub Release itself.

**Rationale:** Using the full target triple in the filename is unambiguous and
follows conventions used by Rust projects (e.g., ripgrep, bat). The `.exe`
extension is included only for Windows targets. Omitting the version from the
filename keeps asset names stable and avoids redundancy with the release tag.

### 13. crates.io publish: stable releases only

The `cargo publish` step is conditional on the tag **not** containing `-`. This
skips crates.io publishing for pre-release tags.

**Rationale:** Pre-release versions on crates.io can confuse users who pin to
the latest version. By skipping pre-releases, only stable, tested versions are
published. The developer can always publish a pre-release manually if needed.

### 14. Release workflow job ordering

The release workflow has three sequential stages:
1. **changelog** — Extracts release notes and detects pre-release status.
2. **release** — Creates the GitHub Release (with the extracted notes) and
   uploads cross-platform binaries built in a matrix.
3. **publish** — Runs `cargo publish` (conditional on stable release).

**Rationale:** The changelog extraction must complete before the release is
created (to provide release notes). Publishing to crates.io must happen after
the release is created to ensure the GitHub Release exists as a reference point.
Sequential ordering enforces these dependencies.

### 15. GitHub Release creation with softprops/action-gh-release

Use `softprops/action-gh-release@v3` to create the GitHub Release and upload
binary assets.

**Rationale:** This is the most widely used GitHub Release action, with
well-documented behavior for pre-release flags, asset uploads, and release
notes. It is maintained and compatible with current GitHub Actions runners.

**Alternative considered:** Using the GitHub CLI (`gh release create`) directly.
Rejected because the action handles asset uploads, pre-release flags, and
idempotency (re-running the workflow updates the existing release) more cleanly
than shell scripting with `gh`.

### 16. CHANGELOG.md format: Keep a Changelog

Adopt the [Keep a Changelog](https://keepachangelog.com/) format with an
`## [Unreleased]` section at the top for ongoing changes.

**Rationale:** This is the most widely recognized changelog format in the Rust
ecosystem. The `## [version]` header structure enables reliable automated
extraction (Decision 10). The `Unreleased` section provides a staging area
for changes that have not yet been released.

The initial `CHANGELOG.md` will include a single `## [Unreleased]` section.
Past changes will not be retroactively documented — the changelog starts from
the point of CI adoption.

### 17. SemVer tag validation

The release workflow SHALL validate that the tag matches a strict SemVer
pattern (`^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$`) before proceeding.

**Rationale:** The `v*` trigger pattern is broad and could match nonsensical
tags like `vtest`. A validation step catches this early with a clear error
message.

**Alternative considered:** Restrict the tag pattern in the `on.push.tags`
filter. Rejected because GitHub Actions tag filters don't support full regex —
only glob patterns like `v[0-9]*` which still allows invalid formats.

### 18. Tag version must match Cargo.toml version

The release workflow SHALL verify that the version in the tag (after stripping
the `v` prefix and any pre-release suffix) matches the `package.version` in
`Cargo.toml`.

**Rationale:** Publishing a mismatched version to crates.io could fail or
create confusion. This catches accidental version mismatches before
building/publishing.

### 19. Cargo.toml metadata for crates.io

The `Cargo.toml` SHALL include required metadata fields for crates.io:
`description`, `license`, `repository`. Optional but recommended: `homepage`,
`keywords`, `categories`.

**Rationale:** `cargo publish` fails without minimum required metadata.
Preparing this upfront prevents release failures.

## Risks / Trade-offs

| Risk | Severity | Mitigation |
|---|---|---|
| Linux aarch64 cross-compilation fails due to linker issues | Medium | Use `gcc-aarch64-linux-gnu` package or `cross-rs/cross` with pre-configured Docker images. The project has no native C dependencies, so cross-compilation should be straightforward. Test the workflow on a non-release tag first. |
| `actions/cache` key collisions cause stale build artifacts | Low | Cache key includes OS and `Cargo.lock` hash. Worst case: a stale cache causes a build failure, which is resolved by re-running the workflow (cache miss triggers a fresh build). |
| GitHub-hosted runner updates break the workflow | Low | Using `-latest` runner tags means occasional breakage when runners update. Acceptable at this project's scale — fix forward when it happens. |
| `CARGO_REGISTRY_TOKEN` secret misconfiguration blocks publishing | Medium | Document the secret setup in the migration plan. Publishing is a separate job that fails independently — binary releases still succeed even if publishing fails. |
| Changelog extraction produces empty release notes if format is wrong | Low | Document the expected `CHANGELOG.md` format. The awk extraction is simple enough to verify manually. If extraction produces empty output, the release is still created — just without notes. |
| Pre-release detection false positive on branch names with `-` | None | The detection runs only on tag-triggered workflows (`v*` tags), not branch names. Tag naming is controlled by the developer. |
| CI minutes consumption on free tier | Low | The project is small (fast builds). Three parallel CI jobs + a six-target release matrix is well within GitHub's free-tier limits for public repositories. |

## Migration Plan

1. **Configure `CARGO_REGISTRY_TOKEN`** — Add the crates.io API token as a
   repository secret in GitHub Settings → Secrets and Variables → Actions.
2. **Create `CHANGELOG.md`** — Add the initial changelog file with an
   `## [Unreleased]` section to the repository root.
3. **Add workflow files** — Create `.github/workflows/build-test-audit.yml` and
   `.github/workflows/release.yml`.
4. **Verify CI** — Push to `dev` or open a PR to trigger the CI workflow.
   Confirm that fmt, clippy, test (all three OSes), and audit jobs pass.
5. **Verify Release** — Create a pre-release tag (e.g., `v0.1.0-rc.1`) to test
   the release workflow end-to-end without publishing to crates.io. Confirm
   that binaries are built and the GitHub Release is created with the
   pre-release flag.
6. **First stable release** — When ready, update `CHANGELOG.md` with the
   release notes under `## [0.1.0]`, tag `v0.1.0`, and push. Confirm that
   binaries are uploaded and the crate is published to crates.io.
