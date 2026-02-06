//! Health scoring system for git repositories.
//!
//! Calculates a health score (0-100) based on multiple factors:
//! - Clean working directory
//! - Sync status with remote
//! - Commit frequency
//! - Branch status

use crate::RepoStatus;
use colored::*;

/// Health score breakdown
#[derive(Debug, Clone)]
pub struct HealthScore {
    pub total: u8,
    pub cleanliness: u8,
    pub sync_status: u8,
    pub branch_status: u8,
}

impl HealthScore {
    /// Get color-coded display of score
    pub fn color_string(&self) -> ColoredString {
        match self.total {
            90..=100 => format!("{}%", self.total).bright_green().bold(),
            70..=89 => format!("{}%", self.total).green(),
            50..=69 => format!("{}%", self.total).yellow(),
            30..=49 => format!("{}%", self.total).bright_yellow(),
            _ => format!("{}%", self.total).red(),
        }
    }

    /// Get health status label
    pub fn status(&self) -> &str {
        match self.total {
            90..=100 => "Excellent",
            70..=89 => "Good",
            50..=69 => "Fair",
            30..=49 => "Poor",
            _ => "Critical",
        }
    }

    /// Get emoji indicator
    pub fn emoji(&self) -> &str {
        match self.total {
            90..=100 => "💚",
            70..=89 => "💙",
            50..=69 => "💛",
            30..=49 => "🧡",
            _ => "❤️",
        }
    }
}

/// Calculate health score for a repository
pub fn calculate_health_score(repo: &RepoStatus) -> HealthScore {
    // Cleanliness score (40 points max)
    let cleanliness = if repo.is_clean {
        40
    } else {
        // Partial credit based on file counts
        let modified = repo.modified_count.unwrap_or(0);
        let untracked = repo.untracked_count.unwrap_or(0);
        let total_changes = modified + untracked;
        
        match total_changes {
            0 => 40,
            1..=5 => 30,
            6..=15 => 20,
            16..=30 => 10,
            _ => 5,
        }
    };

    // Sync status score (40 points max)
    let sync_status = if repo.ahead == 0 && repo.behind == 0 {
        40
    } else {
        let total_divergence = repo.ahead + repo.behind;
        match total_divergence {
            0 => 40,
            1..=3 => 30,
            4..=10 => 20,
            11..=20 => 10,
            _ => 5,
        }
    };

    // Branch status score (20 points max)
    let branch_status = if let Some(ref branch) = repo.branch {
        if branch.starts_with("detached@") {
            5 // Critical: detached HEAD
        } else if branch.contains("(no commits)") {
            10 // New branch
        } else {
            20 // Normal branch
        }
    } else {
        5 // No branch info
    };

    let total = cleanliness + sync_status + branch_status;

    HealthScore {
        total,
        cleanliness,
        sync_status,
        branch_status,
    }
}

/// Calculate average health score for multiple repositories
pub fn average_health_score(repos: &[RepoStatus]) -> Option<HealthScore> {
    if repos.is_empty() {
        return None;
    }

    let scores: Vec<HealthScore> = repos.iter()
        .map(|r| calculate_health_score(r))
        .collect();

    let avg_total = scores.iter().map(|s| s.total as usize).sum::<usize>() / scores.len();
    let avg_clean = scores.iter().map(|s| s.cleanliness as usize).sum::<usize>() / scores.len();
    let avg_sync = scores.iter().map(|s| s.sync_status as usize).sum::<usize>() / scores.len();
    let avg_branch = scores.iter().map(|s| s.branch_status as usize).sum::<usize>() / scores.len();

    Some(HealthScore {
        total: avg_total as u8,
        cleanliness: avg_clean as u8,
        sync_status: avg_sync as u8,
        branch_status: avg_branch as u8,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_repo(is_clean: bool, ahead: usize, behind: usize, branch: &str) -> RepoStatus {
        RepoStatus {
            path: PathBuf::from("/test"),
            is_clean,
            ahead,
            behind,
            branch: Some(branch.to_string()),
            stash_count: Some(0),
            modified_count: Some(if is_clean { 0 } else { 5 }),
            untracked_count: Some(0),
            last_commit: None,
            hooks: None,
        }
    }

    #[test]
    fn test_perfect_health() {
        let repo = create_test_repo(true, 0, 0, "main");
        let score = calculate_health_score(&repo);
        assert_eq!(score.total, 100);
        assert_eq!(score.status(), "Excellent");
    }

    #[test]
    fn test_dirty_repo() {
        let repo = create_test_repo(false, 0, 0, "main");
        let score = calculate_health_score(&repo);
        assert!(score.total < 100);
        assert!(score.cleanliness < 40);
    }

    #[test]
    fn test_ahead_behind() {
        let repo = create_test_repo(true, 5, 3, "main");
        let score = calculate_health_score(&repo);
        assert!(score.sync_status < 40);
    }

    #[test]
    fn test_detached_head() {
        let repo = create_test_repo(true, 0, 0, "detached@abc123");
        let score = calculate_health_score(&repo);
        assert_eq!(score.branch_status, 5);
        assert!(score.total < 100);
    }

    #[test]
    fn test_average_score() {
        let repos = vec![
            create_test_repo(true, 0, 0, "main"),
            create_test_repo(false, 2, 0, "develop"),
        ];
        let avg = average_health_score(&repos).unwrap();
        assert!(avg.total > 50);
        assert!(avg.total < 100);
    }

    #[test]
    fn test_color_string() {
        let score = HealthScore {
            total: 95,
            cleanliness: 40,
            sync_status: 40,
            branch_status: 15,
        };
        assert!(score.color_string().to_string().contains("95"));
    }
}
