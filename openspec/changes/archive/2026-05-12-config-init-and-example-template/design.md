# Design — config-init-and-example-template

## Context

The repository currently tracks `showcase.toml` directly, which contains
project-specific paths and author information. This couples the repo to a
particular user's environment and makes onboarding friction-heavy — new users
must reverse-engineer which fields exist and what values are valid.

The CLI is a flat command with optional flags. Adding an `init` subcommand
requires restructuring to a subcommand-based interface.

## Goals / Non-Goals

**Goals:**

- Stop tracking user-specific config by gitignoring `showcase.toml`.
- Provide `showcase.example.toml` as a fully commented, self-documenting
  template with placeholder data.
- Add `contribution-showcase init` subcommand that writes the template config
  to the working directory.
- Restructure the CLI from flat flags to subcommands (`generate`, `init`).

**Non-Goals:**

- Interactive wizard or prompts during `init`.
- Config schema validation during `init` (just write the template file).
- Auto-detection of git repositories in the working directory.
- Migration tooling (no existing users to migrate).

## Decisions

### 1. Subcommand structure

Use clap's `#[derive(Subcommand)]` with a two-variant enum:

- **`Generate`** — inherits all current flags (`--config`, `--output`,
  `--author`, `--since`, `--until`). Existing behavior is preserved
  unchanged.
- **`Init`** — takes an optional `--output` flag (default `showcase.toml`).
  Writes the example template to the specified path.

Bare invocation (no subcommand) prints help and exits. The top-level `Cli`
struct holds no flags of its own; everything lives on the subcommand variants.

### 2. Template embedding via `include_str!`

The example template is embedded at compile time with
`include_str!("../showcase.example.toml")`, following the same pattern the
project already uses for `templates/page.html`. The `init` command writes
this embedded string to disk.

This keeps the tool fully self-contained as a single binary — no need to
locate or ship a separate template file at runtime.

### 3. Single source of truth for template content

`showcase.example.toml` lives at the repository root (next to `Cargo.toml`).
The binary embeds it via `include_str!`. The checked-in file **is** the
template — there is no second copy. This eliminates drift between what the
repo documents and what the binary produces.

### 4. Example file naming

Follow the `.env` / `.env.example` convention:

- `showcase.toml` — user's real config (gitignored).
- `showcase.example.toml` — committed template with placeholders.

### 5. Overwrite guard

`init` uses `OpenOptions::new().write(true).create_new(true)` to atomically
create the target file — this avoids the check-then-write race condition. If
the file already exists, the call fails and the command prints an error to
stderr and exits with non-zero status. There is no `--force` flag — the user
can delete the file themselves. This follows KISS and avoids accidental data
loss.

### 6. Template content

The example file contains:

- Placeholder projects (e.g., `my-backend`, `my-frontend`) with realistic
  but clearly fake values.
- A placeholder author name.
- Commented-out optional fields (date filters, output path) with inline
  explanations of each field's purpose and accepted formats.

The goal is that a new user can `init`, open the file, and understand every
field without consulting external documentation.

### 7. Gitignore update

Append `/showcase.toml` to `.gitignore`. The existing entries (`/target`,
`/dist`) are unchanged.

## Risks / Trade-offs

| Risk | Severity | Mitigation |
|---|---|---|
| User runs `init` in the wrong directory | Low | Harmless — just creates a `.toml` file that can be deleted. No side effects. |
| Template content drifts from actual config schema | Medium | Eliminated by design: `include_str!` compiles from the same `showcase.example.toml` file. Any schema change that isn't reflected in the example causes a visible gap in the committed file, catchable in review. |
| Bare invocation now prints help instead of running generation | Low | Acceptable breaking change — the tool has zero external users. Subcommand structure is more discoverable long-term. |
| `include_str!` increases binary size | Negligible | A TOML template is a few KB at most. Already done for the HTML template. |
