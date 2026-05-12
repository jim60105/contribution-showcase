use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::Write;

mod collector;
mod config;
mod model;

#[derive(Parser)]
#[command(name = "contribution-showcase", about = "產生貢獻總覽靜態網頁")]
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

    // Validate date range after CLI overrides are merged
    let final_filters = config
        .filters
        .as_ref()
        .map(|f| f.clone())
        .unwrap_or_default();
    config::validate_date_range(
        final_filters.since.as_deref(),
        final_filters.until.as_deref(),
    )?;

    let output_path = args
        .output
        .as_deref()
        .unwrap_or_else(|| config.output_path());

    eprintln!("Scanning repositories...");
    let data = collector::collect(&config)?;

    eprintln!(
        "Summary: {} commits across {} repos, {} OpenSpec proposals",
        data.summary.total_commits, data.summary.total_repos, data.summary.total_proposals
    );

    // Generate HTML
    let template = include_str!("../templates/page.html");
    let json_data = serde_json::to_string(&data)?;
    let json_data = escape_json_for_html_script(&json_data);
    let html = template.replace("\"__SHOWCASE_DATA__\"", &json_data);

    // Write output
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, &html)?;

    eprintln!("Generated: {}", output_path);
    Ok(())
}

fn run_init(args: Init) -> Result<()> {
    const TEMPLATE: &str = include_str!("../showcase.example.toml");

    let path = &args.output;

    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
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
            "title = \"Test\"\n[output]\npath = \"{}\"\n[[projects]]\nname = \"test-project\"\npath = \"{}\"\nbranch = \"main\"\n",
            output_path.display(),
            repo_dir.path().display()
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: None,
            until: None,
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
            "title = \"Test\"\n[output]\npath = \"default.html\"\n[[projects]]\nname = \"proj\"\npath = \"{}\"\nbranch = \"main\"\n",
            repo_dir.path().display()
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
            "title = \"Test\"\n[output]\npath = \"{}\"\n[[projects]]\nname = \"proj\"\npath = \"{}\"\nbranch = \"main\"\n",
            output_path.display(),
            repo_dir.path().display()
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: Some("Test User".to_string()),
            since: None,
            until: None,
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
            "title = \"Test\"\n[output]\npath = \"{}\"\n[[projects]]\nname = \"proj\"\npath = \"{}\"\nbranch = \"main\"\n",
            output_path.display(),
            repo_dir.path().display()
        );
        let config_path = config_dir.path().join("showcase.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let args = Generate {
            config: config_path.to_str().unwrap().to_string(),
            output: None,
            author: None,
            since: Some("2020-01-01".to_string()),
            until: Some("2030-12-31".to_string()),
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
}
