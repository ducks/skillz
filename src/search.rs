use anyhow::Result;
use std::process::Command;

pub fn search_skills(query: &str) -> Result<()> {
    println!("Searching GitHub for skills matching '{}'...\n", query);

    // Use repository search (doesn't require auth)
    // Search in name, description, and README for query + "skill" OR "SKILL.md"
    let search_query = format!("{} skill OR SKILL.md in:name,description,readme", query);
    let api_url = format!(
        "https://api.github.com/search/repositories?q={}",
        urlencoding::encode(&search_query)
    );

    // Check for optional GitHub token for higher rate limits
    let github_token = std::env::var("GITHUB_TOKEN").ok();

    let mut curl_args = vec![
        "-s",
        "-H", "Accept: application/vnd.github.v3+json",
    ];

    let auth_header;
    if let Some(token) = &github_token {
        auth_header = format!("Authorization: token {}", token);
        curl_args.push("-H");
        curl_args.push(&auth_header);
    }

    curl_args.push(&api_url);

    // Use curl to fetch results
    let output = Command::new("curl")
        .args(&curl_args)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("GitHub API request failed");
    }

    let response_text = String::from_utf8_lossy(&output.stdout);

    // Parse JSON response
    let response: serde_json::Value = serde_json::from_str(&response_text)?;

    // Check for errors
    if let Some(message) = response.get("message").and_then(|m| m.as_str()) {
        if message.contains("rate limit") {
            anyhow::bail!("GitHub API rate limit exceeded. Set GITHUB_TOKEN env var for higher limits.");
        }
        anyhow::bail!("GitHub API error: {}", message);
    }

    // Extract repositories from repository search results
    let mut repos = std::collections::HashMap::new();

    if let Some(items) = response.get("items").and_then(|i| i.as_array()) {
        for repo in items {
            let name = repo.get("full_name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");
            let url = repo.get("html_url")
                .and_then(|u| u.as_str())
                .unwrap_or("");
            let description = repo.get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());
            let stars = repo.get("stargazers_count")
                .and_then(|s| s.as_u64())
                .unwrap_or(0) as u32;

            repos.insert(name.to_string(), (url.to_string(), description, stars));
        }
    }

    if repos.is_empty() {
        println!("No skills found matching '{}'", query);
        println!("\nTip: Skills must have a SKILL.md file in their repository.");
        return Ok(());
    }

    println!("Found {} skill repositories:\n", repos.len());

    // Sort by stars
    let mut repos_vec: Vec<_> = repos.into_iter().collect();
    repos_vec.sort_by(|a, b| b.1.2.cmp(&a.1.2));

    for (name, (url, description, stars)) in repos_vec {
        println!("  {} - ⭐ {}", name, stars);
        if let Some(desc) = description {
            if !desc.is_empty() {
                println!("    {}", desc);
            }
        }
        println!("    {}", url);
        println!("    Install: skillz install {}", url);
        println!();
    }

    Ok(())
}
