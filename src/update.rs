use crate::config::Config;
use crate::install;
use crate::registry::Registry;
use crate::validate;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn update_skill(config: &Config, name: &str) -> Result<()> {
    let mut registry = Registry::load()?;

    let entry = registry
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found in registry", name))?
        .clone();

    let skill_path = entry
        .install_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| config.skills_dir().join(name));

    if !skill_path.exists() {
        anyhow::bail!("Skill directory not found: {}", skill_path.display());
    }

    println!("Updating {} from {}...", name, entry.source);
    let spec = install::parse_source(&entry.source)?;
    let parent = skill_path
        .parent()
        .context("Skill install path has no parent directory")?;
    std::fs::create_dir_all(parent)?;
    let checkout = install::checkout_source(&spec, parent)?;
    let selected = install::selected_skill_path(checkout.path(), &spec)?;

    let validation = validate::validate_skill(&selected)?;

    if !validation.valid {
        eprintln!("\nValidation failed for updated skill:");
        for error in &validation.errors {
            eprintln!("  ✗ {}", error);
        }
        anyhow::bail!("Updated skill validation failed");
    }

    // Show warnings if any
    if !validation.warnings.is_empty() {
        println!("\nValidation warnings for updated skill:");
        for warning in &validation.warnings {
            println!("  ⚠ Line {}: {}", warning.line, warning.message);
        }
    }

    let staged = tempfile::Builder::new()
        .prefix(".skillz-update-")
        .tempdir_in(parent)
        .context("Failed to create update staging directory")?;
    let staged_skill = staged.path().join(name);
    install::copy_skill(&selected, &staged_skill)?;

    let backup = tempfile::Builder::new()
        .prefix(".skillz-backup-")
        .tempdir_in(parent)
        .context("Failed to create update backup directory")?;
    let backup_skill = backup.path().join(name);
    std::fs::rename(&skill_path, &backup_skill).context("Failed to stage existing skill")?;
    if let Err(error) = std::fs::rename(&staged_skill, &skill_path) {
        let _ = std::fs::rename(&backup_skill, &skill_path);
        return Err(error).context("Failed to activate updated skill");
    }

    // Update sync time in registry
    registry.update_sync_time(name)?;

    println!("Successfully updated: {}", name);
    Ok(())
}

pub fn update_all(config: &Config, auto: bool) -> Result<()> {
    let registry = Registry::load()?;

    if registry.skills.is_empty() {
        if !auto {
            println!("No skills installed.");
        }
        return Ok(());
    }

    let skill_names: Vec<String> = registry.skills.keys().cloned().collect();

    if auto {
        println!("Auto-syncing {} skill(s)...", skill_names.len());
    } else {
        println!("Updating {} skill(s)...\n", skill_names.len());
    }

    let mut updated = 0;
    let mut failed = 0;

    for name in skill_names {
        match update_skill(config, &name) {
            Ok(_) => {
                updated += 1;
                if !auto {
                    println!();
                }
            }
            Err(e) => {
                if !auto {
                    eprintln!("Failed to update {}: {}", name, e);
                    println!();
                }
                failed += 1;
            }
        }
    }

    if auto {
        if updated > 0 {
            println!("✓ Auto-sync complete: {} updated", updated);
        }
    } else {
        println!("Update complete: {} updated, {} failed", updated, failed);
    }

    Ok(())
}
