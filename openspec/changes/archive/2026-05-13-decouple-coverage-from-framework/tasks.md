## 1. Fix Phase 1 — Remove Framework Gate from Coverage Command Execution

- [ ] 1.1 In `collect_test_metrics()`, remove the `detect_framework()` call and `if framework == "none" { continue; }` guard from Phase 1 (the coverage command spawning loop); keep only the `coverage_command.is_some()` and `project_path.exists()` checks
- [ ] 1.2 Add a Phase 1–specific test: configure a project with `coverage_command` that creates an output file (e.g., `"printf '<coverage line-rate=\"0.75\"></coverage>' > coverage.xml"`), do NOT pre-create the file, and assert `coverage_percent` is populated only after the command runs — this test will fail if the Phase 1 gate is not removed

## 2. Fix Phase 3 — Remove Framework Gate from Coverage Result Parsing

- [ ] 2.1 In `collect_test_metrics()` Phase 3, remove the early-return branch that pushes `TestMetrics { coverage_percent: None, ... }` and `continue`s when `framework == "none"`
- [ ] 2.2 Restructure Phase 3 so that `discover_coverage()` is always called; test file/case counting is conditioned on `framework != "none"` (returning `vec![]` and `0` for unknown frameworks)

## 3. Update Tests

- [ ] 3.1 Note: there are no existing integration-level tests for `collect_test_metrics()` that assert the old `framework == "none"` short-circuit — this task is informational; proceed directly to adding new tests (3.2–3.4)
- [ ] 3.2 Add a test (Phase 3 fix): project directory with `framework == "none"`, a pre-existing valid Istanbul JSON at `coverage_result_path` → `coverage_percent` is populated, `test_file_count` is 0, `test_case_count` is 0
- [ ] 3.3 Add a test: project with `framework == "none"` and an explicit `coverage_result_path` pointing to valid Cobertura XML → `coverage_percent` is populated
- [ ] 3.4 Add a test: project with `framework == "none"` and no coverage file → `coverage_percent` is `None`, `test_file_count` is 0, `test_case_count` is 0
- [ ] 3.5 Add a test: project with `framework == "none"` and `coverage_command` that exits non-zero, no output file produced → `coverage_percent` is `None`

## 4. Verify and Validate

- [ ] 4.1 Run `cargo test` — all tests pass
- [ ] 4.2 Run `cargo fmt -- --check` and `cargo clippy` — no warnings or errors
- [ ] 4.3 Do a manual generation run with `HeartReverie_Workspace/showcase.toml` and confirm HeartReverie coverage % appears in the output
