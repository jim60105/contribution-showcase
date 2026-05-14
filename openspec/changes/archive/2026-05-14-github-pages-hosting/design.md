## Context

contribution-showcase generates a self-contained, single-page HTML report summarising Git contributions. The report is written to `dist/`, which is gitignored, so it disappears after every build and cannot be shared publicly. There is currently no hosting infrastructure, no GitHub Pages configuration, and no live demo link in the README.

GitHub Pages supports two deployment models: (a) serving from a branch/directory, and (b) deploying via a GitHub Actions workflow using the official `actions/deploy-pages` action. The repository already uses GitHub Actions for CI (`ci-test-pipeline`, `ci-release-pipeline`), so adding another workflow is consistent with the existing automation approach.

## Goals / Non-Goals

**Goals:**

- Provide a publicly accessible, persistent URL for the generated HTML report so visitors can preview the tool's output without installing it.
- Keep the deployment mechanism zero-maintenance: committing an updated HTML file to the tracked `docs/` directory is the only step needed to publish a new version.
- Ensure the deployment workflow does not interfere with existing CI pipelines.

**Non-Goals:**

- Automated report regeneration — the workflow does **not** run `cargo run` or rebuild the HTML. The checked-in file in `docs/` is the source of truth. Automating the build-and-commit cycle is a separate concern.
- Custom domain or HTTPS certificate management — the default `jim60105.github.io` subdomain is sufficient.
- Hosting additional assets (CSS, JS, images) — the HTML report is fully self-contained by design, so `docs/` only needs one file.
- Modifying any Rust source code or runtime behaviour.

## Decisions

### 1. `docs/` directory over `gh-pages` branch

A dedicated `docs/` directory on the default branch (`master`) was chosen instead of a separate `gh-pages` branch because:

- **Simplicity** — no orphan branch to create, maintain, or accidentally delete. The report lives alongside the source code in a single branch.
- **Visibility** — contributors can see and update the hosted file through normal pull-request workflows without switching branches.
- **Atomic commits** — changes to the report and to the code that produced it can land in the same merge commit when desired.

### 2. GitHub Actions deployment workflow (`actions/deploy-pages`)

Rather than configuring Pages to serve directly from the `docs/` directory (the legacy "Deploy from a branch" setting), the design uses the modern GitHub Actions-based deployment pipeline:

- `.github/workflows/pages.yml` triggers on pushes to `master` that modify paths under `docs/**`.
- The workflow uses three official actions in sequence:
  1. `actions/checkout` — checks out the repository.
  2. `actions/configure-pages` — prepares the Pages environment.
  3. `actions/upload-pages-artifact` — uploads the `docs/` directory as a Pages artifact.
  4. `actions/deploy-pages` — deploys the uploaded artifact to GitHub Pages.
- This approach gives explicit control over the deployment trigger (path filter), provides deployment status checks visible in the Actions tab, and is the GitHub-recommended method going forward.

**Prerequisite:** GitHub Pages must be enabled in the repository settings (**Settings → Pages → Source: GitHub Actions**). This is a one-time manual step.

### 3. Workflow trigger scoped to `docs/**`

The workflow only fires when files under `docs/` change on `master`. This avoids unnecessary deployments on every push and keeps Actions minutes usage minimal. Other CI workflows are unaffected because they trigger on different paths/events.

### 4. Concurrency control

The workflow should set `concurrency: { group: "pages", cancel-in-progress: false }` to serialise deployments and prevent partial overwrites if multiple pushes land in quick succession.

### 5. README update in Traditional Chinese

A "Live Demo" link (`https://jim60105.github.io/contribution-showcase/contribution-showcase.html`) will be added near the top of `README.md`, following the existing zh-TW language and style conventions. The link placement ensures it is one of the first things a visitor sees.

## Risks / Trade-offs

| Risk / Trade-off | Mitigation |
|---|---|
| **Manual HTML update** — the `docs/` file must be committed manually; it can drift from the latest output. | Acceptable for now. Automated regeneration can be added later as a separate change. The file is small (~174 KB) and changes infrequently. |
| **Repository size** — tracking a 174 KB HTML file adds to the repo. | Negligible compared to `Cargo.lock` and other tracked artefacts. The file compresses well in Git packfiles. |
| **Pages source must be set manually** — the GitHub Actions source setting cannot be configured via code; a repository admin must toggle it once. | Document the step clearly in the workflow file comments and in the PR description. |
| **Single-file fragility** — if the file is deleted from `docs/`, the live demo breaks. | The path filter means no deployment runs if `docs/` is empty, so the last successful deployment remains live. A future CI check could verify the file exists. |
| **No custom 404 / index** — visitors who navigate to the root (`/contribution-showcase/`) will see a 404. | The README links directly to the HTML file path. An `index.html` redirect could be added later if needed, but is out of scope for this change. |
