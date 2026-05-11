# Conventional Commit Parser

## Purpose

Parses commit subjects to extract the conventional-commit type and optional
scope using a hand-written byte-level parser. Provides Traditional Chinese
(zh-TW) labels for each recognized type, enabling localized reporting and
categorization of commit history.

## Requirements

### Requirement: Type Extraction

The system SHALL identify the conventional commit type from the `type(scope):`
or `type:` pattern at the start of a commit subject. The 10 recognized types
are: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`, `build`,
`style`, and `perf`.

#### Scenario: Standard type without scope
- **GIVEN** a commit subject `"feat: add map layer toggle"`
- **WHEN** the parser processes the subject
- **THEN** the extracted type is `"feat"` and scope is empty

#### Scenario: Each recognized type is accepted
- **GIVEN** a commit subject starting with any of the 10 known types followed by `:`
- **WHEN** the parser processes the subject
- **THEN** the extracted type matches the prefix token

### Requirement: Scope Extraction

The system SHALL extract the scope from parentheses between the type and the
colon, following the pattern `type(scope):`.

#### Scenario: Type with scope
- **GIVEN** a commit subject `"fix(auth): correct token refresh logic"`
- **WHEN** the parser processes the subject
- **THEN** the extracted type is `"fix"` and scope is `"auth"`

#### Scenario: Empty parentheses
- **GIVEN** a commit subject `"docs(): update README"`
- **WHEN** the parser processes the subject
- **THEN** the extracted type is `"docs"` and scope is empty

### Requirement: Breaking Change Indicator

The system SHALL recognize the `!` character immediately before `:` as a
breaking change indicator, in patterns `type!:` or `type(scope)!:`.

#### Scenario: Breaking change without scope
- **GIVEN** a commit subject `"feat!: remove deprecated API"`
- **WHEN** the parser processes the subject
- **THEN** the extracted type is `"feat"` and the breaking change flag is set

#### Scenario: Breaking change with scope
- **GIVEN** a commit subject `"refactor(core)!: restructure module layout"`
- **WHEN** the parser processes the subject
- **THEN** the extracted type is `"refactor"`, scope is `"core"`, and the breaking change flag is set

### Requirement: Non-Matching Subject Fallback

The system SHALL assign the type `"other"` to any commit subject that does not
match the `type:` or `type(scope):` pattern.

#### Scenario: Free-form subject
- **GIVEN** a commit subject `"Merge branch 'dev' into main"`
- **WHEN** the parser processes the subject
- **THEN** the extracted type is `"other"` and scope is empty

#### Scenario: Unknown prefix before colon
- **GIVEN** a commit subject `"update: fix something"`
- **WHEN** the parser processes the subject
- **THEN** the extracted type is `"other"` because `"update"` is not a recognized type

### Requirement: Type Label Mapping

The system SHALL map each recognized type to a Traditional Chinese (zh-TW)
label as follows:

| Type       | Label     |
|------------|-----------|
| `feat`     | 新功能    |
| `fix`      | 錯誤修復  |
| `docs`     | 文件      |
| `refactor` | 重構      |
| `test`     | 測試      |
| `chore`    | 雜務      |
| `ci`       | CI        |
| `build`    | 建置      |
| `style`    | 風格      |
| `perf`     | 效能      |

#### Scenario: Known type label lookup
- **GIVEN** a parsed commit with type `"feat"`
- **WHEN** the label is resolved
- **THEN** the label is `"新功能"`

### Requirement: Unknown Type Fallback Label

The system SHALL use the label `"其他"` for commits whose type is `"other"`.

#### Scenario: Fallback label
- **GIVEN** a parsed commit with type `"other"`
- **WHEN** the label is resolved
- **THEN** the label is `"其他"`
