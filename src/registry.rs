use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillEntry {
    pub source: String,
    pub installed_at: String,
    pub last_synced: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    #[serde(skip)]
    registry_path: PathBuf,

    #[serde(default)]
    pub skills: HashMap<String, SkillEntry>,
}

impl Registry {
    pub fn load() -> Result<Self> {
        let registry_path = Self::default_registry_path()?;

        if registry_path.exists() {
            let contents = std::fs::read_to_string(&registry_path)
                .context("Failed to read registry file")?;
            let mut registry: Registry = toml::from_str(&contents)
                .context("Failed to parse registry file")?;
            registry.registry_path = registry_path;
            Ok(registry)
        } else {
            Ok(Registry {
                registry_path,
                skills: HashMap::new(),
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.registry_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create registry directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize registry")?;
        std::fs::write(&self.registry_path, contents)
            .context("Failed to write registry file")?;

        Ok(())
    }

    pub fn add(&mut self, name: String, source: String) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.skills.insert(
            name,
            SkillEntry {
                source,
                installed_at: now.clone(),
                last_synced: now,
            },
        );
        self.save()
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.skills.remove(name);
        self.save()
    }

    pub fn update_sync_time(&mut self, name: &str) -> Result<()> {
        if let Some(entry) = self.skills.get_mut(name) {
            entry.last_synced = chrono::Utc::now().to_rfc3339();
            self.save()?;
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&SkillEntry> {
        self.skills.get(name)
    }

    fn default_registry_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?;
        Ok(config_dir.join("skillz").join("registry.toml"))
    }
}
