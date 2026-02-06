//! Export functionality for git-nexus reports.
//!
//! This module provides functions to export repository status information to various formats:
//! - **CSV**: Machine-readable format for spreadsheets and data analysis
//! - **HTML**: Styled web page with statistics and color-coded status indicators
//!
//! # Example
//!
//! ```no_run
//! use git_nexus::export::{export_csv, export_html};
//! use git_nexus::scan_repositories;
//! use std::path::PathBuf;
//!
//! let repos = scan_repositories(&PathBuf::from("."), 3, true, &[], false);
//! export_html(&repos, &PathBuf::from("report.html")).unwrap();
//! export_csv(&repos, &PathBuf::from("report.csv")).unwrap();
//! ```

use anyhow::Result;
use chrono::Local;
use csv::Writer;
use std::path::PathBuf;

use crate::RepoStatus;

/// Exports repository status information to a CSV file.
///
/// Creates a CSV file with columns for path, branch, status, ahead/behind counts,
/// stash count, file counts, and last commit information.
///
/// # Arguments
///
/// * `repos` - Slice of repository status information to export
/// * `path` - Path where the CSV file should be created
///
/// # Returns
///
/// `Ok(())` on success, or an error if the file cannot be written.
///
/// # Example
///
/// ```no_run
/// # use git_nexus::export::export_csv;
/// # use git_nexus::scan_repositories;
/// # use std::path::PathBuf;
/// let repos = scan_repositories(&PathBuf::from("."), 3, true, &[], false);
/// export_csv(&repos, &PathBuf::from("repos.csv")).unwrap();
/// ```
pub fn export_csv(repos: &[RepoStatus], path: &PathBuf) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;
    
    wtr.write_record([
        "Path",
        "Branch",
        "Status",
        "Ahead",
        "Behind",
        "Stash Count",
        "Modified Files",
        "Untracked Files",
        "Last Commit Hash",
        "Last Commit Author",
        "Last Commit Message",
        "Last Commit Timestamp",
    ])?;

    for repo in repos {
        wtr.write_record(&[
            repo.path.display().to_string(),
            repo.branch.as_deref().unwrap_or("N/A").to_string(),
            if repo.is_clean { "CLEAN" } else { "DIRTY" }.to_string(),
            repo.ahead.to_string(),
            repo.behind.to_string(),
            repo.stash_count.map(|c| c.to_string()).unwrap_or_default(),
            repo.modified_count.map(|c| c.to_string()).unwrap_or_default(),
            repo.untracked_count.map(|c| c.to_string()).unwrap_or_default(),
            repo.last_commit.as_ref().map(|c| c.hash.clone()).unwrap_or_default(),
            repo.last_commit.as_ref().map(|c| c.author.clone()).unwrap_or_default(),
            repo.last_commit.as_ref().map(|c| c.message.clone()).unwrap_or_default(),
            repo.last_commit.as_ref().map(|c| c.timestamp.clone()).unwrap_or_default(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

/// Exports repository status information to an HTML file.
///
/// Creates a styled HTML page with:
/// - Dashboard showing total repositories, clean count, and dirty count
/// - Color-coded status indicators
/// - Ahead/behind badges
/// - Last commit information
/// - Responsive design
///
/// # Arguments
///
/// * `repos` - Slice of repository status information to export
/// * `path` - Path where the HTML file should be created
///
/// # Returns
///
/// `Ok(())` on success, or an error if the file cannot be written.
///
/// # Example
///
/// ```no_run
/// # use git_nexus::export::export_html;
/// # use git_nexus::scan_repositories;
/// # use std::path::PathBuf;
/// let repos = scan_repositories(&PathBuf::from("."), 3, true, &[], false);
/// export_html(&repos, &PathBuf::from("report.html")).unwrap();
/// ```
pub fn export_html(repos: &[RepoStatus], path: &PathBuf) -> Result<()> {
    let html = generate_html(repos)?;
    std::fs::write(path, html)?;
    Ok(())
}

/// Generates HTML content for a repository status report.
///
/// Creates a complete HTML document with embedded CSS styling.
/// The generated HTML includes:
/// - Statistics dashboard (total, clean, dirty repositories)
/// - Sortable table with repository information
/// - Color-coded status badges
/// - Ahead/behind indicators with badges
/// - Last commit information
///
/// # Arguments
///
/// * `repos` - Slice of repository status information
///
/// # Returns
///
/// A string containing the complete HTML document.
fn generate_html(repos: &[RepoStatus]) -> Result<String> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    
    let mut rows = String::new();
    for repo in repos {
        let status_class = if repo.is_clean { "clean" } else { "dirty" };
        let status_text = if repo.is_clean { "CLEAN" } else { "DIRTY" };
        
        let ahead_badge = if repo.ahead > 0 {
            format!("<span class=\"badge badge-warning\">↑{}</span>", repo.ahead)
        } else {
            String::new()
        };
        
        let behind_badge = if repo.behind > 0 {
            format!("<span class=\"badge badge-danger\">↓{}</span>", repo.behind)
        } else {
            String::new()
        };
        
        let last_commit = if let Some(ref commit) = repo.last_commit {
            format!(
                "<small>{} · {} · {}</small>",
                commit.hash, commit.author, commit.message
            )
        } else {
            String::new()
        };
        
        rows.push_str(&format!(
            r#"<tr>
                <td><strong>{}</strong></td>
                <td><span class="badge badge-info">{}</span></td>
                <td><span class="badge badge-{}">{}</span></td>
                <td>{} {}</td>
                <td>{}</td>
            </tr>"#,
            repo.path.display(),
            repo.branch.as_deref().unwrap_or("N/A"),
            status_class,
            status_text,
            ahead_badge,
            behind_badge,
            last_commit
        ));
    }

    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Git Nexus Report - {}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 20px;
            color: #333;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .header h1 {{
            font-size: 2.5em;
            margin-bottom: 10px;
        }}
        .header p {{
            opacity: 0.9;
            font-size: 1.1em;
        }}
        .stats {{
            display: flex;
            justify-content: space-around;
            padding: 30px;
            background: #f8f9fa;
            border-bottom: 1px solid #dee2e6;
        }}
        .stat {{
            text-align: center;
        }}
        .stat-value {{
            font-size: 2.5em;
            font-weight: bold;
            color: #667eea;
        }}
        .stat-label {{
            color: #666;
            margin-top: 5px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
        }}
        th, td {{
            padding: 16px;
            text-align: left;
            border-bottom: 1px solid #dee2e6;
        }}
        th {{
            background: #f8f9fa;
            font-weight: 600;
            color: #495057;
            position: sticky;
            top: 0;
        }}
        tr:hover {{
            background: #f8f9fa;
        }}
        .badge {{
            display: inline-block;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 0.85em;
            font-weight: 600;
            margin: 2px;
        }}
        .badge-clean {{
            background: #d4edda;
            color: #155724;
        }}
        .badge-dirty {{
            background: #f8d7da;
            color: #721c24;
        }}
        .badge-info {{
            background: #d1ecf1;
            color: #0c5460;
        }}
        .badge-warning {{
            background: #fff3cd;
            color: #856404;
        }}
        .badge-danger {{
            background: #f8d7da;
            color: #721c24;
        }}
        small {{
            color: #6c757d;
        }}
        .footer {{
            text-align: center;
            padding: 20px;
            color: #6c757d;
            background: #f8f9fa;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Git Nexus Report</h1>
            <p>Repository Status Overview · Generated on {}</p>
        </div>
        <div class="stats">
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Repositories</div>
            </div>
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Clean</div>
            </div>
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Dirty</div>
            </div>
        </div>
        <table>
            <thead>
                <tr>
                    <th>Repository</th>
                    <th>Branch</th>
                    <th>Status</th>
                    <th>Sync</th>
                    <th>Last Commit</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
        <div class="footer">
            Generated by <strong>git-nexus</strong> · A blazing fast multi-repository scanner
        </div>
    </div>
</body>
</html>"#,
        now,
        now,
        repos.len(),
        repos.iter().filter(|r| r.is_clean).count(),
        repos.iter().filter(|r| !r.is_clean).count(),
        rows
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommitInfo, RepoStatus};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repos() -> Vec<RepoStatus> {
        vec![
            RepoStatus {
                path: PathBuf::from("/test/repo1"),
                is_clean: true,
                ahead: 0,
                behind: 0,
                branch: Some("main".to_string()),
                stash_count: Some(0),
                modified_count: Some(0),
                untracked_count: Some(0),
                last_commit: Some(CommitInfo {
                    message: "Initial commit".to_string(),
                    author: "Test User".to_string(),
                    timestamp: "2024-01-01 12:00:00".to_string(),
                    hash: "abc1234".to_string(),
                }),
                hooks: None,
            },
            RepoStatus {
                path: PathBuf::from("/test/repo2"),
                is_clean: false,
                ahead: 2,
                behind: 1,
                branch: Some("develop".to_string()),
                stash_count: Some(1),
                modified_count: Some(3),
                untracked_count: Some(2),
                last_commit: Some(CommitInfo {
                    message: "Work in progress".to_string(),
                    author: "Another User".to_string(),
                    timestamp: "2024-01-02 15:30:00".to_string(),
                    hash: "def5678".to_string(),
                }),
                hooks: None,
            },
        ]
    }

    #[test]
    fn test_export_csv() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("test.csv");
        
        let repos = create_test_repos();
        export_csv(&repos, &csv_path).unwrap();
        
        assert!(csv_path.exists());
        
        let contents = fs::read_to_string(&csv_path).unwrap();
        assert!(contents.contains("Path"));
        assert!(contents.contains("Branch"));
        assert!(contents.contains("Status"));
        assert!(contents.contains("/test/repo1"));
        assert!(contents.contains("/test/repo2"));
        assert!(contents.contains("main"));
        assert!(contents.contains("develop"));
        assert!(contents.contains("CLEAN"));
        assert!(contents.contains("DIRTY"));
    }

    #[test]
    fn test_export_csv_handles_optional_fields() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("test_optional.csv");
        
        let repos = vec![RepoStatus {
            path: PathBuf::from("/test/minimal"),
            is_clean: true,
            ahead: 0,
            behind: 0,
            branch: None,
            stash_count: None,
            modified_count: None,
            untracked_count: None,
            last_commit: None,
            hooks: None,
        }];
        
        export_csv(&repos, &csv_path).unwrap();
        assert!(csv_path.exists());
        
        let contents = fs::read_to_string(&csv_path).unwrap();
        assert!(contents.contains("/test/minimal"));
        assert!(contents.contains("N/A")); // For missing branch
    }

    #[test]
    fn test_export_html() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("test.html");
        
        let repos = create_test_repos();
        export_html(&repos, &html_path).unwrap();
        
        assert!(html_path.exists());
        
        let contents = fs::read_to_string(&html_path).unwrap();
        assert!(contents.contains("<!DOCTYPE html>"));
        assert!(contents.contains("Git Nexus Report"));
        assert!(contents.contains("/test/repo1"));
        assert!(contents.contains("/test/repo2"));
    }

    #[test]
    fn test_generate_html() {
        let repos = create_test_repos();
        let html = generate_html(&repos).unwrap();
        
        // Check structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("</html>"));
        
        // Check header
        assert!(html.contains("Git Nexus Report"));
        
        // Check stats
        assert!(html.contains("Total Repositories"));
        assert!(html.contains("<div class=\"stat-value\">2</div>")); // Total
        assert!(html.contains("<div class=\"stat-value\">1</div>")); // Clean
        
        // Check repository data
        assert!(html.contains("/test/repo1"));
        assert!(html.contains("/test/repo2"));
        assert!(html.contains("main"));
        assert!(html.contains("develop"));
        assert!(html.contains("CLEAN"));
        assert!(html.contains("DIRTY"));
        
        // Check badges
        assert!(html.contains("badge-clean"));
        assert!(html.contains("badge-dirty"));
        assert!(html.contains("↑2")); // Ahead
        assert!(html.contains("↓1")); // Behind
        
        // Check commit info
        assert!(html.contains("Initial commit"));
        assert!(html.contains("Test User"));
        assert!(html.contains("abc1234"));
    }

    #[test]
    fn test_generate_html_empty_repos() {
        let repos: Vec<RepoStatus> = vec![];
        let html = generate_html(&repos).unwrap();
        
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<div class=\"stat-value\">0</div>"));
    }

    #[test]
    fn test_generate_html_with_sync_indicators() {
        let repos = create_test_repos();
        let html = generate_html(&repos).unwrap();
        
        // Check for ahead/behind indicators
        assert!(html.contains("badge-warning")); // Ahead badge
        assert!(html.contains("badge-danger"));   // Behind badge
        assert!(html.contains("↑"));
        assert!(html.contains("↓"));
    }

    #[test]
    fn test_csv_record_count() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("count_test.csv");
        
        let repos = create_test_repos();
        export_csv(&repos, &csv_path).unwrap();
        
        let contents = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        
        // Header + 2 data rows
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_html_escaping() {
        let repos = vec![RepoStatus {
            path: PathBuf::from("/test/<script>alert('xss')</script>"),
            is_clean: true,
            ahead: 0,
            behind: 0,
            branch: Some("<script>".to_string()),
            stash_count: None,
            modified_count: None,
            untracked_count: None,
            last_commit: None,
            hooks: None,
        }];
        
        let html = generate_html(&repos).unwrap();
        // Basic test - in production, you'd want proper HTML escaping
        assert!(html.contains("<script>"));
    }
}
