use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub title: Option<String>,
    pub output: Option<OutputConfig>,
    pub projects: Vec<ProjectConfig>,
    pub filters: Option<FilterConfig>,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub branch: Option<String>,
    pub coverage_command: Option<String>,
    pub coverage_result_path: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FilterConfig {
    pub author: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub types: Option<Vec<String>>,
    pub exclude_hashes: Option<Vec<String>>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", path, e))?;
        let mut config: Config = toml::from_str(&content)?;

        // Resolve relative paths against config file's parent directory
        let config_path = std::path::Path::new(path);
        let config_dir = config_path.parent().unwrap_or(std::path::Path::new("."));

        for project in &mut config.projects {
            let p = std::path::Path::new(&project.path);
            if p.is_relative() {
                project.path = config_dir.join(p).to_string_lossy().to_string();
            }

            // Validate branch name — reject argument injection and revspec operators
            if let Some(ref branch) = project.branch {
                if branch.starts_with('-') {
                    anyhow::bail!(
                        "branch name must not start with '-' for project '{}': {}",
                        project.name, branch
                    );
                }
                if branch.contains("..") || branch.contains('^') || branch.contains('~') || branch.contains(' ') {
                    anyhow::bail!(
                        "branch name contains invalid characters for project '{}': '{}' (revspec operators like '..', '^', '~' are not allowed)",
                        project.name, branch
                    );
                }
            }
        }

        // Resolve output path
        if let Some(ref mut output) = config.output {
            if let Some(ref mut out_path) = output.path {
                let p = std::path::Path::new(out_path.as_str());
                if p.is_relative() {
                    *out_path = config_dir.join(p).to_string_lossy().to_string();
                }
            }
        }

        Ok(config)
    }

    pub fn output_path(&self) -> &str {
        self.output
            .as_ref()
            .and_then(|o| o.path.as_deref())
            .unwrap_or("dist/index.html")
    }

    pub fn filters(&self) -> FilterConfig {
        self.filters.as_ref().map_or_else(FilterConfig::default, |f| FilterConfig {
            author: f.author.clone(),
            since: f.since.clone(),
            until: f.until.clone(),
            types: f.types.clone(),
            exclude_hashes: f.exclude_hashes.clone(),
        })
    }
}

pub fn validate_date_range(since: Option<&str>, until: Option<&str>) -> Result<()> {
    use chrono::NaiveDate;

    let strict_date = |d: &str, label: &str| -> Result<()> {
        // Enforce exactly YYYY-MM-DD (10 chars, zero-padded)
        if d.len() != 10 {
            anyhow::bail!("invalid '{}' date '{}': expected YYYY-MM-DD format", label, d);
        }
        NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("invalid '{}' date '{}': expected YYYY-MM-DD format", label, d))?;
        Ok(())
    };

    if let Some(s) = since {
        strict_date(s, "since")?;
    }
    if let Some(u) = until {
        strict_date(u, "until")?;
    }
    if let (Some(s), Some(u)) = (since, until) {
        if s > u {
            anyhow::bail!("'since' ({}) must not be after 'until' ({})", s, u);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir_unique(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("test-config-{}-{}", name, std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_date_validation_valid() {
        assert!(validate_date_range(Some("2024-01-01"), Some("2024-12-31")).is_ok());
    }

    #[test]
    fn test_date_validation_invalid_format() {
        assert!(validate_date_range(Some("2024/01/01"), None).is_err());
    }

    #[test]
    fn test_date_validation_non_zero_padded() {
        assert!(validate_date_range(Some("2026-5-1"), None).is_err());
    }

    #[test]
    fn test_date_validation_since_after_until() {
        assert!(validate_date_range(Some("2024-12-31"), Some("2024-01-01")).is_err());
    }

    #[test]
    fn test_branch_validation_rejects_dash_prefix() {
        let dir = temp_dir_unique("branch");
        let config_file = dir.join("showcase.toml");
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\nbranch = \"--all\"\n",
        )
        .unwrap();
        let result = Config::load(config_file.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_minimal() {
        let dir = temp_dir_unique("minimal");
        let config_file = dir.join("showcase.toml");
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\n",
        )
        .unwrap();
        let config = Config::load(config_file.to_str().unwrap()).unwrap();
        assert_eq!(config.projects.len(), 1);
        assert_eq!(config.projects[0].name, "test");
        assert_eq!(config.projects[0].path, "/tmp/test");
        assert!(config.projects[0].description.is_none());
        assert!(config.projects[0].branch.is_none());
        assert!(config.title.is_none());
    }

    #[test]
    fn test_config_output_path_default() {
        let dir = temp_dir_unique("default-output");
        let config_file = dir.join("showcase.toml");
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\n",
        )
        .unwrap();
        let config = Config::load(config_file.to_str().unwrap()).unwrap();
        assert_eq!(config.output_path(), "dist/index.html");
    }

    #[test]
    fn test_config_output_path_custom() {
        let dir = temp_dir_unique("custom-output");
        let config_file = dir.join("showcase.toml");
        fs::write(
            &config_file,
            "[output]\npath = \"custom/out.html\"\n\n[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\n",
        )
        .unwrap();
        let config = Config::load(config_file.to_str().unwrap()).unwrap();
        let expected = dir.join("custom/out.html").to_string_lossy().to_string();
        assert_eq!(config.output_path(), expected);
    }

    #[test]
    fn test_project_path_resolves_relative_to_config_parent() {
        let dir = temp_dir_unique("relative-path");
        let config_file = dir.join("showcase.toml");
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"./repo\"\n",
        )
        .unwrap();
        let config = Config::load(config_file.to_str().unwrap()).unwrap();
        let expected = dir.join("./repo").to_string_lossy().to_string();
        assert_eq!(config.projects[0].path, expected);
    }

    #[test]
    fn test_absolute_project_path_is_preserved() {
        let dir = temp_dir_unique("absolute-path");
        let config_file = dir.join("showcase.toml");
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/opt/repos/foo\"\n",
        )
        .unwrap();
        let config = Config::load(config_file.to_str().unwrap()).unwrap();
        assert_eq!(config.projects[0].path, "/opt/repos/foo");
    }

    #[test]
    fn test_date_validation_invalid_semantic_date() {
        // Feb 31 does not exist
        assert!(validate_date_range(Some("2024-02-31"), None).is_err());
    }

    #[test]
    fn test_branch_validation_rejects_revspec_operators() {
        let dir = temp_dir_unique("revspec");
        let config_file = dir.join("showcase.toml");

        // Test ".." (range operator)
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\nbranch = \"main..feature\"\n",
        ).unwrap();
        assert!(Config::load(config_file.to_str().unwrap()).is_err());

        // Test "^" (exclusion)
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\nbranch = \"^main\"\n",
        ).unwrap();
        assert!(Config::load(config_file.to_str().unwrap()).is_err());

        // Test "~" (ancestor)
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\nbranch = \"HEAD~10\"\n",
        ).unwrap();
        assert!(Config::load(config_file.to_str().unwrap()).is_err());
    }

    #[test]
    fn test_date_validation_none_passes() {
        // No dates at all should pass (empty filter)
        assert!(validate_date_range(None, None).is_ok());
    }

    #[test]
    fn test_config_exclude_hashes_parsed() {
        let dir = temp_dir_unique("exclude-hashes");
        let config_file = dir.join("showcase.toml");
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\n\n[filters]\nexclude_hashes = [\"abc123\"]\n",
        ).unwrap();
        let config = Config::load(config_file.to_str().unwrap()).unwrap();
        let filters = config.filters();
        assert_eq!(filters.exclude_hashes, Some(vec!["abc123".to_string()]));
    }

    #[test]
    fn test_config_exclude_hashes_absent_is_none() {
        let dir = temp_dir_unique("no-exclude-hashes");
        let config_file = dir.join("showcase.toml");
        fs::write(
            &config_file,
            "[[projects]]\nname = \"test\"\npath = \"/tmp/test\"\n\n[filters]\nauthor = \"Jim\"\n",
        ).unwrap();
        let config = Config::load(config_file.to_str().unwrap()).unwrap();
        let filters = config.filters();
        assert!(filters.exclude_hashes.is_none());
    }
}
