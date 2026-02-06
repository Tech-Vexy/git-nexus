//! Suggestions engine for providing contextual recommendations.
//!
//! This module analyzes repository status and generates helpful suggestions
//! for resolving common issues.

use crate::resolution::Action;
use crate::RepoStatus;
use colored::*;

/// Priority level for suggestions
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// A suggestion for fixing a repository issue
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API
pub struct Suggestion {
    pub title: String,
    pub description: String,
    pub action: Action,
    pub priority: Priority,
    pub reason: String,
}

impl Suggestion {
    pub fn display(&self) {
        let priority_color = match self.priority {
            Priority::Critical => "●".red().bold(),
            Priority::High => "●".yellow().bold(),
            Priority::Medium => "●".blue(),
            Priority::Low => "●".bright_black(),
        };
        
        println!("  {} {}", priority_color, self.title.bold());
        println!("    {}", self.description.bright_black());
        println!("    {} {}", "→".cyan(), self.action.git_command().cyan());
    }
}

/// Generate suggestions for a repository based on its status
pub fn generate_suggestions(status: &RepoStatus) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // Check for dirty repository
    if !status.is_clean {
        suggestions.extend(suggest_dirty_fixes(status));
    }

    // Check for ahead commits
    if status.ahead > 0 {
        suggestions.extend(suggest_ahead_fixes(status));
    }

    // Check for behind commits
    if status.behind > 0 {
        suggestions.extend(suggest_behind_fixes(status));
    }

    // Check for detached HEAD
    if let Some(ref branch) = status.branch {
        if branch.starts_with("detached@") {
            suggestions.push(suggest_detached_head_fix(branch));
        }
    }

    // Check for stashes
    if let Some(stash_count) = status.stash_count {
        if stash_count > 0 {
            suggestions.push(suggest_stash_action(stash_count));
        }
    }

    // Sort by priority
    suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
    
    suggestions
}

/// Generate suggestions for dirty repository
fn suggest_dirty_fixes(status: &RepoStatus) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    
    let modified = status.modified_count.unwrap_or(0);
    let untracked = status.untracked_count.unwrap_or(0);
    
    if modified > 0 || untracked > 0 {
        // Suggest committing changes
        suggestions.push(Suggestion {
            title: "Commit your changes".to_string(),
            description: format!(
                "You have {} modified and {} untracked file(s)",
                modified, untracked
            ),
            action: Action::CommitWip {
                message: "WIP: Auto-commit by git-nexus".to_string(),
            },
            priority: Priority::High,
            reason: "Uncommitted changes can be lost".to_string(),
        });

        // Suggest stashing
        suggestions.push(Suggestion {
            title: "Stash your changes".to_string(),
            description: "Save changes for later without committing".to_string(),
            action: Action::Stash {
                message: Some("git-nexus auto-stash".to_string()),
            },
            priority: Priority::Medium,
            reason: "Clean working directory temporarily".to_string(),
        });

        // Suggest discarding (low priority, destructive)
        suggestions.push(Suggestion {
            title: "Discard changes (destructive)".to_string(),
            description: "⚠️  Permanently remove all uncommitted changes".to_string(),
            action: Action::DiscardChanges,
            priority: Priority::Low,
            reason: "Use only if changes are not needed".to_string(),
        });
    }

    suggestions
}

/// Generate suggestions for repository ahead of remote
fn suggest_ahead_fixes(status: &RepoStatus) -> Vec<Suggestion> {
    vec![Suggestion {
        title: format!("Push {} commit(s) to remote", status.ahead),
        description: "Your local branch has unpushed commits".to_string(),
        action: Action::Push,
        priority: Priority::Medium,
        reason: "Share your work with the team".to_string(),
    }]
}

/// Generate suggestions for repository behind remote
fn suggest_behind_fixes(status: &RepoStatus) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    if status.is_clean {
        // Safe to pull if working directory is clean
        suggestions.push(Suggestion {
            title: format!("Pull {} commit(s) from remote", status.behind),
            description: "Your local branch is behind the remote".to_string(),
            action: Action::Pull,
            priority: Priority::High,
            reason: "Stay up to date with team changes".to_string(),
        });
    } else {
        // Need to stash first if dirty
        suggestions.push(Suggestion {
            title: "Stash changes before pulling".to_string(),
            description: format!(
                "You're {} commit(s) behind but have uncommitted changes",
                status.behind
            ),
            action: Action::Stash {
                message: Some("Before pull".to_string()),
            },
            priority: Priority::High,
            reason: "Avoid merge conflicts".to_string(),
        });
    }

    // Suggest sync if both ahead and behind
    if status.ahead > 0 && status.behind > 0 {
        suggestions.push(Suggestion {
            title: "Sync with remote".to_string(),
            description: format!(
                "Diverged: {} ahead, {} behind",
                status.ahead, status.behind
            ),
            action: Action::Sync,
            priority: Priority::Critical,
            reason: "Branches have diverged".to_string(),
        });
    }

    suggestions
}

/// Generate suggestion for detached HEAD state
fn suggest_detached_head_fix(branch: &str) -> Suggestion {
    let hash = branch
        .strip_prefix("detached@")
        .unwrap_or("unknown");
    
    Suggestion {
        title: "Create branch from detached HEAD".to_string(),
        description: format!("Currently at commit {}", hash),
        action: Action::CreateBranch {
            name: format!("from-detached-{}", hash),
        },
        priority: Priority::Critical,
        reason: "Commits may be lost when switching branches".to_string(),
    }
}

/// Generate suggestion for stashes
fn suggest_stash_action(count: usize) -> Suggestion {
    Suggestion {
        title: format!("Pop stash (you have {})", count),
        description: "Restore previously stashed changes".to_string(),
        action: Action::StashPop,
        priority: Priority::Low,
        reason: "Don't forget about stashed work".to_string(),
    }
}

/// Generate a summary of all issues
pub fn summarize_issues(repos: &[RepoStatus]) -> IssueSummary {
    let mut summary = IssueSummary::default();
    
    for repo in repos {
        if !repo.is_clean {
            summary.dirty_repos += 1;
        }
        if repo.ahead > 0 {
            summary.ahead_repos += 1;
            summary.total_unpushed += repo.ahead;
        }
        if repo.behind > 0 {
            summary.behind_repos += 1;
            summary.total_unpulled += repo.behind;
        }
        if let Some(ref branch) = repo.branch {
            if branch.starts_with("detached@") {
                summary.detached_heads += 1;
            }
        }
        if let Some(count) = repo.stash_count {
            if count > 0 {
                summary.repos_with_stashes += 1;
            }
        }
    }
    
    summary.total_repos = repos.len();
    summary.clean_repos = repos.iter().filter(|r| r.is_clean).count();
    
    summary
}

/// Summary of issues across all repositories
#[derive(Debug, Default)]
#[allow(dead_code)] // Public API
pub struct IssueSummary {
    pub total_repos: usize,
    pub clean_repos: usize,
    pub dirty_repos: usize,
    pub ahead_repos: usize,
    pub behind_repos: usize,
    pub detached_heads: usize,
    pub repos_with_stashes: usize,
    pub total_unpushed: usize,
    pub total_unpulled: usize,
}

impl IssueSummary {
    pub fn display(&self) {
        println!("\n{}", "═".repeat(60).bright_black());
        println!("{}", "  WORKSPACE SUMMARY".bold().cyan());
        println!("{}", "═".repeat(60).bright_black());
        
        println!("  {} {} total repositories", "📊".cyan(), self.total_repos);
        
        if self.clean_repos > 0 {
            println!("  {} {} clean", "✓".green(), self.clean_repos);
        }
        
        if self.dirty_repos > 0 {
            println!("  {} {} with uncommitted changes", "●".yellow(), self.dirty_repos);
        }
        
        if self.ahead_repos > 0 {
            println!(
                "  {} {} ahead of remote ({} commits)",
                "↑".yellow(),
                self.ahead_repos,
                self.total_unpushed
            );
        }
        
        if self.behind_repos > 0 {
            println!(
                "  {} {} behind remote ({} commits)",
                "↓".red(),
                self.behind_repos,
                self.total_unpulled
            );
        }
        
        if self.detached_heads > 0 {
            println!("  {} {} detached HEAD state", "⚠".bright_red(), self.detached_heads);
        }
        
        if self.repos_with_stashes > 0 {
            println!("  {} {} with stashed changes", "📦".magenta(), self.repos_with_stashes);
        }
        
        println!("{}", "═".repeat(60).bright_black());
    }

    pub fn has_issues(&self) -> bool {
        self.dirty_repos > 0
            || self.ahead_repos > 0
            || self.behind_repos > 0
            || self.detached_heads > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_status(is_clean: bool, ahead: usize, behind: usize) -> RepoStatus {
        RepoStatus {
            path: PathBuf::from("/test"),
            is_clean,
            ahead,
            behind,
            branch: Some("main".to_string()),
            stash_count: Some(0),
            modified_count: Some(if is_clean { 0 } else { 1 }),
            untracked_count: Some(0),
            last_commit: None,
            hooks: None,
        }
    }

    #[test]
    fn test_generate_suggestions_dirty() {
        let status = create_test_status(false, 0, 0);
        let suggestions = generate_suggestions(&status);
        
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.title.contains("Commit")));
        assert!(suggestions.iter().any(|s| s.title.contains("Stash")));
    }

    #[test]
    fn test_generate_suggestions_ahead() {
        let status = create_test_status(true, 2, 0);
        let suggestions = generate_suggestions(&status);
        
        assert!(suggestions.iter().any(|s| s.title.contains("Push")));
    }

    #[test]
    fn test_generate_suggestions_behind() {
        let status = create_test_status(true, 0, 3);
        let suggestions = generate_suggestions(&status);
        
        assert!(suggestions.iter().any(|s| s.title.contains("Pull")));
    }

    #[test]
    fn test_generate_suggestions_detached() {
        let mut status = create_test_status(true, 0, 0);
        status.branch = Some("detached@abc1234".to_string());
        
        let suggestions = generate_suggestions(&status);
        assert!(suggestions.iter().any(|s| s.title.contains("detached")));
    }

    #[test]
    fn test_summarize_issues() {
        let repos = vec![
            create_test_status(true, 0, 0),
            create_test_status(false, 2, 0),
            create_test_status(true, 0, 3),
        ];
        
        let summary = summarize_issues(&repos);
        assert_eq!(summary.total_repos, 3);
        assert_eq!(summary.clean_repos, 2);
        assert_eq!(summary.dirty_repos, 1);
        assert_eq!(summary.ahead_repos, 1);
        assert_eq!(summary.behind_repos, 1);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }
}
