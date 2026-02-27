use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub domains: Vec<String>,
    pub scan_interval: u64,
    pub scan_concurrency: usize,
    pub upstream_dns: String,
    pub listen_addr: String,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            domains: vec![
                "github.com".to_string(),
                "api.github.com".to_string(),
                "raw.githubusercontent.com".to_string(),
                "assets-cdn.github.com".to_string(),
            ],
            scan_interval: 1800,
            scan_concurrency: 50,
            upstream_dns: "223.5.5.5:53".to_string(),
            listen_addr: "127.0.0.1:53".to_string(),
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}