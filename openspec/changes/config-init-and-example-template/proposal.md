# config-init-and-example-template

## Why

The repository currently tracks `showcase.toml` with real project-specific
information (paths, author names, date ranges). Anyone who clones the repo
inherits someone else's config and risks committing their own private details
back. There is also no guided way for a new user to bootstrap a valid config
file — they must read source code or copy an existing file and hand-edit it.

Adding a gitignored `showcase.toml`, a checked-in `showcase.example.toml` with
placeholder data and inline documentation, and a CLI `init` subcommand that
writes the template to the working directory solves both problems: real configs
stay local, and new users can get started with a single command.

## What Changes

1. **Gitignore `showcase.toml`** — add the filename to `.gitignore` so
   user-specific configs are never tracked.
2. **Add `showcase.example.toml`** — a fully commented template file with
   placeholder/fake project names, paths, author, and date ranges. Serves as
   both documentation and a copy-and-rename starting point.
3. **`init` subcommand** — `contribution-showcase init` writes the template
   `showcase.toml` (identical content to the example file) into the current
   working directory. If the file already exists, the command refuses to
   overwrite and exits with a non-zero status.
4. **Restructure CLI to subcommands** — the current flat-flag interface moves
   under a `generate` subcommand (`contribution-showcase generate [FLAGS]`).
   Invoking the binary with no subcommand prints help. The existing flags
   (`--config`, `--output`, `--author`, `--since`, `--until`) remain unchanged
   under `generate`.

## Capabilities

### New Capabilities

- **`config-init-command`** — defines the `init` subcommand: template content,
  file-write behaviour, overwrite guard, and exit codes.

### Modified Capabilities

- **`rust-cli-scaffold`** — restructure from a single flat command to a
  subcommand-based CLI (`generate`, `init`). `generate` inherits all current
  flags and default behaviour. Top-level `--help` / `--version` remain.

## Impact

- `src/main.rs` / `src/cli.rs` — clap derive structs change from flat
  `Args` to an enum-based `Subcommand` with `Generate` and `Init` variants.
- `.gitignore` — add `showcase.toml`.
- `showcase.example.toml` — new file (checked in).
- `showcase.toml` — removed from version control (stays as local-only).
- Existing tests covering CLI parsing need updating to use the `generate`
  subcommand form.
- No database, network, or cross-repository impact.
