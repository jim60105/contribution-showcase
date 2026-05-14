## 1. Config Layer

- [x] 1.1 Add `url: Option<String>` field to `ProjectConfig` struct in `src/config.rs`
- [x] 1.2 Add unit test: parse a TOML config with `url` set and verify `ProjectConfig.url` is `Some(...)`
- [x] 1.3 Add unit test: parse a TOML config without `url` and verify `ProjectConfig.url` is `None`

## 2. Data Model

- [x] 2.1 Add `url: Option<String>` field to `ProjectData` struct in `src/model.rs`
- [x] 2.2 Verify `ProjectData` still derives `Serialize` so the new field appears in JSON output

## 3. Data Flow

- [x] 3.1 Update collector logic in `src/collector.rs` to propagate `ProjectConfig.url` into `ProjectData.url`
- [x] 3.2 Add unit test: when `ProjectConfig.url` is `Some(...)`, the resulting `ProjectData.url` carries the same value
- [x] 3.3 Add unit test: when `ProjectConfig.url` is `None`, the resulting `ProjectData.url` is `None`

## 4. HTML Template

- [x] 4.1 Update `templates/page.html` to conditionally wrap the project name `<h3>` content in an `<a href="..." target="_blank" rel="noopener noreferrer">` tag when `url` is present
- [x] 4.2 Verify that when `url` is absent or empty, the project name renders as plain escaped text with no `<a>` tag

## 5. Example Config

- [x] 5.1 Add a commented `# url = "https://github.com/user/repo"` line to `showcase.example.toml` inside the `[[projects]]` block

## 6. Integration Tests

- [x] 6.1 Add end-to-end test: generate HTML output from a config with `url` set and assert the output contains an `<a>` tag with the correct `href`, `target="_blank"`, and `rel="noopener noreferrer"`
- [x] 6.2 Add end-to-end test: generate HTML output from a config without `url` and assert the project name is rendered without an `<a>` tag
- [x] 6.3 Add test: URL containing special characters (`&`, `"`, `'`) is properly attribute-escaped in the rendered `href`
- [x] 6.4 Add test: empty string `url = ""` is treated as absent and project name renders as plain text
