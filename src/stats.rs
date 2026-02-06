//! Statistics and analytics for repositories.
//!
//! Provides detailed metrics like lines of code, repository age, commit counts, etc.

use crate::RepoStatus;
use colored::*;
use git2::Repository;
use walkdir::WalkDir;

/// Statistics for a single repository
#[derive(Debug, Clone)]
pub struct RepoStats {
    pub path: String,
    pub lines_of_code: usize,
    pub file_count: usize,
    pub commit_count: usize,
    pub contributor_count: usize,
    pub age_days: i64,
    pub languages: Vec<(String, usize)>, // (extension, line count)
}

/// Calculate statistics for a repository
pub fn calculate_stats(repo_status: &RepoStatus) -> Option<RepoStats> {
    let repo = Repository::open(&repo_status.path).ok()?;
    
    // Calculate lines of code
    let (lines_of_code, file_count, languages) = count_lines_of_code(&repo_status.path);
    
    // Count commits
    let commit_count = count_commits(&repo);
    
    // Count contributors
    let contributor_count = count_contributors(&repo);
    
    // Calculate repository age
    let age_days = calculate_age_days(&repo);
    
    Some(RepoStats {
        path: repo_status.path.to_string_lossy().to_string(),
        lines_of_code,
        file_count,
        commit_count,
        contributor_count,
        age_days,
        languages,
    })
}

/// Count lines of code in the repository
fn count_lines_of_code(path: &std::path::Path) -> (usize, usize, Vec<(String, usize)>) {
    let mut total_lines = 0;
    let mut file_count = 0;
    let mut language_stats: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    // Common code file extensions
    let code_extensions = vec![
        "rs", "py", "js", "ts", "java", "c", "cpp", "h", "hpp", "go", "rb", "php",
        "cs", "swift", "kt", "scala", "r", "m", "mm", "dart", "lua", "pl", "sh",
        "jsx", "tsx", "vue", "svelte", "html", "css", "scss", "sass", "less",
    ];
    
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip common non-source directories
            let file_name = e.file_name().to_string_lossy();
            let is_root = e.path() == path;
            is_root || (
                !file_name.starts_with('.') &&
                file_name != "target" &&
                file_name != "node_modules" &&
                file_name != "build" &&
                file_name != "dist" &&
                file_name != "vendor"
            )
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy().to_string();
                if code_extensions.contains(&ext_str.as_str()) {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let line_count = content.lines().count();
                        total_lines += line_count;
                        file_count += 1;
                        *language_stats.entry(ext_str).or_insert(0) += line_count;
                    }
                }
            }
        }
    }
    
    // Sort languages by line count
    let mut languages: Vec<(String, usize)> = language_stats.into_iter().collect();
    languages.sort_by(|a, b| b.1.cmp(&a.1));
    
    (total_lines, file_count, languages)
}

/// Count total commits in the repository
fn count_commits(repo: &Repository) -> usize {
    let mut revwalk = match repo.revwalk() {
        Ok(rw) => rw,
        Err(_) => return 0,
    };
    
    if revwalk.push_head().is_err() {
        return 0;
    }
    
    revwalk.count()
}

/// Count unique contributors
fn count_contributors(repo: &Repository) -> usize {
    let mut contributors = std::collections::HashSet::new();
    
    let mut revwalk = match repo.revwalk() {
        Ok(rw) => rw,
        Err(_) => return 0,
    };
    
    if revwalk.push_head().is_err() {
        return 0;
    }
    
    for oid in revwalk {
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                let author = commit.author();
                if let Some(email) = author.email() {
                    contributors.insert(email.to_string());
                }
            }
        }
    }
    
    contributors.len()
}

/// Calculate repository age in days
fn calculate_age_days(repo: &Repository) -> i64 {
    let mut revwalk = match repo.revwalk() {
        Ok(rw) => rw,
        Err(_) => return 0,
    };
    
    if revwalk.push_head().is_err() {
        return 0;
    }
    
    // Find the first (oldest) commit
    let mut oldest_time = None;
    for oid in revwalk {
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                oldest_time = Some(commit.time().seconds());
            }
        }
    }
    
    if let Some(oldest) = oldest_time {
        let now = chrono::Utc::now().timestamp();
        (now - oldest) / 86400 // Convert seconds to days
    } else {
        0
    }
}

/// Display statistics dashboard for all repositories
pub fn display_stats_dashboard(repos: &[RepoStatus]) {
    println!("\n{}", "📊 Repository Statistics Dashboard".bright_cyan().bold());
    println!("{}", "═".repeat(80).bright_black());
    
    let mut total_loc = 0;
    let mut total_files = 0;
    let mut total_commits = 0;
    let mut all_languages: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for repo_status in repos {
        if let Some(stats) = calculate_stats(repo_status) {
            total_loc += stats.lines_of_code;
            total_files += stats.file_count;
            total_commits += stats.commit_count;
            
            for (lang, lines) in &stats.languages {
                *all_languages.entry(lang.clone()).or_insert(0) += lines;
            }
            
            // Display individual repo stats
            println!("\n{} {}", "📁".bright_yellow(), stats.path.bright_white().bold());
            println!("  {} {} lines across {} files", 
                "📝".bright_blue(),
                format_number(stats.lines_of_code),
                stats.file_count
            );
            println!("  {} {} commits by {} contributors",
                "📌".bright_magenta(),
                format_number(stats.commit_count),
                stats.contributor_count
            );
            println!("  {} {} days old",
                "🗓️ ".bright_green(),
                format_number(stats.age_days as usize)
            );
            
            // Show top 3 languages
            if !stats.languages.is_empty() {
                print!("  {} ", "🔤".bright_cyan());
                let top_langs: Vec<String> = stats.languages.iter()
                    .take(3)
                    .map(|(lang, lines)| format!("{} ({})", lang, format_number(*lines)))
                    .collect();
                println!("{}", top_langs.join(", "));
            }
        }
    }
    
    // Display summary
    println!("\n{}", "═".repeat(80).bright_black());
    println!("{}", "📊 Workspace Summary".bright_cyan().bold());
    println!("  {} {} total lines of code across {} files",
        "📝".bright_blue(),
        format_number(total_loc),
        format_number(total_files)
    );
    println!("  {} {} total commits",
        "📌".bright_magenta(),
        format_number(total_commits)
    );
    
    // Top languages across all repos
    if !all_languages.is_empty() {
        let mut lang_vec: Vec<(String, usize)> = all_languages.into_iter().collect();
        lang_vec.sort_by(|a, b| b.1.cmp(&a.1));
        
        println!("  {} Top languages:", "🔤".bright_cyan());
        for (i, (lang, lines)) in lang_vec.iter().take(5).enumerate() {
            let percentage = (*lines as f64 / total_loc as f64) * 100.0;
            println!("    {}. {} - {} lines ({:.1}%)",
                i + 1,
                lang,
                format_number(*lines),
                percentage
            );
        }
    }
}

/// Format large numbers with commas
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;
    
    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }
    
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
        assert_eq!(format_number(42), "42");
    }
}
