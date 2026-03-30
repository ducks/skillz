use anyhow::{Context, Result};
use std::process::Command;
use crate::config::Config;
use crate::registry::Registry;

pub fn update_skill(config: &Config, name: &str) -> Result<()> {
    let mut registry = Registry::load()?;

    let entry = registry.get(name)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found in registry", name))?
        .clone();

    let skills_dir = config.skills_dir();
    let skill_path = skills_dir.join(name);

    if !skill_path.exists() {
        anyhow::bail!("Skill directory not found: {}", skill_path.display());
    }

    println!("Updating {} from {}...", name, entry.source);

    // Remove existing directory
    std::fs::remove_dir_all(&skill_path)
        .context("Failed to remove old skill directory")?;

    // Clone fresh copy
    let output = Command::new("git")
        .args(&["clone", &entry.source, skill_path.to_str().unwrap()])
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git clone failed: {}", stderr);
    }

    // Validate SKILL.md still exists
    let skill_file = skill_path.join("SKILL.md");
    if !skill_file.exists() {
        anyhow::bail!("Updated repository no longer contains SKILL.md");
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

pub fn update_all(config: &Config) -> Result<()> {
    let registry = Registry::load()?;

    if registry.skills.is_empty() {
        println!("No skills installed.");
        return Ok(());
    }

    let skill_names: Vec<String> = registry.skills.keys().cloned().collect();

    println!("Updating {} skill(s)...\n", skill_names.len());

    let mut updated = 0;
    let mut failed = 0;

    for name in skill_names {
        match update_skill(config, &name) {
            Ok(_) => {
                updated += 1;
                println!();
            }
            Err(e) => {
                eprintln!("Failed to update {}: {}", name, e);
                failed += 1;
                println!();
            }
        }
    }

    println!("Update complete: {} updated, {} failed", updated, failed);
    Ok(())
}
