## Context

The generated HTML report displays each project's name as plain text in an `<h3>` heading. Users have no way to navigate from the report back to the source repository. Since projects may be hosted on GitHub, GitLab, Gitea, or self-hosted instances, auto-detecting the URL from Git remotes is fragile (multiple remotes, SSH vs HTTPS, authentication tokens in URLs). A user-specified optional URL sidesteps all of this complexity.

The data currently flows as: `showcase.toml` → `ProjectConfig` (config.rs) → `ProjectData` (model.rs, via collector.rs) → JSON blob → `page.html` template rendering. The new `url` field follows this exact pipeline with no structural changes.

## Goals / Non-Goals

**Goals:**

- Allow users to optionally associate a URL with each project in the TOML config.
- Render the project name as a clickable hyperlink when a URL is present, opening in a new tab.
- Preserve current behavior exactly when no URL is provided — no visual or functional change.
- Keep the implementation simple: no new crates, no network calls, no URL parsing.

**Non-Goals:**

- **Auto-detecting URLs from Git remotes.** This would require handling multiple remotes, SSH-style URLs, authentication tokens, and provider-specific URL formats. The complexity is not justified when the user already knows their repository URL.
- **Linking individual commits or file paths.** Commit-level links require knowledge of the Git hosting provider's URL structure (e.g., `/commit/<sha>` on GitHub vs. `/-/commit/<sha>` on GitLab). This is a separate feature with significantly more complexity.
- **URL validation.** Adding a URL validation crate or regex adds a dependency for minimal benefit. The URL is user-provided configuration that only appears in an `href` attribute — browsers handle malformed URLs gracefully, and the user is the one who will click it.
- **Supporting multiple URLs per project** (e.g., separate links for issues, CI, docs). One URL per project keeps the config surface minimal.

## Decisions

### 1. The `url` field is an opaque `Option<String>` with no validation

The URL is treated as a plain string passed directly into an HTML `href` attribute. No URL parsing crate (e.g., `url`) is added because:

- The user provides their own URL and is the primary consumer of the link — they have every incentive to provide a valid one.
- Browsers render invalid `href` values harmlessly (the link simply does not navigate correctly).
- Validation would impose opinions about allowed schemes (`https` only? `http`? `ssh`?), which conflicts with the provider-agnostic goal.
- Zero new dependencies aligns with the project's minimal-dependency philosophy.

### 2. Only the project name heading is linked

Wrapping the project name `<h3>` content in an anchor tag is the minimal, highest-value touch point. It gives users a clear, discoverable link without:

- Requiring knowledge of provider-specific URL patterns for commits, diffs, or files.
- Cluttering the report with per-commit links that would vary by hosting platform.
- Changing the visual layout — the heading simply gains an underline/color when a URL is present.

### 3. Security: `target="_blank"` paired with `rel="noopener noreferrer"`

Opening links in a new tab (`target="_blank"`) is the expected UX for an external reference from a report. The `rel="noopener noreferrer"` attribute is required to prevent the opened page from accessing `window.opener`, which is a well-known security concern (reverse tabnabbing). This is standard practice and costs nothing.

### 4. Data flows through the existing pipeline unchanged

The `url` field is added at each layer of the existing pipeline:

1. **`config.rs`** — `ProjectConfig` gains `url: Option<String>`. Serde deserializes it from TOML; missing fields naturally become `None`.
2. **`collector.rs`** — The collector passes `url` from `ProjectConfig` into `ProjectData` during project data construction. No transformation needed.
3. **`model.rs`** — `ProjectData` gains `url: Option<String>`. Serde serializes it into the JSON blob as `null` when absent.
4. **`page.html`** — The JavaScript template conditionally wraps the project name: if `p.url` is truthy, emit an `<a>` tag; otherwise, emit plain text as before.

This approach requires no new abstractions, no intermediate types, and no changes to the generation pipeline's structure.

### 5. The example config documents the field with a comment

`showcase.example.toml` gets a commented-out `# url = "https://github.com/user/repo"` line in the `[[projects]]` section. This makes the field discoverable without requiring users to read external documentation, consistent with how other optional fields are documented in the example.

## Risks / Trade-offs

- **No validation means silent misconfiguration.** A typo in the URL (e.g., `htps://...`) will produce a broken link with no warning. This is an acceptable trade-off: the user will notice immediately when clicking the link, and adding validation would require deciding which URL schemes to allow.
- **XSS via `javascript:` URLs.** A malicious or accidental `url = "javascript:alert(1)"` would execute script when clicked. However, the config file is a local, user-authored file — the threat model assumes the user controls their own config. The `escapeHtml` function already applied to other fields does not prevent scheme-based injection in `href`, but this is consistent with the project's trust model (local tool, local config, no untrusted input).
- **Future feature pressure.** Once project URLs exist, users may request per-commit links, badge rendering, or API-driven stats. The opaque-string design intentionally avoids coupling to any provider, keeping the door open for future features without constraining them.
