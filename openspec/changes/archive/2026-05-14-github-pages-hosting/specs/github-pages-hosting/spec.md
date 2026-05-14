## ADDED Requirements

### Requirement: Static HTML Hosting Directory

The project SHALL include a `docs/` directory tracked in Git containing the pre-built HTML report file (`contribution-showcase.html`). This directory MUST NOT be gitignored and SHALL be committed to the default branch so that GitHub Pages can serve its contents directly.

#### Scenario: docs directory exists in repository

- **WHEN** the repository is cloned
- **THEN** a `docs/` directory SHALL exist at the repository root containing the file `contribution-showcase.html`

#### Scenario: docs directory is not gitignored

- **WHEN** a contributor runs `git ls-files docs/` on the default branch
- **THEN** the output SHALL list `docs/contribution-showcase.html` as a tracked file

### Requirement: GitHub Actions Pages Workflow

The repository SHALL include a GitHub Actions workflow at `.github/workflows/pages.yml` that deploys the `docs/` directory to GitHub Pages using the official `actions/deploy-pages` action. The workflow SHALL trigger on pushes to the default branch that modify files under `docs/`.

#### Scenario: Workflow file exists at expected path

- **WHEN** a contributor inspects `.github/workflows/pages.yml`
- **THEN** the file SHALL exist and contain a valid GitHub Actions workflow definition

#### Scenario: Workflow triggers on docs changes pushed to default branch

- **WHEN** a push to the default branch includes changes to files under the `docs/` directory
- **THEN** the Pages deployment workflow SHALL be triggered

#### Scenario: Workflow does not trigger on unrelated changes

- **WHEN** a push to the default branch modifies only files outside the `docs/` directory
- **THEN** the Pages deployment workflow SHALL NOT be triggered

#### Scenario: Workflow deploys docs directory using deploy-pages action

- **WHEN** the Pages deployment workflow runs
- **THEN** it SHALL upload the `docs/` directory as a GitHub Pages artifact and deploy it using the `actions/deploy-pages` action

### Requirement: README Demo Link

The `README.md` SHALL include a link to the live demo at the GitHub Pages URL (`https://jim60105.github.io/contribution-showcase/contribution-showcase.html`). The link text SHALL be in Traditional Chinese (zh-TW) consistent with the project's documentation language conventions.

#### Scenario: Live demo link is present in README

- **WHEN** a user reads the `README.md`
- **THEN** it SHALL contain a hyperlink pointing to `https://jim60105.github.io/contribution-showcase/contribution-showcase.html`

#### Scenario: Link text is in Traditional Chinese

- **WHEN** a user reads the live demo link in `README.md`
- **THEN** the link text SHALL be written in Traditional Chinese (zh-TW)

### Requirement: Workflow Concurrency Control

The Pages deployment workflow SHALL use a concurrency group to prevent parallel deployments. When a new deployment is triggered while a previous one is still in progress, the workflow SHALL queue the new deployment and wait for the in-progress deployment to complete (i.e., `cancel-in-progress: false`) to prevent partial overwrites.

#### Scenario: Concurrency group is configured

- **WHEN** the Pages deployment workflow definition is inspected
- **THEN** it SHALL include a `concurrency` configuration with a group name specific to Pages deployment

#### Scenario: In-progress deployment is cancelled on new push

- **WHEN** a new push to the default branch modifies files under `docs/` while a previous Pages deployment is still running
- **THEN** the new deployment SHALL be queued and wait for the in-progress deployment to finish before proceeding
