//! Resolution module for fixing common git repository issues.
//!
//! This module provides safe operations to fix common problems in git repositories,
//! including staging changes, creating commits, stashing, pulling, pushing, and more.

use anyhow::{Context, Result};
use colored::*;
use git2::{Repository, Signature};
use std::path::Path;

/// Actions that can be performed to resolve repository issues
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Public API - variants may be used by consumers
pub enum Action {
    /// Stage all changes (git add .)
    StageAll,
    /// Create a WIP commit with all changes
    CommitWip { message: String },
    /// Stash all uncommitted changes
    Stash { message: Option<String> },
    /// Pull latest changes from remote
    Pull,
    /// Push local commits to remote
    Push,
    /// Create a branch from detached HEAD
    CreateBranch { name: String },
    /// Discard all uncommitted changes (destructive!)
    DiscardChanges,
    /// Pop the most recent stash
    StashPop,
    /// Sync with remote (pull + push)
    Sync,
}

impl Action {
    /// Returns true if this action is destructive (cannot be easily undone)
    pub fn is_destructive(&self) -> bool {
        matches!(self, Action::DiscardChanges)
    }

    /// Returns a human-readable description of the action
    pub fn description(&self) -> String {
        match self {
            Action::StageAll => "Stage all changes".to_string(),
            Action::CommitWip { message } => format!("Create commit: {}", message),
            Action::Stash { message } => {
                if let Some(msg) = message {
                    format!("Stash changes: {}", msg)
                } else {
                    "Stash changes".to_string()
                }
            }
            Action::Pull => "Pull latest changes from remote".to_string(),
            Action::Push => "Push local commits to remote".to_string(),
            Action::CreateBranch { name } => format!("Create branch: {}", name),
            Action::DiscardChanges => "Discard all uncommitted changes (DESTRUCTIVE)".to_string(),
            Action::StashPop => "Pop most recent stash".to_string(),
            Action::Sync => "Sync with remote (pull + push)".to_string(),
        }
    }

    /// Returns the git command equivalent for documentation
    pub fn git_command(&self) -> String {
        match self {
            Action::StageAll => "git add .".to_string(),
            Action::CommitWip { message } => format!("git commit -m \"{}\"", message),
            Action::Stash { message } => {
                if let Some(msg) = message {
                    format!("git stash push -m \"{}\"", msg)
                } else {
                    "git stash".to_string()
                }
            }
            Action::Pull => "git pull".to_string(),
            Action::Push => "git push".to_string(),
            Action::CreateBranch { name } => format!("git checkout -b {}", name),
            Action::DiscardChanges => "git reset --hard && git clean -fd".to_string(),
            Action::StashPop => "git stash pop".to_string(),
            Action::Sync => "git pull && git push".to_string(),
        }
    }
}

/// Result of applying an action to a repository
#[derive(Debug)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}

impl ActionResult {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            details: None,
        }
    }

    pub fn success_with_details(message: String, details: String) -> Self {
        Self {
            success: true,
            message,
            details: Some(details),
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            details: None,
        }
    }

    pub fn display(&self) {
        if self.success {
            println!("  {} {}", "✓".green().bold(), self.message.green());
        } else {
            println!("  {} {}", "✗".red().bold(), self.message.red());
        }
        if let Some(ref details) = self.details {
            println!("    {}", details.bright_black());
        }
    }
}

/// Apply an action to a repository
pub fn apply_action(repo_path: &Path, action: &Action, dry_run: bool) -> Result<ActionResult> {
    let mut repo = Repository::open(repo_path)
        .context(format!("Failed to open repository at {:?}", repo_path))?;

    if dry_run {
        return Ok(ActionResult::success_with_details(
            format!("Would execute: {}", action.description()),
            format!("Command: {}", action.git_command()),
        ));
    }

    match action {
        Action::StageAll => stage_all(&repo),
        Action::CommitWip { message } => commit_changes(&repo, message),
        Action::Stash { message } => stash_changes(&mut repo, message.as_deref()),
        Action::Pull => pull_changes(&repo),
        Action::Push => push_changes(&repo),
        Action::CreateBranch { name } => create_branch(&repo, name),
        Action::DiscardChanges => discard_changes(&repo),
        Action::StashPop => stash_pop(&mut repo),
        Action::Sync => sync_with_remote(&mut repo),
    }
}

/// Stage all changes in the repository
fn stage_all(repo: &Repository) -> Result<ActionResult> {
    let mut index = repo.index()?;
    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    
    Ok(ActionResult::success(
        "Staged all changes".to_string()
    ))
}

/// Create a commit with the given message
fn commit_changes(repo: &Repository, message: &str) -> Result<ActionResult> {
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    
    let signature = repo.signature()
        .or_else(|_| Signature::now("git-nexus", "git-nexus@localhost"))
        .context("Failed to create signature")?;
    
    let parent_commit = repo.head()?.peel_to_commit()?;
    
    let oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;
    
    Ok(ActionResult::success_with_details(
        format!("Created commit: {}", message),
        format!("Commit: {}", &oid.to_string()[..7]),
    ))
}

/// Stash all uncommitted changes
fn stash_changes(repo: &mut Repository, message: Option<&str>) -> Result<ActionResult> {
    let signature = repo.signature()
        .or_else(|_| Signature::now("git-nexus", "git-nexus@localhost"))?;
    
    let stash_id = repo.stash_save(
        &signature,
        message.unwrap_or("git-nexus auto-stash"),
        Some(git2::StashFlags::INCLUDE_UNTRACKED),
    )?;
    
    Ok(ActionResult::success_with_details(
        "Stashed changes".to_string(),
        format!("Stash: {}", &stash_id.to_string()[..7]),
    ))
}

/// Pull changes from remote
fn pull_changes(repo: &Repository) -> Result<ActionResult> {
    // Get current branch
    let head = repo.head()?;
    
    if !head.is_branch() {
        return Ok(ActionResult::failure(
            "Cannot pull: not on a branch".to_string()
        ));
    }
    
    let branch = repo.find_branch(head.shorthand().unwrap(), git2::BranchType::Local)?;
    let upstream = branch.upstream()
        .context("No upstream branch configured")?;
    
    // Fetch
    let remote_name = upstream.name()?.unwrap();
    let parts: Vec<&str> = remote_name.splitn(2, '/').collect();
    let remote_name = parts[0];
    
    let mut remote = repo.find_remote(remote_name)?;
    remote.fetch(&[head.shorthand().unwrap()], None, None)?;
    
    // Fast-forward merge
    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    
    let analysis = repo.merge_analysis(&[&fetch_commit])?;
    
    if analysis.0.is_up_to_date() {
        return Ok(ActionResult::success("Already up to date".to_string()));
    }
    
    if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", head.shorthand().unwrap());
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "fast-forward")?;
        repo.set_head(&refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        
        Ok(ActionResult::success("Pulled and fast-forwarded".to_string()))
    } else {
        Ok(ActionResult::failure(
            "Cannot pull: merge required (not implemented)".to_string()
        ))
    }
}

/// Push changes to remote
fn push_changes(repo: &Repository) -> Result<ActionResult> {
    let head = repo.head()?;
    
    if !head.is_branch() {
        return Ok(ActionResult::failure(
            "Cannot push: not on a branch".to_string()
        ));
    }
    
    let branch = repo.find_branch(head.shorthand().unwrap(), git2::BranchType::Local)?;
    let upstream = branch.upstream();
    
    if upstream.is_err() {
        return Ok(ActionResult::failure(
            "No upstream branch configured".to_string()
        ));
    }
    
    // Note: Actual push requires authentication, which is complex
    // For now, return a message indicating the user should push manually
    Ok(ActionResult::failure(
        "Push requires authentication - please use 'git push' manually".to_string()
    ))
}

/// Create a new branch from current HEAD
fn create_branch(repo: &Repository, name: &str) -> Result<ActionResult> {
    let head = repo.head()?;
    let target = head.peel_to_commit()?;
    
    repo.branch(name, &target, false)?;
    
    // Checkout the new branch
    repo.set_head(&format!("refs/heads/{}", name))?;
    
    Ok(ActionResult::success(format!("Created and switched to branch '{}'", name)))
}

/// Discard all uncommitted changes (DESTRUCTIVE)
fn discard_changes(repo: &Repository) -> Result<ActionResult> {
    // Reset to HEAD
    let head = repo.head()?;
    let target = head.peel_to_commit()?;
    
    repo.reset(
        target.as_object(),
        git2::ResetType::Hard,
        None,
    )?;
    
    // Clean untracked files
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            .force()
            .remove_untracked(true)
    ))?;
    
    Ok(ActionResult::success_with_details(
        "Discarded all changes".to_string(),
        "⚠️  This action cannot be undone!".to_string(),
    ))
}

/// Pop the most recent stash
fn stash_pop(repo: &mut Repository) -> Result<ActionResult> {
    repo.stash_pop(0, None)?;
    
    Ok(ActionResult::success("Popped stash".to_string()))
}

/// Sync with remote (pull then push)
fn sync_with_remote(repo: &mut Repository) -> Result<ActionResult> {
    // Try to pull first
    let pull_result = pull_changes(repo)?;
    
    if !pull_result.success {
        return Ok(pull_result);
    }
    
    // Then try to push
    let push_result = push_changes(repo)?;
    
    if pull_result.message.contains("up to date") && push_result.success {
        Ok(ActionResult::success("Synced with remote".to_string()))
    } else {
        Ok(ActionResult::success_with_details(
            "Pulled changes".to_string(),
            "Push requires manual authentication".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_is_destructive() {
        assert!(Action::DiscardChanges.is_destructive());
        assert!(!Action::StageAll.is_destructive());
        assert!(!Action::Stash { message: None }.is_destructive());
    }

    #[test]
    fn test_action_description() {
        let action = Action::CommitWip {
            message: "WIP".to_string(),
        };
        assert_eq!(action.description(), "Create commit: WIP");

        let action = Action::DiscardChanges;
        assert!(action.description().contains("DESTRUCTIVE"));
    }

    #[test]
    fn test_action_git_command() {
        assert_eq!(Action::StageAll.git_command(), "git add .");
        assert_eq!(
            Action::CommitWip {
                message: "test".to_string()
            }
            .git_command(),
            "git commit -m \"test\""
        );
    }

    #[test]
    fn test_action_result() {
        let result = ActionResult::success("Test success".to_string());
        assert!(result.success);
        assert_eq!(result.message, "Test success");

        let result = ActionResult::failure("Test failure".to_string());
        assert!(!result.success);
    }
}
