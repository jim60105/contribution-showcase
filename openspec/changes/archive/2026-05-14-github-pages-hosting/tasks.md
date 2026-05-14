## 1. Static Hosting Setup

- [x] 1.1 Create `docs/` directory in the repository root
- [x] 1.2 Copy the pre-built HTML report to `docs/contribution-showcase.html`
- [x] 1.3 Verify `docs/contribution-showcase.html` exists and is valid HTML (~174KB)

## 2. GitHub Actions Workflow

- [x] 2.1 Create `.github/workflows/pages.yml` with trigger on push to `master` filtered to `docs/**` path changes
- [x] 2.2 Add `actions/configure-pages`, `actions/upload-pages-artifact`, and `actions/deploy-pages` steps targeting the `docs/` directory
- [x] 2.3 Configure concurrency group `pages` with `cancel-in-progress: false` to serialise deployments and prevent partial overwrites
- [x] 2.4 Set appropriate permissions (`contents: read`, `pages: write`, `id-token: write`) and deployment environment (`github-pages`, `url: ${{ steps.deployment.outputs.page_url }}`)
- [x] 2.5 Validate workflow syntax with `actionlint` or manual YAML review

## 3. Documentation

- [x] 3.1 Add a live demo link in Traditional Chinese (zh-TW) to `README.md` pointing to `https://jim60105.github.io/contribution-showcase/contribution-showcase.html`

## 4. Verification

- [x] 4.1 Confirm `docs/contribution-showcase.html` is tracked by Git (not gitignored)
- [x] 4.2 Confirm `.github/workflows/pages.yml` is valid YAML and uses correct action versions
- [x] 4.3 Document the one-time manual step: enable GitHub Pages source as "GitHub Actions" in repository Settings → Pages
