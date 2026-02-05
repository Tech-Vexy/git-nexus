use anyhow::Result;
use chrono::Local;
use csv::Writer;
use std::fs::File;
use std::path::PathBuf;

use crate::RepoStatus;

pub fn export_csv(repos: &[RepoStatus], path: &PathBuf) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;
    
    wtr.write_record(&[
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

pub fn export_html(repos: &[RepoStatus], path: &PathBuf) -> Result<()> {
    let html = generate_html(repos)?;
    std::fs::write(path, html)?;
    Ok(())
}

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
