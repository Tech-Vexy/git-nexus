use anyhow::Result;
use git2::Repository;
use serde::Deserialize;
use std::path::Path;

// GitHub API integration is scaffolded but not fully implemented
// This module will be completed in a future version
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GitHubIssue {
    number: u32,
    title: String,
    state: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GitHubPR {
    number: u32,
    title: String,
    state: String,
}

#[allow(dead_code)]
pub struct GitHubInfo {
    pub open_issues: usize,
    pub open_prs: usize,
}

#[allow(dead_code)]
pub fn get_github_info(repo_path: &Path, token: Option<&str>) -> Result<Option<GitHubInfo>> {
    let repo = Repository::open(repo_path)?;
    
    // Try to get remote URL
    let remote = repo.find_remote("origin").ok();
    let url = remote.and_then(|r| r.url().map(String::from));
    
    if let Some(url) = url {
        if let Some((owner, repo_name)) = parse_github_url(&url) {
            return fetch_github_data(&owner, &repo_name, token);
        }
    }
    
    Ok(None)
}

#[allow(dead_code)]
fn parse_github_url(url: &str) -> Option<(String, String)> {
    // Handle https://github.com/owner/repo.git
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.trim_end_matches(".git").split('/').collect();
        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    
    // Handle git@github.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let parts: Vec<&str> = rest.trim_end_matches(".git").split('/').collect();
        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    
    None
}

#[allow(dead_code)]
fn fetch_github_data(owner: &str, repo: &str, token: Option<&str>) -> Result<Option<GitHubInfo>> {
    let client = reqwest::blocking::Client::new();
    let base_url = "https://api.github.com";
    
    let mut builder = client.get(format!("{}/repos/{}/{}", base_url, owner, repo));
    
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("token {}", token));
    }
    
    builder = builder.header("User-Agent", "git-nexus");
    
    let response = builder.send();
    
    if response.is_err() {
        return Ok(None); // GitHub API might be unreachable
    }
    
    // Get issues count
    let issues_url = format!("{}/repos/{}/{}/issues?state=open&per_page=1", base_url, owner, repo);
    let mut issues_builder = client.get(&issues_url).header("User-Agent", "git-nexus");
    if let Some(token) = token {
        issues_builder = issues_builder.header("Authorization", format!("token {}", token));
    }
    
    let issues_response = issues_builder.send();
    let open_issues = if let Ok(resp) = issues_response {
        if let Some(link_header) = resp.headers().get("link") {
            parse_link_header(link_header.to_str().unwrap_or(""))
        } else {
            0
        }
    } else {
        0
    };
    
    // Get PRs count
    let prs_url = format!("{}/repos/{}/{}/pulls?state=open&per_page=1", base_url, owner, repo);
    let mut prs_builder = client.get(&prs_url).header("User-Agent", "git-nexus");
    if let Some(token) = token {
        prs_builder = prs_builder.header("Authorization", format!("token {}", token));
    }
    
    let prs_response = prs_builder.send();
    let open_prs = if let Ok(resp) = prs_response {
        if let Some(link_header) = resp.headers().get("link") {
            parse_link_header(link_header.to_str().unwrap_or(""))
        } else {
            0
        }
    } else {
        0
    };
    
    Ok(Some(GitHubInfo {
        open_issues,
        open_prs,
    }))
}

#[allow(dead_code)]
fn parse_link_header(link: &str) -> usize {
    // Parse GitHub link header to get total count
    // Example: <url?page=2>; rel="next", <url?page=10>; rel="last"
    for part in link.split(',') {
        if part.contains("rel=\"last\"") {
            if let Some(page_str) = part.split("page=").nth(1) {
                if let Some(num_str) = page_str.split('>').next() {
                    if let Ok(num) = num_str.parse::<usize>() {
                        return num;
                    }
                }
            }
        }
    }
    0
}
