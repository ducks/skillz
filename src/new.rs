use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn new_skill(name: &str, path: Option<&str>) -> Result<()> {
    // Determine target directory
    let target = if let Some(p) = path {
        Path::new(p).join(name)
    } else {
        Path::new(".").join(name)
    };

    if target.exists() {
        anyhow::bail!("Directory already exists: {}", target.display());
    }

    println!("Creating new skill: {}", name);
    println!("Location: {}\n", target.display());

    // Create directory
    fs::create_dir_all(&target)
        .context(format!("Failed to create directory: {}", target.display()))?;

    // Create SKILL.md with template
    let skill_content = format!(r#"# {}

A Claude Code skill for [brief description].

## What this skill does

[Explain what the skill helps with]

## Usage

```bash
# Example usage
```

## Instructions

[Detailed instructions for Claude on how to use this skill]

## Examples

[Example scenarios where this skill is useful]
"#, name);

    let skill_file = target.join("SKILL.md");
    fs::write(&skill_file, skill_content)
        .context("Failed to write SKILL.md")?;

    // Create README.md
    let readme_content = format!(r#"# {}

A Claude Code skill.

## Installation

```bash
skillz install github:username/{}
```

## Description

[Add description here]

## Usage

Once installed, the skill will be available to Claude Code.

## License

MIT OR Apache-2.0
"#, name, name);

    let readme_file = target.join("README.md");
    fs::write(&readme_file, readme_content)
        .context("Failed to write README.md")?;

    // Initialize git repo
    let git_init = Command::new("git")
        .args(&["init"])
        .current_dir(&target)
        .output();

    if git_init.is_ok() {
        // Create .gitignore
        let gitignore = target.join(".gitignore");
        fs::write(&gitignore, ".DS_Store\n")
            .context("Failed to write .gitignore")?;

        println!("✓ Created skill directory: {}", target.display());
        println!("✓ Created SKILL.md (edit this with your skill prompt)");
        println!("✓ Created README.md");
        println!("✓ Initialized git repository");
        println!("\nNext steps:");
        println!("  1. cd {}", name);
        println!("  2. Edit SKILL.md with your skill prompt");
        println!("  3. git add . && git commit -m \"Initial skill\"");
        println!("  4. Create GitHub repo and push");
        println!("  5. Install with: skillz install github:username/{}", name);
    } else {
        println!("✓ Created skill directory: {}", target.display());
        println!("✓ Created SKILL.md (edit this with your skill prompt)");
        println!("✓ Created README.md");
        println!("⚠ Git not available - skipped git init");
        println!("\nNext steps:");
        println!("  1. cd {}", name);
        println!("  2. Edit SKILL.md with your skill prompt");
        println!("  3. Initialize git and push to GitHub");
        println!("  4. Install with: skillz install github:username/{}", name);
    }

    Ok(())
}
