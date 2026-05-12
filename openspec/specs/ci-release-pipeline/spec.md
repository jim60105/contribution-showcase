# CI Release Pipeline Specification

### Requirement: Release Workflow Trigger

The release workflow SHALL be triggered exclusively by tag push events matching the pattern `v*`. No other events (manual dispatch, branch push, pull request) SHALL trigger this workflow.

#### Scenario: Tag push matching v* triggers the workflow
- **WHEN** a tag matching `v*` (e.g., `v1.0.0`, `v2.3.1-rc.1`) is pushed to the repository
- **THEN** the release workflow SHALL execute

#### Scenario: Non-matching tags do not trigger the workflow
- **WHEN** a tag not matching `v*` (e.g., `release-1.0`, `latest`) is pushed to the repository
- **THEN** the release workflow SHALL NOT execute

#### Scenario: Branch pushes do not trigger the workflow
- **WHEN** a commit is pushed to any branch (including `master` or `dev`)
- **THEN** the release workflow SHALL NOT execute

---

### Requirement: SemVer Tag Validation

The release workflow SHALL validate that the pushed tag conforms to a strict SemVer pattern before proceeding with release creation or binary builds. Tags not matching `^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$` SHALL cause the workflow to fail early with a clear error message.

#### Scenario: Valid stable tag passes validation
- **WHEN** a tag `v1.2.0` is pushed
- **THEN** the SemVer validation step SHALL pass and the workflow SHALL proceed

#### Scenario: Valid pre-release tag passes validation
- **WHEN** a tag `v1.2.0-rc.1` is pushed
- **THEN** the SemVer validation step SHALL pass and the workflow SHALL proceed

#### Scenario: Invalid tag fails validation
- **WHEN** a tag `vtest` or `v-bad` is pushed
- **THEN** the SemVer validation step SHALL fail and the workflow SHALL NOT proceed with release creation or binary builds

---

### Requirement: Tag Version Matches Cargo.toml Version

The release workflow SHALL verify that the version extracted from the tag (after stripping the `v` prefix and any pre-release suffix) matches the `package.version` field in `Cargo.toml`. A mismatch SHALL cause the workflow to fail before any release or publishing occurs.

#### Scenario: Tag version matches Cargo.toml
- **WHEN** a tag `v0.2.0` is pushed AND `Cargo.toml` contains `version = "0.2.0"`
- **THEN** the version check SHALL pass and the workflow SHALL proceed

#### Scenario: Pre-release tag matches Cargo.toml base version
- **WHEN** a tag `v0.2.0-rc.1` is pushed AND `Cargo.toml` contains `version = "0.2.0"`
- **THEN** the version check SHALL pass (the pre-release suffix is stripped before comparison)

#### Scenario: Tag version does not match Cargo.toml
- **WHEN** a tag `v0.3.0` is pushed AND `Cargo.toml` contains `version = "0.2.0"`
- **THEN** the version check SHALL fail with an error indicating the mismatch

---

### Requirement: Workflow File Location

The release workflow MUST be defined in the file `.github/workflows/release.yml` within the repository root.

#### Scenario: Workflow file exists at the expected path
- **WHEN** the CI pipeline is configured
- **THEN** the release workflow file SHALL exist at `.github/workflows/release.yml`

---

### Requirement: GitHub Release Creation with Changelog Extraction

The workflow SHALL create a GitHub Release using `softprops/action-gh-release@v3`. The release body SHALL be populated by extracting the relevant version section from `CHANGELOG.md`. If extraction fails or yields no content, a default fallback message SHALL be used.

#### Scenario: Changelog section exists for the tagged version
- **WHEN** a tag `v1.2.0` is pushed AND `CHANGELOG.md` contains a section header matching the version `1.2.0`
- **THEN** the workflow SHALL extract the content between that version's header and the next version header (or end of file) AND use it as the GitHub Release body

#### Scenario: Changelog section does not exist for the tagged version
- **WHEN** a tag `v1.2.0` is pushed AND `CHANGELOG.md` does not contain a section header matching the version `1.2.0`
- **THEN** the workflow SHALL use a default fallback message as the GitHub Release body

#### Scenario: CHANGELOG.md does not exist
- **WHEN** a tag is pushed AND `CHANGELOG.md` does not exist in the repository root
- **THEN** the workflow SHALL use a default fallback message as the GitHub Release body

#### Scenario: Release is created with the correct tag name
- **WHEN** a tag `v1.2.0` is pushed
- **THEN** the GitHub Release SHALL be created with the tag name `v1.2.0` as its title/tag reference

#### Scenario: Changelog content is passed via file
- **WHEN** the changelog section is extracted
- **THEN** the content SHALL be written to a temporary file and passed to the release action via `body_path` (not `body`) to safely handle multiline content

---

### Requirement: CHANGELOG.md Format for Extraction

The changelog extraction logic SHALL expect `CHANGELOG.md` to use version section headers that contain the version number (without the `v` prefix). The extraction SHALL use `awk` to locate the section matching the tagged version and capture all content until the next version section header.

#### Scenario: Standard Keep a Changelog format
- **WHEN** `CHANGELOG.md` contains a header like `## [1.2.0]` or `## [1.2.0] - 2024-01-15`
- **THEN** the extraction logic SHALL correctly identify and extract that version's section content

#### Scenario: Pre-release version in changelog
- **WHEN** `CHANGELOG.md` contains a header like `## [1.2.0-rc.1]`
- **THEN** the extraction logic SHALL correctly identify and extract that pre-release version's section content

---

### Requirement: Pre-release Detection

The workflow SHALL detect pre-release versions by checking whether the tag name contains a hyphen (`-`) character (per SemVer pre-release convention). Tags containing `-` (e.g., `v1.0.0-rc.1`, `v2.0.0-beta.3`) SHALL be marked as pre-releases in the GitHub Release.

#### Scenario: Stable release tag
- **WHEN** a tag `v1.2.0` is pushed (no hyphen after the version)
- **THEN** the GitHub Release SHALL NOT be marked as a pre-release

#### Scenario: Pre-release tag with rc suffix
- **WHEN** a tag `v1.2.0-rc.1` is pushed
- **THEN** the GitHub Release SHALL be marked as a pre-release

#### Scenario: Pre-release tag with beta suffix
- **WHEN** a tag `v2.0.0-beta.3` is pushed
- **THEN** the GitHub Release SHALL be marked as a pre-release

#### Scenario: Pre-release detection uses contains check on tag name
- **WHEN** determining pre-release status
- **THEN** the workflow SHALL use the expression `contains(github.ref_name, '-')` to detect pre-releases

---

### Requirement: Cross-platform Binary Build Matrix

The workflow SHALL build release binaries for exactly six target platforms using a matrix strategy. Each build MUST produce a statically or dynamically linked binary appropriate for the target platform.

#### Scenario: Linux x86_64 binary build
- **WHEN** the build matrix executes
- **THEN** a binary targeting `x86_64-unknown-linux-gnu` SHALL be built on an `ubuntu-latest` runner

#### Scenario: Linux aarch64 binary build
- **WHEN** the build matrix executes
- **THEN** a binary targeting `aarch64-unknown-linux-gnu` SHALL be built on an `ubuntu-latest` runner via cross-compilation

#### Scenario: Windows x86_64 binary build
- **WHEN** the build matrix executes
- **THEN** a binary targeting `x86_64-pc-windows-msvc` SHALL be built on a `windows-latest` runner

#### Scenario: macOS x86_64 binary build
- **WHEN** the build matrix executes
- **THEN** a binary targeting `x86_64-apple-darwin` SHALL be built on a `macos-latest` runner

#### Scenario: Windows aarch64 binary build
- **WHEN** the build matrix executes
- **THEN** a binary targeting `aarch64-pc-windows-msvc` SHALL be built on a `windows-latest` runner

#### Scenario: macOS aarch64 binary build
- **WHEN** the build matrix executes
- **THEN** a binary targeting `aarch64-apple-darwin` SHALL be built on a `macos-latest` runner

#### Scenario: All six targets are included in the matrix
- **WHEN** the release workflow is triggered
- **THEN** the build matrix SHALL include exactly the following six targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`, `x86_64-apple-darwin`, `aarch64-apple-darwin`

---

### Requirement: aarch64 Linux Cross-compilation Setup

The workflow SHALL install the `crossbuild-essential-arm64` package on the `ubuntu-latest` runner when building for the `aarch64-unknown-linux-gnu` target. The appropriate linker and target configuration MUST be set for successful cross-compilation.

#### Scenario: Cross-compilation toolchain is installed for aarch64 Linux
- **WHEN** the build matrix entry for `aarch64-unknown-linux-gnu` executes
- **THEN** the workflow SHALL install the `crossbuild-essential-arm64` apt package before building

#### Scenario: Rust target is added for aarch64 Linux
- **WHEN** the build matrix entry for `aarch64-unknown-linux-gnu` executes
- **THEN** the workflow SHALL run `rustup target add aarch64-unknown-linux-gnu` to install the target toolchain

#### Scenario: Cross-compilation toolchain is NOT installed for native targets
- **WHEN** the build matrix entry for a native target (e.g., `x86_64-unknown-linux-gnu`) executes
- **THEN** the workflow SHALL NOT install `crossbuild-essential-arm64`

---

### Requirement: Binary Asset Naming Convention and Upload

Built binaries SHALL be renamed to follow a consistent naming convention and uploaded as release assets to the GitHub Release. The naming convention MUST include the binary name and target triple (the version is captured by the GitHub Release itself). Windows binaries MUST retain the `.exe` extension.

#### Scenario: Linux binary asset naming
- **WHEN** a binary is built for `x86_64-unknown-linux-gnu` with tag `v1.2.0`
- **THEN** the binary SHALL be uploaded as `contribution-showcase-x86_64-unknown-linux-gnu` to the GitHub Release

#### Scenario: Windows binary asset naming
- **WHEN** a binary is built for `x86_64-pc-windows-msvc` with tag `v1.2.0`
- **THEN** the binary SHALL be uploaded as `contribution-showcase-x86_64-pc-windows-msvc.exe` to the GitHub Release

#### Scenario: Windows ARM binary asset naming
- **WHEN** a binary is built for `aarch64-pc-windows-msvc` with tag `v1.2.0`
- **THEN** the binary SHALL be uploaded as `contribution-showcase-aarch64-pc-windows-msvc.exe` to the GitHub Release

#### Scenario: macOS binary asset naming
- **WHEN** a binary is built for `aarch64-apple-darwin` with tag `v1.2.0`
- **THEN** the binary SHALL be uploaded as `contribution-showcase-aarch64-apple-darwin` to the GitHub Release

#### Scenario: All binaries are uploaded to the same release
- **WHEN** all six matrix builds complete successfully
- **THEN** the GitHub Release SHALL contain exactly six binary assets, one for each target platform

#### Scenario: Binary upload uses softprops/action-gh-release
- **WHEN** binaries are uploaded as release assets
- **THEN** the workflow SHALL use `softprops/action-gh-release@v3` with the `files` parameter to upload the assets

---

### Requirement: crates.io Publishing for Stable Releases

The workflow SHALL publish the crate to crates.io for stable releases only. Pre-release versions (tags containing `-`) SHALL be skipped. Publishing MUST use `cargo publish` with the `CARGO_REGISTRY_TOKEN` secret for authentication.

#### Scenario: Stable release triggers crates.io publish
- **WHEN** a tag `v1.2.0` is pushed (no hyphen, stable release)
- **THEN** the workflow SHALL execute `cargo publish` to publish the crate to crates.io

#### Scenario: Pre-release skips crates.io publish
- **WHEN** a tag `v1.2.0-rc.1` is pushed (contains hyphen, pre-release)
- **THEN** the workflow SHALL NOT execute `cargo publish`

#### Scenario: Publish condition uses contains check
- **WHEN** determining whether to publish to crates.io
- **THEN** the workflow SHALL use `!contains(github.ref_name, '-')` as the conditional expression

#### Scenario: Publish uses CARGO_REGISTRY_TOKEN for authentication
- **WHEN** `cargo publish` is executed
- **THEN** the `CARGO_REGISTRY_TOKEN` secret SHALL be provided as an environment variable for authentication with crates.io

---

### Requirement: Required Secrets and Permissions

The release workflow MUST declare `permissions: contents: write` to allow creating GitHub Releases and uploading assets. The workflow MUST require the `CARGO_REGISTRY_TOKEN` repository secret for crates.io publishing.

#### Scenario: Contents write permission is declared
- **WHEN** the release workflow is defined
- **THEN** the workflow SHALL declare `permissions: contents: write` at the workflow or job level

#### Scenario: CARGO_REGISTRY_TOKEN secret is available for publishing
- **WHEN** the crates.io publish step executes
- **THEN** the step SHALL reference `secrets.CARGO_REGISTRY_TOKEN` for authentication

#### Scenario: Workflow does not require additional permissions beyond contents write
- **WHEN** the release workflow executes
- **THEN** no permissions beyond `contents: write` SHALL be required for the workflow to function correctly

---

### Requirement: Cargo.toml Metadata for crates.io

The project's `Cargo.toml` SHALL include all metadata fields required by crates.io for successful publication: `description`, `license`, and `repository`. The `cargo publish --dry-run` command MUST succeed before any actual publish attempt.

#### Scenario: Required metadata fields are present
- **WHEN** `Cargo.toml` is inspected
- **THEN** the `description`, `license`, and `repository` fields SHALL be present and non-empty

#### Scenario: Dry-run publish succeeds
- **WHEN** `cargo publish --dry-run` is executed
- **THEN** the command SHALL succeed without errors, confirming the crate is ready for publication
