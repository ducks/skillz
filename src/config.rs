use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    config_path: PathBuf,

    #[serde(rename = "skills-dir")]
    pub skills_dir: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path()?;

        if config_path.exists() {
            let contents =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;
            let mut config: Config =
                toml::from_str(&contents).context("Failed to parse config file")?;
            config.config_path = config_path;
            Ok(config)
        } else {
            Ok(Config {
                config_path,
                skills_dir: None,
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&self.config_path, contents).context("Failed to write config file")?;

        Ok(())
    }

    pub fn skills_dir(&self) -> PathBuf {
        if let Some(dir) = &self.skills_dir {
            PathBuf::from(shellexpand::tilde(dir).to_string())
        } else {
            Self::default_skills_dir()
        }
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "skills-dir" => {
                self.skills_dir = Some(value.to_string());
                Ok(())
            }
            _ => anyhow::bail!("Unknown config key: {}", key),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "skills-dir" => Some(self.skills_dir().display().to_string()),
            _ => None,
        }
    }

    fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to determine config directory")?;
        Ok(config_dir.join("skillz").join("config.toml"))
    }

    fn default_skills_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("skills")
    }
}
