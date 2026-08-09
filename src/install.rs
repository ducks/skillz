use crate::config::Config;
use crate::registry::Registry;
use crate::validate;
use anyhow::{Context, Result};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSpec {
    pub repo_url: String,
    pub subdir: Option<PathBuf>,
    pub skill_name: String,
}

impl SourceSpec {
    pub fn registry_source(&self) -> String {
        match &self.subdir {
            Some(subdir) => format!("{}#{}", self.repo_url, subdir.display()),
            None => self.repo_url.clone(),
        }
    }
}

pub fn install(config: &Config, source: &str, target: Option<&str>) -> Result<()> {
    let spec = parse_source(source)?;
    let mut registry = Registry::load()?;
    if registry.get(&spec.skill_name).is_some() {
        anyhow::bail!(
            "Skill '{}' is already registered; remove it before installing another copy",
            spec.skill_name
        );
    }
    let skills_dir = config.skills_dir_for(target)?;
    std::fs::create_dir_all(&skills_dir).with_context(|| {
        format!(
            "Failed to create skills directory: {}",
            skills_dir.display()
        )
    })?;

    let target_path = skills_dir.join(&spec.skill_name);
    if target_path.exists() {
        anyhow::bail!(
            "Skill '{}' already exists at {}",
            spec.skill_name,
            target_path.display()
        );
    }

    println!(
        "Installing {} from {}...",
        spec.skill_name,
        spec.registry_source()
    );

    let checkout = checkout_source(&spec, &skills_dir)?;
    let skill_path = selected_skill_path(checkout.path(), &spec)?;
    let validation = validate::validate_skill(&skill_path)?;

    if !validation.valid {
        eprintln!("\nValidation failed:");
        for error in &validation.errors {
            eprintln!("  ✗ {}", error);
        }
        anyhow::bail!("Skill validation failed");
    }

    if !validation.warnings.is_empty() {
        println!("\nValidation warnings:");
        for warning in &validation.warnings {
            println!("  ⚠ Line {}: {}", warning.line, warning.message);
        }
        println!(
            "\nProceed with installation? The skill will still work, but you should review these warnings."
        );
        print!("Continue? [y/N] ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            anyhow::bail!("Installation cancelled by user");
        }
    }

    copy_skill(&skill_path, &target_path)?;
    let registry_source = spec.registry_source();
    let install_path = target_path.to_string_lossy().into_owned();
    registry.add(spec.skill_name.clone(), registry_source, Some(install_path))?;

    println!("Successfully installed: {}", spec.skill_name);
    println!("Location: {}", target_path.display());
    Ok(())
}

pub fn parse_source(source: &str) -> Result<SourceSpec> {
    let (repo_source, subdir) = match source.split_once('#') {
        Some((repo, path)) => (repo, Some(parse_subdir(path)?)),
        None => (source, None),
    };

    let repo_url = if repo_source.starts_with("https://github.com/")
        || repo_source.starts_with("http://github.com/")
    {
        repo_source.to_string()
    } else if let Some(path) = repo_source.strip_prefix("github:") {
        format!("https://github.com/{}", path)
    } else if Path::new(repo_source).exists() {
        anyhow::bail!(
            "Local path installation not yet supported. Use a GitHub URL or github:user/repo format."
        );
    } else {
        anyhow::bail!(
            "Invalid source format. Use https://github.com/user/repo or github:user/repo"
        );
    };

    let skill_name = match &subdir {
        Some(path) => path
            .file_name()
            .and_then(|name| name.to_str())
            .context("Could not extract skill name from subdirectory")?
            .to_string(),
        None => extract_repo_name(&repo_url)?,
    };

    if !skill_name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    {
        anyhow::bail!("Skill name contains unsafe characters: {}", skill_name);
    }

    Ok(SourceSpec {
        repo_url,
        subdir,
        skill_name,
    })
}

pub fn checkout_source(spec: &SourceSpec, parent: &Path) -> Result<tempfile::TempDir> {
    let checkout = tempfile::Builder::new()
        .prefix(".skillz-checkout-")
        .tempdir_in(parent)
        .context("Failed to create temporary checkout directory")?;

    let output = Command::new("git")
        .args(["clone", "--depth", "1", &spec.repo_url])
        .arg(checkout.path())
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git clone failed: {}", stderr.trim());
    }

    Ok(checkout)
}

pub fn selected_skill_path(checkout: &Path, spec: &SourceSpec) -> Result<PathBuf> {
    let selected = match &spec.subdir {
        Some(subdir) => checkout.join(subdir),
        None => checkout.to_path_buf(),
    };

    if !selected.is_dir() {
        anyhow::bail!(
            "Skill subdirectory does not exist in repository: {}",
            spec.subdir
                .as_deref()
                .unwrap_or_else(|| Path::new("."))
                .display()
        );
    }
    Ok(selected)
}

pub fn copy_skill(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        anyhow::bail!(
            "Skill destination already exists: {}",
            destination.display()
        );
    }
    copy_directory(source, destination)
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)
        .with_context(|| format!("Failed to create {}", destination.display()))?;

    for entry in
        std::fs::read_dir(source).with_context(|| format!("Failed to read {}", source.display()))?
    {
        let entry = entry?;
        if entry.file_name() == ".git" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            std::fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        } else {
            anyhow::bail!(
                "Unsupported filesystem entry in skill: {}",
                source_path.display()
            );
        }
    }
    Ok(())
}

fn parse_subdir(value: &str) -> Result<PathBuf> {
    if value.is_empty() || value.contains('\\') || value.contains('#') {
        anyhow::bail!("Skill subdirectory must be a non-empty repository-relative path");
    }
    let path = PathBuf::from(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        anyhow::bail!("Skill subdirectory must not contain '.' or '..' components");
    }
    Ok(path)
}

fn extract_repo_name(url: &str) -> Result<String> {
    let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
    if let Some(name) = parts.last() {
        Ok(name.trim_end_matches(".git").to_string())
    } else {
        anyhow::bail!("Could not extract repository name from URL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_https() {
        let spec = parse_source("https://github.com/user/my-skill").unwrap();
        assert_eq!(spec.repo_url, "https://github.com/user/my-skill");
        assert_eq!(spec.skill_name, "my-skill");
        assert_eq!(spec.subdir, None);
    }

    #[test]
    fn parses_github_shorthand() {
        let spec = parse_source("github:user/my-skill").unwrap();
        assert_eq!(spec.repo_url, "https://github.com/user/my-skill");
        assert_eq!(spec.skill_name, "my-skill");
    }

    #[test]
    fn parses_monorepo_subdirectory() {
        let spec =
            parse_source("github:ducks/replaybook#skills/replaybook-build-scenario").unwrap();
        assert_eq!(spec.repo_url, "https://github.com/ducks/replaybook");
        assert_eq!(
            spec.subdir,
            Some(PathBuf::from("skills/replaybook-build-scenario"))
        );
        assert_eq!(spec.skill_name, "replaybook-build-scenario");
        assert_eq!(
            spec.registry_source(),
            "https://github.com/ducks/replaybook#skills/replaybook-build-scenario"
        );
    }

    #[test]
    fn rejects_unsafe_subdirectories() {
        assert!(parse_source("github:user/repo#../skill").is_err());
        assert!(parse_source("github:user/repo#/skill").is_err());
        assert!(parse_source("github:user/repo#").is_err());
    }

    #[test]
    fn copies_only_selected_skill_contents() {
        let source = tempfile::tempdir().unwrap();
        let destination_parent = tempfile::tempdir().unwrap();
        std::fs::write(source.path().join("SKILL.md"), "# Test").unwrap();
        std::fs::create_dir(source.path().join("assets")).unwrap();
        std::fs::write(source.path().join("assets/example.txt"), "example").unwrap();
        std::fs::create_dir(source.path().join(".git")).unwrap();
        std::fs::write(source.path().join(".git/config"), "ignored").unwrap();

        let destination = destination_parent.path().join("test-skill");
        copy_skill(source.path(), &destination).unwrap();

        assert_eq!(
            std::fs::read_to_string(destination.join("SKILL.md")).unwrap(),
            "# Test"
        );
        assert!(destination.join("assets/example.txt").is_file());
        assert!(!destination.join(".git").exists());
    }

    #[test]
    fn extracts_repo_name_with_git_suffix() {
        let spec = parse_source("https://github.com/user/repo.git").unwrap();
        assert_eq!(spec.skill_name, "repo");
    }
}
