use clap::Parser;
use anyhow::Result;

mod config;
mod model;
mod collector;

#[derive(Parser)]
#[command(name = "contribution-showcase", about = "產生貢獻總覽靜態網頁")]
struct Cli {
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

fn escape_json_for_html_script(json: &str) -> String {
    json.replace('<', r"\u003c")
        .replace('>', r"\u003e")
        .replace('&', r"\u0026")
        .replace('\u{2028}', r"\u2028")
        .replace('\u{2029}', r"\u2029")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = config::Config::load(&cli.config)?;

    // Apply CLI overrides
    let filters = config.filters.get_or_insert_with(Default::default);
    if let Some(ref author) = cli.author {
        filters.author = Some(author.clone());
    }
    if let Some(ref since) = cli.since {
        filters.since = Some(since.clone());
    }
    if let Some(ref until) = cli.until {
        filters.until = Some(until.clone());
    }

    // Validate date range after CLI overrides are merged
    let final_filters = config.filters.as_ref().map(|f| f.clone()).unwrap_or_default();
    config::validate_date_range(
        final_filters.since.as_deref(),
        final_filters.until.as_deref(),
    )?;

    let output_path = cli.output.as_deref().unwrap_or_else(|| config.output_path());

    eprintln!("Scanning repositories...");
    let data = collector::collect(&config)?;

    eprintln!("Summary: {} commits across {} repos, {} OpenSpec proposals",
        data.summary.total_commits, data.summary.total_repos, data.summary.total_proposals);

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
}
