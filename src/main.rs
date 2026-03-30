use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod config;
mod install;
mod registry;
mod search;
mod update;
mod validate;

use config::Config;
use registry::Registry;

#[derive(Parser)]
#[command(name = "skillz")]
#[command(about = "Claude Code skill package manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a skill from GitHub or local path
    Install {
        /// GitHub URL (https://github.com/user/repo) or github:user/repo or local path
        source: String,
    },
    /// List installed skills
    List,
    /// Remove an installed skill
    Remove {
        /// Skill name to remove
        name: String,
    },
    /// Update installed skill(s)
    Update {
        /// Skill name to update (omit to update all)
        name: Option<String>,
        /// Auto-sync all skills on startup (check for updates)
        #[arg(long)]
        auto: bool,
    },
    /// Search GitHub for skills
    Search {
        /// Search query
        query: String,
    },
    /// Configure skillz
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set a config value
    Set {
        /// Config key (e.g., "skills-dir")
        key: String,
        /// Config value
        value: String,
    },
    /// Get a config value
    Get {
        /// Config key
        key: String,
    },
    /// Show all config values
    Show,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Commands::Install { source } => {
            install::install(&config, &source)?;
        }
        Commands::List => {
            list_skills(&config)?;
        }
        Commands::Remove { name } => {
            remove_skill(&config, &name)?;
        }
        Commands::Update { name, auto } => {
            if let Some(skill_name) = name {
                update::update_skill(&config, &skill_name)?;
            } else {
                update::update_all(&config, auto)?;
            }
        }
        Commands::Search { query } => {
            search::search_skills(&query)?;
        }
        Commands::Config { action } => {
            handle_config(action)?;
        }
    }

    Ok(())
}

fn list_skills(config: &Config) -> Result<()> {
    let registry = Registry::load()?;
    let skills_dir = config.skills_dir();

    if registry.skills.is_empty() {
        println!("No skills installed.");
        return Ok(());
    }

    println!("Installed skills in {}:\n", skills_dir.display());

    let mut skills: Vec<_> = registry.skills.iter().collect();
    skills.sort_by_key(|(name, _)| *name);

    for (name, entry) in skills {
        let installed = chrono::DateTime::parse_from_rfc3339(&entry.installed_at)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| entry.installed_at.clone());

        let synced = chrono::DateTime::parse_from_rfc3339(&entry.last_synced)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| entry.last_synced.clone());

        println!("  {} - {}", name, entry.source);
        println!("    installed: {}  |  last synced: {}", installed, synced);
        println!();
    }

    Ok(())
}

fn remove_skill(config: &Config, name: &str) -> Result<()> {
    let mut registry = Registry::load()?;
    let skill_path = config.skills_dir().join(name);

    if !skill_path.exists() {
        anyhow::bail!("Skill '{}' not found", name);
    }

    let skill_file = skill_path.join("SKILL.md");
    if !skill_file.exists() {
        anyhow::bail!("'{}' exists but is not a valid skill (no SKILL.md)", name);
    }

    std::fs::remove_dir_all(&skill_path)
        .context(format!("Failed to remove skill directory: {}", skill_path.display()))?;

    // Remove from registry
    registry.remove(name)?;

    println!("Removed skill: {}", name);
    Ok(())
}

fn handle_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Set { key, value } => {
            let mut config = Config::load()?;
            config.set(&key, &value)?;
            config.save()?;
            println!("Set {} = {}", key, value);
        }
        ConfigAction::Get { key } => {
            let config = Config::load()?;
            if let Some(value) = config.get(&key) {
                println!("{}", value);
            } else {
                println!("Key '{}' not found", key);
            }
        }
        ConfigAction::Show => {
            let config = Config::load()?;
            println!("Config file: {}", config.config_path().display());
            println!("\nSettings:");
            println!("  skills-dir = {}", config.skills_dir().display());
        }
    }
    Ok(())
}
