use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::Write;

mod collector;
mod config;
mod model;

#[derive(Parser)]
#[command(
    name = "contribution-showcase",
    version,
    about = "產生貢獻總覽靜態網頁"
)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate the contribution showcase HTML page
    Generate(Generate),
    /// Create a starter config file
    Init(Init),
}

fn parse_positive_usize(s: &str) -> Result<usize, String> {
    let val: usize = s.parse().map_err(|e| format!("{e}"))?;
    if val < 1 {
        return Err("value must be ≥ 1".to_string());
    }
    Ok(val)
}

#[derive(Parser)]
struct Generate {
    /// Config file path
    #[arg(short, long, default_value = "showcase.toml")]
    config: String,

    /// Output HTML file path (overrides config)
    #[arg(short, long)]
    output: Option<String>,

    /// Filter by author name (overrides config)
    #[arg(long)]
    author: Option<String>,

    /// Filter commits since date YYYY-MM-DD (overrides config)
    #[arg(long)]
    since: Option<String>,

    /// Filter commits until date YYYY-MM-DD (overrides config)
    #[arg(long)]
    until: Option<String>,

    /// Maximum timeline buckets before escalating granularity
    #[arg(long, value_parser = parse_positive_usize)]
    timeline_max_buckets: Option<usize>,

    /// Report title (overrides config)
    #[arg(long)]
    title: Option<String>,

    /// Comma-separated commit types to include (overrides config), e.g. feat,fix,docs
    #[arg(long)]
    types: Option<String>,

    /// Comma-separated commit hashes to exclude (overrides config)
    #[arg(long)]
    exclude_hashes: Option<String>,
}

#[derive(Parser)]
struct Init {
    /// Output config file path
    #[arg(short, long, default_value = "showcase.toml")]
    output: String,
}

fn escape_json_for_html_script(json: &str) -> String {
    json.replace('<', r"\u003c")
        .replace('>', r"\u003e")
        .replace('&', r"\u0026")
        .replace('\u{2028}', r"\u2028")
        .replace('\u{2029}', r"\u2029")
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Parse a comma-separated string into a list of trimmed, non-empty, lowercased values.
fn parse_csv_list(input: &str, lowercase: bool) -> Vec<String> {
    input
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            if lowercase {
                trimmed.to_ascii_lowercase()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn run_generate(args: Generate) -> Result<()> {
    let mut config = config::Config::load(&args.config)?;

    // Apply CLI overrides
    let filters = config.filters.get_or_insert_with(Default::default);
    if let Some(ref author) = args.author {
        filters.author = Some(author.clone());
    }
    if let Some(ref since) = args.since {
        filters.since = Some(since.clone());
    }
    if let Some(ref until) = args.until {
        filters.until = Some(until.clone());
    }
    if let Some(ref types_csv) = args.types {
        let parsed = parse_csv_list(types_csv, true);
        if !parsed.is_empty() {
            filters.types = Some(parsed);
        }
    }
    if let Some(ref hashes_csv) = args.exclude_hashes {
        let parsed = parse_csv_list(hashes_csv, true);
        if !parsed.is_empty() {
            filters.exclude_hashes = Some(parsed);
        }
    }

    if let Some(max_buckets) = args.timeline_max_buckets {
        let output = config.output.get_or_insert_with(Default::default);
        output.timeline_max_buckets = Some(max_buckets);
    }

    if let Some(ref title) = args.title {
        config.title = Some(title.clone());
    }

    // Validate date range after CLI overrides are merged
    let final_filters = config.filters.clone().unwrap_or_default();
    config::validate_date_range(
        final_filters.since.as_deref(),
        final_filters.until.as_deref(),
    )?;

    let output_path = args.output.clone().unwrap_or_else(|| config.output_path());

    eprintln!("Scanning repositories...");
    let data = collector::collect(&config)?;

    eprintln!(
        "Summary: {} commits across {} repos, {} OpenSpec proposals",
        data.summary.total_commits, data.summary.total_repos, data.summary.total_proposals
    );

    // Generate HTML
    let template = include_str!("../templates/page.html");
    let html = template.replace("__PAGE_TITLE__", &escape_html(&data.title));
    let json_data = serde_json::to_string(&data)?;
    let json_data = escape_json_for_html_script(&json_data);
    let html = html.replace("\"__SHOWCASE_DATA__\"", &json_data);

    // Write output
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_path, &html)?;

    eprintln!("Generated: {}", output_path);
    Ok(())
}

fn run_init(args: Init) -> Result<()> {
    const TEMPLATE: &str = include_str!("../showcase.example.toml");

    let path = &args.output;

    if let Some(parent) = std::path::Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => {
            file.write_all(TEMPLATE.as_bytes())?;
            if path == "showcase.toml" {
                eprintln!("Created {path} — edit it, then run 'contribution-showcase generate'");
            } else {
                eprintln!(
                    "Created {path} — edit it, then run 'contribution-showcase generate --config {path}'"
                );
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            anyhow::bail!("{path} already exists")
        }
        Err(e) => Err(e.into()),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate(args) => run_generate(args),
        Command::Init(args) => run_init(args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode a value as a TOML literal string, escaping single quotes as `''`.
    /// This keeps Windows-style paths (with backslashes) parseable in test fixtures.
    fn toml_literal(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }

    #[test]
    fn test_escape_script_close_tag() {
        let input = r#"{"html":"</script>"}"#;
        let escaped = escape_json_for_html_script(input);
        assert!(!escaped.contains("</script>"));
        assert!(escaped.contains(r"\u003c/script\u003e"));
    }

    #[test]
    fn test_escape_angle_brackets_and_ampersand() {
        let input = r#"{"a":"<b>&c</b>"}"#;
        let escaped = escape_json_for_html_script(input);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(!escaped.contains('&'));
    }

    #[test]
    fn test_escape_line_separator_chars() {
        let input = "line\u{2028}sep\u{2029}end";
        let escaped = escape_json_for_html_script(input);
        assert!(!escaped.contains('\u{2028}'));
        assert!(!escaped.contains('\u{2029}'));
        assert!(escaped.contains(r"\u2028"));
        assert!(escaped.contains(r"\u2029"));
    }

    #[test]
    fn test_escaped_json_is_valid_js() {
        let input = r#"{"key":"value<>&"}"#;
        let escaped = escape_json_for_html_script(input);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(!escaped.contains('&'));
    }

    // --- escape_html tests ---

    #[test]
    fn test_escape_html_plain_text() {
        assert_eq!(escape_html("hello world"), "hello world");
    }

    #[test]
    fn test_escape_html_empty_string() {
        assert_eq!(escape_html(""), "");
    }

    #[test]
    fn test_escape_html_ampersand() {
        assert_eq!(escape_html("Tom & Jerry"), "Tom &amp; Jerry");
    }

    #[test]
    fn test_escape_html_angle_brackets() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_escape_html_double_quotes() {
        assert_eq!(escape_html(r#"say "hi""#), "say &quot;hi&quot;");
    }

    #[test]
    fn test_escape_html_combined_characters() {
        assert_eq!(
            escape_html("A & B <C> \"D\""),
            "A &amp; B &lt;C&gt; &quot;D&quot;"
        );
    }

    #[test]
    fn test_escape_html_title_breakout_payload() {
        let payload = r#"</title><script>alert(1)</script>"#;
        let escaped = escape_html(payload);
        assert_eq!(
            escaped,
            "&lt;/title&gt;&lt;script&gt;alert(1)&lt;/script&gt;"
        );
        assert!(!escaped.contains("</title>"));
    }

    // --- 4.2: Init subcommand tests ---

    #[test]
    fn test_init_writes_template_to_default_path() {
        let dir = tempfile::tempdir().unwrap();
        let default_path = dir.path().join("showcase.toml");
        let args = Init {
            output: default_path.to_str().unwrap().to_string(),
        };
        run_init(args).unwrap();
        let content = std::fs::read_to_string(&default_path).unwrap();
        let template = include_str!("../showcase.example.toml");
        assert_eq!(content, template);
    }

    #[test]
    fn test_init_writes_template_to_custom_path() {
        let dir = tempfile::tempdir().unwrap();
        let custom_path = dir.path().join("subdir").join("my-config.toml");
        let args = Init {
            output: custom_path.to_str().unwrap().to_string(),
        };
        run_init(args).unwrap();
        assert!(custom_path.exists());
        let content = std::fs::read_to_string(&custom_path).unwrap();
        let template = include_str!("../showcase.example.toml");
        assert_eq!(content, template);
    }

    #[test]
    fn test_init_refuses_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("showcase.toml");
        std::fs::write(&path, "existing content").unwrap();
        let args = Init {
            output: path.to_str().unwrap().to_string(),
        };
        let result = run_init(args);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("already exists"),
            "Error message should mention 'already exists'"
        );
        // Original file must be preserved
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "existing content");
    }

    // --- 4.3: Bare invocation prints help ---

    #[test]
    fn test_bare_invocation_returns_help_error() {
        let result = Cli::try_parse_from(["contribution-showcase"]);
        match result {
            Err(e) => assert_eq!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ),
            Ok(_) => panic!("Expected error for bare invocation, got Ok"),
        }
    }

    #[test]
    fn test_generate_subcommand_parses_defaults() {
        let cli = Cli::try_parse_from(["contribution-showcase", "generate"]).unwrap();
        match cli.command {
            Command::Generate(g) => {
                assert_eq!(g.config, "showcase.toml");
                assert!(g.output.is_none());
                assert!(g.author.is_none());
            }
            _ => panic!("Expected Generate subcommand"),
        }
    }

    #[test]
    fn test_init_subcommand_parses_custom_output() {
        let cli = Cli::try_parse_from(["contribution-showcase", "init", "--output", "custom.toml"])
            .unwrap();
        match cli.command {
            Command::Init(i) => assert_eq!(i.output, "custom.toml"),
            _ => panic!("Expected Init subcommand"),
        }
    }

    #[test]
    fn test_old_flat_invocation_rejected() {
        let result = Cli::try_parse_from(["contribution-showcase", "--config", "showcase.toml"]);
        assert!(result.is_err(), "Old flat invocation should be rejected");
    }

    // --- 4.4: Template content validation ---

    #[test]
    fn test_template_parses_as_valid_config() {
        let template = include_str!("../showcase.example.toml");
        let result = toml::from_str::<crate::config::Config>(template);
        assert!(
            result.is_ok(),
            "Template failed to parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_template_contains_required_sections() {
        let template = include_str!("../showcase.example.toml");
        assert!(template.contains("title"), "Missing 'title' field");
        assert!(template.contains("[output]"), "Missing '[output]' section");
        assert!(
            template.contains("[filters]"),
            "Missing '[filters]' section"
        );
        assert!(
            template.contains("[[projects]]"),
            "Missing '[[projects]]' section"
        );
    }

    // --- run_generate() integration tests ---

    fn init_temp_git_repo_for_main() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(repo)
                .env("GIT_AUTHOR_NAME", "Test User")
                .env("GIT_AUTHOR_EMAIL", "test@example.com")
                .env("GIT_COMMITTER_NAME", "Test User")
                .env("GIT_COMMITTER_EMAIL", "test@example.com")
                .output()
                .unwrap()
        };
        run(&["init", "--initial-branch", "main"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "Test User"]);
        run(&["config", "commit.gpgsign", "false"]);
        std::fs::write(repo.join("file.txt"), "hello world\n").unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", "feat: initial commit"]);
        dir
    }

    #[test]
    fn test_run_generate_basic() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"test-project\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        assert!(output_path.exists());
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(html.contains("test-project"));
    }

    #[test]
    fn test_run_generate_with_output_override() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = \"default.html\"\n[[projects]]\nname = \"proj\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let override_output = config_dir.path().join("override/out.html");
        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: Some(override_output.to_str().unwrap().to_string()),
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        assert!(override_output.exists());
    }

    #[test]
    fn test_run_generate_with_author_filter() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/out.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"proj\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: Some("Test User".to_string()),
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        assert!(output_path.exists());
    }

    #[test]
    fn test_run_generate_with_date_filters() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/out.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"proj\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: Some("2020-01-01".to_string()),
            until: Some("2030-12-31".to_string()),
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        assert!(output_path.exists());
    }

    #[test]
    fn test_run_init_default_path_returns_ok() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("showcase.toml");
        let args = Init {
            output: path.to_str().unwrap().to_string(),
        };
        assert!(run_init(args).is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_run_init_custom_path_returns_ok() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom/dir/my.toml");
        let args = Init {
            output: path.to_str().unwrap().to_string(),
        };
        assert!(run_init(args).is_ok());
        assert!(path.exists());
    }

    // --- Project URL linking integration tests ---

    #[test]
    fn test_run_generate_with_project_url() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"url-project\"\npath = {}\nbranch = \"main\"\nurl = \"https://github.com/example/repo\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        // URL appears in the embedded JSON data
        assert!(
            html.contains("https://github.com/example/repo"),
            "HTML should contain the project URL in the embedded JSON data"
        );
        // Template contains the conditional link rendering logic
        assert!(
            html.contains("target=\"_blank\""),
            "Template should contain target=_blank for link rendering"
        );
        assert!(
            html.contains("noopener noreferrer"),
            "Template should contain noopener noreferrer for link security"
        );
    }

    #[test]
    fn test_run_generate_without_project_url() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"no-url-project\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("no-url-project"),
            "Project name should be present in the embedded JSON"
        );
        // When no URL is set, the JSON should contain "url":null
        assert!(
            html.contains(r#""url":null"#),
            "JSON should contain url:null when no URL is configured"
        );
    }

    #[test]
    fn test_run_generate_with_special_chars_in_url() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"special-proj\"\npath = {}\nbranch = \"main\"\nurl = \"https://example.com/repo?a=1&b=2\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        // The & in the URL is escaped to \u0026 by escape_json_for_html_script
        assert!(
            html.contains(r"https://example.com/repo?a=1\u0026b=2"),
            "URL with & should be HTML-safe escaped in embedded JSON"
        );
    }

    #[test]
    fn test_run_generate_with_empty_url() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"empty-url-project\"\npath = {}\nbranch = \"main\"\nurl = \"\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("empty-url-project"),
            "Project name should be present"
        );
        // Empty string URL should be in JSON as "" — the JS template treats
        // empty strings as no-URL via safeProjectUrl(), rendering plain text
        assert!(
            html.contains(r#""url":"""#),
            "JSON should contain empty string URL"
        );
    }

    #[test]
    fn test_run_generate_with_javascript_url_is_safe() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"xss-project\"\npath = {}\nbranch = \"main\"\nurl = \"javascript:alert(1)\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("safeProjectUrl"),
            "Template should contain safeProjectUrl function for URL scheme validation"
        );
        assert!(
            html.contains("javascript"),
            "The raw URL value is in the JSON data (filtered by JS at render time)"
        );
    }

    #[test]
    fn test_generated_html_contains_commit_trends_heading() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"p1\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("提交趨勢"),
            "HTML should contain renamed heading '提交趨勢'"
        );
        assert!(
            !html.contains("時間軸"),
            "HTML should no longer contain old heading '時間軸'"
        );
    }

    #[test]
    fn test_generated_html_contains_type_lines_in_json() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"p1\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("type_lines"),
            "Embedded JSON should contain 'type_lines' field in timeline entries"
        );
    }

    #[test]
    fn test_generated_html_contains_timeline_tooltip() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/output.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"p1\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("timeline-tooltip"),
            "HTML should contain timeline-tooltip class for hover tooltips"
        );
    }

    // --- CLI --timeline-max-buckets tests ---

    #[test]
    fn test_cli_timeline_max_buckets_parsed() {
        let cli = Cli::try_parse_from(["prog", "generate", "--timeline-max-buckets", "7"]).unwrap();
        match cli.command {
            Command::Generate(g) => assert_eq!(g.timeline_max_buckets, Some(7)),
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn test_cli_timeline_max_buckets_default_none() {
        let cli = Cli::try_parse_from(["prog", "generate"]).unwrap();
        match cli.command {
            Command::Generate(g) => assert_eq!(g.timeline_max_buckets, None),
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn test_cli_timeline_max_buckets_rejects_zero() {
        let result = Cli::try_parse_from(["prog", "generate", "--timeline-max-buckets", "0"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_timeline_max_buckets_override_applied() {
        let dir = tempfile::tempdir().unwrap();
        let repo_dir = dir.path().join("repo");
        std::fs::create_dir_all(&repo_dir).unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_dir)
            .output()
            .unwrap();

        let config_path = dir.path().join("showcase.toml");
        std::fs::write(
            &config_path,
            format!(
                "title = 'test'\n[[projects]]\nname = 'p'\npath = {}\n",
                toml_literal(repo_dir.to_str().unwrap())
            ),
        )
        .unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: Some(5),
            title: None,
            types: None,
            exclude_hashes: None,
        };

        let mut config = config::Config::load(&args.config).unwrap();
        assert!(
            config.output.is_none()
                || config
                    .output
                    .as_ref()
                    .unwrap()
                    .timeline_max_buckets
                    .is_none()
        );

        if let Some(max_buckets) = args.timeline_max_buckets {
            let output = config.output.get_or_insert_with(Default::default);
            output.timeline_max_buckets = Some(max_buckets);
        }

        assert_eq!(config.timeline_max_buckets(), 5);
    }

    #[test]
    fn test_generated_html_title_matches_config() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("out.html");
        let config_content = format!(
            "title = \"我的超讚專案！貢獻總覽\"\n[output]\npath = {}\n[[projects]]\nname = \"p\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("<title>我的超讚專案！貢獻總覽</title>"),
            "Page title should match configured title"
        );
        assert!(
            !html.contains("__PAGE_TITLE__"),
            "Placeholder should be replaced"
        );
    }

    #[test]
    fn test_generated_html_title_escapes_special_chars() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("out.html");
        let config_content = format!(
            "title = \"A & B <C>\"\n[output]\npath = {}\n[[projects]]\nname = \"p\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("<title>A &amp; B &lt;C&gt;</title>"),
            "HTML-sensitive characters should be escaped in <title>"
        );
    }

    #[test]
    fn test_generated_html_title_defaults_when_omitted() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("out.html");
        let config_content = format!(
            "[output]\npath = {}\n[[projects]]\nname = \"p\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("<title>貢獻總覽</title>"),
            "Default title should be used when title is omitted from config"
        );
    }

    // --- CLI filter flags tests ---

    #[test]
    fn test_cli_title_overrides_config_and_appears_in_html() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/out.html");
        let config_content = format!(
            "title = \"Config Title\"\n[output]\npath = {}\n[[projects]]\nname = \"proj\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: Some("CLI Title".to_string()),
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("<title>CLI Title</title>"),
            "CLI --title should override config title in HTML"
        );
    }

    #[test]
    fn test_cli_title_affects_default_output_filename() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let config_content = format!(
            "title = \"Config Name\"\n[[projects]]\nname = \"proj\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        // Config title -> "config-name.html", CLI title -> "cli-report.html"
        // The CLI title should win, producing "cli-report.html" in the default output dir.
        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: Some("CLI Report".to_string()),
            types: None,
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        // output_path() returns relative "dist/cli-report.html" when no [output] section
        let expected = std::path::Path::new("dist/cli-report.html");
        assert!(
            expected.exists(),
            "Default output filename should use CLI title"
        );
        // Clean up the generated file
        let _ = std::fs::remove_file(expected);
    }

    #[test]
    fn test_cli_types_filters_commits() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/out.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"proj\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: Some("docs".to_string()),
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            !html.contains("initial commit"),
            "Types filter should exclude non-matching commit types"
        );
    }

    #[test]
    fn test_cli_exclude_hashes_excludes_commits() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/out.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[[projects]]\nname = \"proj\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let hash_output = std::process::Command::new("git")
            .args(["log", "--format=%H", "-1"])
            .current_dir(repo_dir.path())
            .output()
            .unwrap();
        let hash = String::from_utf8(hash_output.stdout)
            .unwrap()
            .trim()
            .to_string();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: None,
            exclude_hashes: Some(hash),
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            !html.contains("initial commit"),
            "Excluded hash should remove the commit from output"
        );
    }

    #[test]
    fn test_cli_comma_parsing_handles_whitespace_and_empty() {
        let repo_dir = init_temp_git_repo_for_main();
        let config_dir = tempfile::tempdir().unwrap();
        let output_path = config_dir.path().join("dist/out.html");
        let config_content = format!(
            "title = \"Test\"\n[output]\npath = {}\n[filters]\ntypes = [\"feat\"]\n[[projects]]\nname = \"proj\"\npath = {}\nbranch = \"main\"\n",
            toml_literal(&output_path.to_string_lossy()),
            toml_literal(&repo_dir.path().to_string_lossy())
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
            timeline_max_buckets: None,
            title: None,
            types: Some(" , , ".to_string()),
            exclude_hashes: None,
        };
        run_generate(args).unwrap();
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            html.contains("initial commit"),
            "Empty comma-separated list should not override config types"
        );
    }

    #[test]
    fn test_parse_csv_list_lowercases_and_trims() {
        assert_eq!(parse_csv_list(" Feat , FIX ", true), vec!["feat", "fix"]);
        assert_eq!(parse_csv_list(",,", true), Vec::<String>::new());
        assert_eq!(parse_csv_list("AbC123", true), vec!["abc123"]);
        assert_eq!(parse_csv_list(" Feat , FIX ", false), vec!["Feat", "FIX"]);
    }
}
