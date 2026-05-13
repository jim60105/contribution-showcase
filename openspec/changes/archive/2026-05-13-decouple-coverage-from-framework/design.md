## Context

`collect_test_metrics()` in `src/collector.rs` conflates three independent concerns behind a single `detect_framework()` gate:

1. **Coverage command execution** (Phase 1) — spawning a user-specified shell command.
2. **Coverage result parsing** (Phase 3) — reading Cobertura XML or Istanbul JSON from disk.
3. **Test file/case counting** (Phase 3) — scanning source trees for test patterns.

Concerns (1) and (2) depend only on user configuration (`coverage_command`, `coverage_result_path`) and the presence of coverage output files. They have no logical dependency on whether the project uses a recognised test framework. Concern (3) genuinely requires framework knowledge to locate test files and count cases.

Today, if `detect_framework()` returns `"none"`, the function skips the coverage command entirely and short-circuits Phase 3 with `coverage_percent: None`. This means a project that uses an unrecognised framework (or a polyglot project) cannot report coverage even when the user has explicitly configured a working coverage command and result path.

## Goals / Non-Goals

**Goals:**

- Remove the framework gate from coverage command execution (Phase 1) so that any project with a configured `coverage_command` runs it unconditionally.
- Remove the framework gate from coverage result parsing (Phase 3) so that `discover_coverage()` is always called regardless of detected framework.
- Keep test file/case counting gated on framework detection — return zeros when the framework is `"none"`.
- Preserve the existing public API surface: no changes to `ShowcaseConfig`, `ProjectConfig`, or `TestMetrics` structs.
- No regressions for projects that already use a recognised framework.

**Non-Goals:**

- Adding support for new test frameworks (orthogonal; tracked separately).
- Extending coverage file format support beyond Cobertura XML and Istanbul JSON.
- Adding test execution capabilities (the tool only reads coverage results, it does not run tests itself beyond spawning the user-supplied coverage command).
- Adding timeout or retry logic for coverage commands.

## Decisions

### 1. Remove framework gate from Phase 1 entirely

**Decision:** Remove `detect_framework()` and the `if framework == "none" { continue; }` guard from Phase 1.

**Rationale:** The `coverage_command` field is explicit user intent — the user has deliberately configured a command to run. There is no ambiguity about whether coverage should execute. The existing guards (`coverage_command.is_some()` and `project_path.exists()`) are sufficient.

### 2. Always attempt `discover_coverage()` in Phase 3

**Decision:** Call `discover_coverage()` for every project, including those with `framework == "none"`.

**Rationale:** `discover_coverage()` already returns `None` gracefully when no coverage file is found. This matches the existing behaviour for known frameworks and requires no defensive changes. Projects without coverage output simply get `coverage_percent: None`, which is the correct representation.

### 3. Inline fix rather than extracting helper functions

**Decision:** Apply the fix inline within `collect_test_metrics()`.

**Rationale:** The function is already a straightforward two-phase loop. Extracting helpers (e.g., `collect_coverage_for_project`, `collect_test_counts_for_project`) would add indirection without meaningful clarity gains. The change is small — removing two guard clauses and restructuring one conditional block.

## Risks / Trade-offs

- **Coverage commands run more broadly.** After this change, every project with a `coverage_command` will execute it, even if `detect_framework()` returns `"none"`. This is the desired behaviour, but projects that previously silently skipped may now run slow or failing commands. Users should only configure commands that work.
- **No timeout handling.** A misconfigured coverage command could block generation indefinitely. Adding timeout support is out of scope for this change; the existing behaviour (wait for child process) is unchanged.
- **Zero test counts for unknown frameworks.** Projects with `framework == "none"` will report `test_file_count: 0` and `test_case_count: 0` even if they have coverage data. This is acceptable — test counting genuinely requires framework knowledge, and zero is an honest representation of "we don't know how to count these."
