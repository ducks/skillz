use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use crate::config::Config;
use crate::registry::Registry;

pub fn install(config: &Config, source: &str) -> Result<()> {
    let (repo_url, skill_name) = parse_source(source)?;
    let mut registry = Registry::load()?;

    let skills_dir = config.skills_dir();
    std::fs::create_dir_all(&skills_dir)
        .context(format!("Failed to create skills directory: {}", skills_dir.display()))?;

    let target_path = skills_dir.join(&skill_name);

    if target_path.exists() {
        anyhow::bail!("Skill '{}' already exists at {}", skill_name, target_path.display());
    }

    println!("Installing {} from {}...", skill_name, repo_url);

    // Clone the repository
    let output = Command::new("git")
        .args(&["clone", &repo_url, target_path.to_str().unwrap()])
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git clone failed: {}", stderr);
    }

    // Validate that SKILL.md exists
    let skill_file = target_path.join("SKILL.md");
    if !skill_file.exists() {
        // Clean up the cloned directory
        std::fs::remove_dir_all(&target_path).ok();
        anyhow::bail!("Invalid skill: SKILL.md not found in repository");
    }

    // Remove .git directory to avoid nested git repos (optional)
    let git_dir = target_path.join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir).ok();
    }

    // Add to registry
    registry.add(skill_name.clone(), repo_url.clone())?;

    println!("Successfully installed: {}", skill_name);
    println!("Location: {}", target_path.display());

    Ok(())
}

fn parse_source(source: &str) -> Result<(String, String)> {
    // Handle different source formats:
    // 1. https://github.com/user/repo
    // 2. github:user/repo
    // 3. Local path (future: could copy instead of clone)

    if source.starts_with("https://github.com/") || source.starts_with("http://github.com/") {
        let repo_url = source.to_string();
        let skill_name = extract_repo_name(&repo_url)?;
        Ok((repo_url, skill_name))
    } else if source.starts_with("github:") {
        let path = source.strip_prefix("github:").unwrap();
        let repo_url = format!("https://github.com/{}", path);
        let skill_name = extract_repo_name(&repo_url)?;
        Ok((repo_url, skill_name))
    } else if Path::new(source).exists() {
        // Local path - copy instead of clone
        anyhow::bail!("Local path installation not yet supported. Use GitHub URL or github:user/repo format.");
    } else {
        anyhow::bail!("Invalid source format. Use https://github.com/user/repo or github:user/repo");
    }
}

fn extract_repo_name(url: &str) -> Result<String> {
    let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
    if let Some(name) = parts.last() {
        let name = name.trim_end_matches(".git");
        Ok(name.to_string())
    } else {
        anyhow::bail!("Could not extract repository name from URL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_https() {
        let (url, name) = parse_source("https://github.com/user/my-skill").unwrap();
        assert_eq!(url, "https://github.com/user/my-skill");
        assert_eq!(name, "my-skill");
    }

    #[test]
    fn test_parse_github_shorthand() {
        let (url, name) = parse_source("github:user/my-skill").unwrap();
        assert_eq!(url, "https://github.com/user/my-skill");
        assert_eq!(name, "my-skill");
    }

    #[test]
    fn test_extract_repo_name_with_git() {
        let name = extract_repo_name("https://github.com/user/repo.git").unwrap();
        assert_eq!(name, "repo");
    }
}
