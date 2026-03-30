use crate::config::Config;
use crate::registry::Registry;
use crate::validate;
use anyhow::{Context, Result};
use std::process::Command;

pub fn update_skill(config: &Config, name: &str) -> Result<()> {
    let mut registry = Registry::load()?;

    let entry = registry
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found in registry", name))?
        .clone();

    let skills_dir = config.skills_dir();
    let skill_path = skills_dir.join(name);

    if !skill_path.exists() {
        anyhow::bail!("Skill directory not found: {}", skill_path.display());
    }

    println!("Updating {} from {}...", name, entry.source);

    // Remove existing directory
    std::fs::remove_dir_all(&skill_path).context("Failed to remove old skill directory")?;

    // Clone fresh copy
    let output = Command::new("git")
        .args(&["clone", &entry.source, skill_path.to_str().unwrap()])
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git clone failed: {}", stderr);
    }

    // Validate skill
    let validation = validate::validate_skill(&skill_path)?;

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

    // Remove .git directory
    let git_dir = skill_path.join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir).ok();
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
