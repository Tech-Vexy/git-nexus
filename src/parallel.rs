//! Parallel batch operations for git-nexus.
//!
//! Enables concurrent execution of git operations across multiple repositories.

use crate::resolution::{Action, ActionResult, apply_action};
use crate::RepoStatus;
use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

/// Result of a batch operation
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub repo_path: String,
    pub action_result: ActionResult,
}

/// Execute an action on multiple repositories in parallel
pub fn execute_parallel(
    repos: &[RepoStatus],
    action: Action,
    dry_run: bool,
) -> Vec<BatchResult> {
    let multi_progress = Arc::new(MultiProgress::new());
    let results = Arc::new(Mutex::new(Vec::new()));
    
    // Create progress bars for each repository
    let progress_bars: Vec<_> = repos
        .iter()
        .map(|repo| {
            let pb = multi_progress.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap(),
            );
            pb.set_message(format!("Processing: {}", repo.path.display()));
            pb
        })
        .collect();
    
    // Execute actions in parallel
    repos
        .par_iter()
        .zip(progress_bars.par_iter())
        .for_each(|(repo, pb)| {
            pb.set_message(format!("Applying action: {}", repo.path.display()));
            
            let result = if dry_run {
                ActionResult {
                    success: true,
                    message: format!("[DRY RUN] Would apply: {}", action.description()),
                    details: None,
                }
            } else {
                match apply_action(&repo.path, &action, false) {
                    Ok(result) => result,
                    Err(e) => ActionResult {
                        success: false,
                        message: format!("Failed: {}", e),
                        details: None,
                    },
                }
            };
            
            let batch_result = BatchResult {
                repo_path: repo.path.to_string_lossy().to_string(),
                action_result: result,
            };
            
            pb.finish_with_message(format!(
                "{}: {}",
                repo.path.display(),
                if batch_result.action_result.success {
                    "✓"
                } else {
                    "✗"
                }
            ));
            
            results.lock().unwrap().push(batch_result);
        });
    
    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

/// Execute different actions on multiple repositories in parallel
pub fn execute_parallel_actions(
    operations: Vec<(RepoStatus, Action)>,
    dry_run: bool,
) -> Vec<BatchResult> {
    let multi_progress = Arc::new(MultiProgress::new());
    let results = Arc::new(Mutex::new(Vec::new()));
    
    // Create progress bars for each operation
    let progress_bars: Vec<_> = operations
        .iter()
        .map(|(repo, _)| {
            let pb = multi_progress.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap(),
            );
            pb.set_message(format!("Processing: {}", repo.path.display()));
            pb
        })
        .collect();
    
    // Execute operations in parallel
    operations
        .par_iter()
        .zip(progress_bars.par_iter())
        .for_each(|((repo, action), pb)| {
            pb.set_message(format!("Applying {}: {}", action.description(), repo.path.display()));
            
            let result = if dry_run {
                ActionResult {
                    success: true,
                    message: format!("[DRY RUN] Would apply: {}", action.description()),
                    details: None,
                }
            } else {
                match apply_action(&repo.path, action, false) {
                    Ok(result) => result,
                    Err(e) => ActionResult {
                        success: false,
                        message: format!("Failed: {}", e),
                        details: None,
                    },
                }
            };
            
            let batch_result = BatchResult {
                repo_path: repo.path.to_string_lossy().to_string(),
                action_result: result,
            };
            
            pb.finish_with_message(format!(
                "{}: {}",
                repo.path.display(),
                if batch_result.action_result.success {
                    "✓"
                } else {
                    "✗"
                }
            ));
            
            results.lock().unwrap().push(batch_result);
        });
    
    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

/// Display summary of batch operation results
pub fn display_batch_summary(results: &[BatchResult]) {
    let successful = results.iter().filter(|r| r.action_result.success).count();
    let failed = results.len() - successful;
    
    println!("\n{}", "═".repeat(60));
    println!("Batch Operation Summary");
    println!("{}", "═".repeat(60));
    println!("  ✓ Successful: {}", successful);
    if failed > 0 {
        println!("  ✗ Failed: {}", failed);
    }
    println!("{}", "═".repeat(60));
    
    // Show failed operations
    if failed > 0 {
        println!("\nFailed operations:");
        for result in results.iter().filter(|r| !r.action_result.success) {
            println!("  ✗ {}: {}", result.repo_path, result.action_result.message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_batch_result_creation() {
        let result = BatchResult {
            repo_path: "/test/repo".to_string(),
            action_result: ActionResult {
                success: true,
                message: "Success".to_string(),
                details: None,
            },
        };
        
        assert_eq!(result.repo_path, "/test/repo");
        assert!(result.action_result.success);
    }

    #[test]
    fn test_display_batch_summary() {
        let results = vec![
            BatchResult {
                repo_path: "/repo1".to_string(),
                action_result: ActionResult {
                    success: true,
                    message: "OK".to_string(),
                    details: None,
                },
            },
            BatchResult {
                repo_path: "/repo2".to_string(),
                action_result: ActionResult {
                    success: false,
                    message: "Failed".to_string(),
                    details: None,
                },
            },
        ];
        
        // Just test that it doesn't panic
        display_batch_summary(&results);
    }
}
