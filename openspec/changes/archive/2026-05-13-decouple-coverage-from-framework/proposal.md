# Decouple Coverage from Framework Detection

## Why

`collect_test_metrics()` in `src/collector.rs` incorrectly gates coverage
command execution and coverage result parsing on `detect_framework()`. When
framework detection returns `"none"`, two things silently break:

1. **Phase 1** skips spawning the coverage command (line 768–770), even though
   the user explicitly configured `coverage_command`.
2. **Phase 3** pushes a `TestMetrics` with `coverage_percent: None` and all
   zero counts (line 812–821), never calling `discover_coverage()`.

This affects every project whose test tooling is not in the hard-coded
detection list — Deno, Bun, Go, Zig, or any custom CI script that produces
standard Cobertura XML / Istanbul JSON output. A user who sets
`coverage_command = "deno task test:coverage"` and
`coverage_result_path = "coverage.xml"` gets **no coverage data and no
error message**, because the command is never executed.

The existing spec (`openspec/specs/test-metrics-collection/spec.md`,
requirement "Per-Project Coverage Command Execution") already states:

> When `coverage_command` is set, the system **SHALL** execute it via `sh -c`
> in the project directory before reading coverage results.

There is no conditional on framework detection. The implementation violates
its own spec.

Coverage command execution and coverage result parsing are orthogonal to
framework detection. Framework detection legitimately drives test file/case
counting (pattern matching differs per language), but it has no bearing on
whether a user-configured shell command should run or whether a standard
coverage output format can be parsed.

## What Changes

Decouple the three independent concerns inside `collect_test_metrics()`:

| Concern | Current gate | Proposed gate |
|---|---|---|
| Run `coverage_command` | `coverage_command.is_some()` **AND** `framework != "none"` | `coverage_command.is_some()` only |
| Parse coverage result (`discover_coverage`) | `framework != "none"` | Always attempt (returns `None` if no file found) |
| Count test files/cases | `framework != "none"` | `framework != "none"` (unchanged) |

Concretely in `src/collector.rs`:

- **Phase 1**: Remove the `detect_framework()` call and the
  `if framework == "none" { continue; }` guard. If `coverage_command` is
  `Some` and the project path exists, spawn the command unconditionally.

- **Phase 3**: Remove the early-return branch that short-circuits when
  `framework == "none"`. Instead, always call `discover_coverage()`. When
  `framework == "none"`, set `test_file_count` and `test_case_count` to 0
  but still populate `coverage_percent` from `discover_coverage()`.

No changes to `detect_framework()` or `discover_coverage()` themselves —
they already work correctly in isolation. The bug is purely in the
orchestration logic of `collect_test_metrics()`.

## Capabilities

### New Capabilities

None. This change fixes incorrect gating; it does not introduce new features.

### Modified Capabilities

- **`test-metrics-collection` → "Per-Project Coverage Command Execution"**:
  Clarify that coverage command execution depends solely on the presence of
  `coverage_command` in the project config. Remove any implicit dependency on
  framework detection from this requirement.

- **`test-metrics-collection` → "Framework Detection"**:
  Narrow the stated scope — framework detection drives test file/case counting
  only. It does not gate coverage command execution or coverage result parsing.

- **`test-metrics-collection` → "Coverage Report Discovery"**:
  Clarify that `discover_coverage()` is always attempted for every project,
  regardless of detected framework. Projects with `framework == "none"` can
  still have `coverage_percent` if a valid report file exists.

## Impact

- **`src/collector.rs`** — `collect_test_metrics()` function: remove
  framework gate from Phase 1 and Phase 3. Roughly 5–10 lines changed.
- **Existing tests** — Tests that assert the current short-circuit behavior
  for `framework == "none"` will need updating to expect
  `discover_coverage()` to be called.
- **New tests needed** — Integration scenarios:
  - Project with `coverage_command` set but `framework == "none"` → command
    runs, coverage result is parsed.
  - Project with no `coverage_command` and `framework == "none"` →
    `discover_coverage()` still called, returns `None` if no report exists.
- **No config schema changes** — `coverage_command` and
  `coverage_result_path` already exist.
- **No template/HTML changes** — the HTML template already handles
  `coverage_percent: None` gracefully.
- **Spec delta** — `openspec/specs/test-metrics-collection/spec.md` needs a
  delta spec to reflect the decoupled gating.
