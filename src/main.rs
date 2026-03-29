use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod config;
mod install;

use config::Config;

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
        Commands::Config { action } => {
            handle_config(action)?;
        }
    }

    Ok(())
}

fn list_skills(config: &Config) -> Result<()> {
    let skills_dir = config.skills_dir();

    if !skills_dir.exists() {
        println!("No skills directory found at: {}", skills_dir.display());
        return Ok(());
    }

    println!("Installed skills in {}:\n", skills_dir.display());

    let mut found_any = false;
    for entry in std::fs::read_dir(&skills_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                found_any = true;
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                println!("  - {}", name);
            }
        }
    }

    if !found_any {
        println!("  (no skills installed)");
    }

    Ok(())
}

fn remove_skill(config: &Config, name: &str) -> Result<()> {
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
