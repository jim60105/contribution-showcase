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
}

#[derive(Debug, Deserialize, Default)]
pub struct FilterConfig {
    pub author: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub types: Option<Vec<String>>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", path, e))?;
        let config: Config = toml::from_str(&content)?;
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
        })
    }
}
