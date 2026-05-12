## ADDED Requirements

### Requirement: Wiki Project Type Override

After parsing the conventional-commit type from a commit subject, the system
SHALL override `commit_type` to `"docs"` when the project name ends with
`.wiki`. The original `scope` field SHALL be preserved.

#### Scenario: Commit in a .wiki project with a non-docs prefix

- **GIVEN** a project named `"my-project.wiki"` and a commit subject `"feat(nav): add sidebar"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"docs"` and `scope` is `"nav"`

#### Scenario: Commit in a .wiki project with no conventional prefix

- **GIVEN** a project named `"my-project.wiki"` and a commit subject `"Update architecture page"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"docs"` and `scope` is `""`

#### Scenario: Commit in a .wiki project that is already docs

- **GIVEN** a project named `"my-project.wiki"` and a commit subject `"docs: update readme"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"docs"` (unchanged) and `scope` is `""`

#### Scenario: Commit in a non-wiki project is not overridden

- **GIVEN** a project named `"my-frontend"` and a commit subject `"feat: add map"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"feat"` (not overridden)

#### Scenario: Project name that contains .wiki but does not end with it

- **GIVEN** a project named `"my-project.wiki-tools"` and a commit subject `"feat: add parser"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"feat"` (not overridden)

#### Scenario: Case-sensitive suffix match

- **GIVEN** a project named `"my-project.WIKI"` and a commit subject `"feat: add page"`
- **WHEN** the system parses the commit
- **THEN** `commit_type` is `"feat"` (not overridden — match is case-sensitive)
