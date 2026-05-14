## Why

The generated HTML report is the primary output of contribution-showcase, yet there is no way to share it publicly. The `dist/` directory is gitignored, so the report vanishes after each build and cannot be served directly from the repository. Hosting the report on GitHub Pages gives the project a live demo, lets potential users see what the tool produces before installing it, and provides a persistent URL for the README and external references.

## What Changes

1. **Static hosting directory** — introduce a `docs/` directory (tracked in Git) containing the pre-built `contribution-showcase.html`. GitHub Pages can serve directly from `docs/` on the default branch, making deployment zero-config after the initial repository setting is toggled.

2. **GitHub Actions workflow for Pages deployment** — add a workflow (`.github/workflows/pages.yml`) that deploys the `docs/` directory to GitHub Pages using the official `actions/deploy-pages` action. This is the modern, recommended approach and decouples the hosting configuration from branch-level settings.

3. **README update** — add a "Live Demo" link near the top of `README.md` pointing to `https://jim60105.github.io/contribution-showcase/contribution-showcase.html` so visitors can immediately see the report. The README is written in Traditional Chinese (zh-TW); the addition will follow the same language and style.

## Capabilities

### New Capabilities

- `github-pages-hosting`: Deploy the generated HTML report to GitHub Pages via a dedicated `docs/` directory and GitHub Actions workflow, providing a publicly accessible live demo.

### Modified Capabilities

_(none — this change adds deployment infrastructure and documentation; no existing runtime behaviour or spec requirements are altered)_

## Impact

- **Repository structure** — new `docs/` directory with a single HTML file; new `.github/workflows/pages.yml`.
- **CI/CD** — one additional GitHub Actions workflow triggered on pushes that touch `docs/`. No effect on existing `ci-test-pipeline` or `ci-release-pipeline`.
- **Dependencies** — none; only standard GitHub-provided Actions (`actions/checkout`, `actions/upload-pages-artifact`, `actions/deploy-pages`).
- **Backward compatibility** — no concern; no users depend on the absence of a `docs/` directory or a Pages deployment.
