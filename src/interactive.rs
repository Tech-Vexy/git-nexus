//! Interactive CLI mode for fixing repositories.
//!
//! This module provides user-friendly interactive prompts for selecting
//! repositories and applying fixes.

use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};

use crate::resolution::{ apply_action, Action};
use crate::suggestions::generate_suggestions;
use crate::RepoStatus;

/// Interactive fix mode - let user select repos and actions
pub fn interactive_fix_mode(repos: &[RepoStatus]) -> Result<()> {
    if repos.is_empty() {
        println!("{}", "No repositories found.".yellow());
        return Ok(());
    }

    // Filter repos with issues
    let repos_with_issues: Vec<&RepoStatus> = repos
        .iter()
        .filter(|r| !r.is_clean || r.ahead > 0 || r.behind > 0)
        .collect();

    if repos_with_issues.is_empty() {
        println!("{}", "✓ All repositories are clean and up to date!".green());
        return Ok(());
    }

    println!("\n{}", "╔═══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║          GIT-NEXUS INTERACTIVE FIX MODE              ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════╝".cyan());
    
    println!(
        "\nFound {} {} with issues.\n",
        repos_with_issues.len(),
        if repos_with_issues.len() == 1 {
            "repository"
        } else {
            "repositories"
        }
    );

    // Show repository list with issues
    for (idx, repo) in repos_with_issues.iter().enumerate() {
        let status_icon = if repo.is_clean { "●" } else { "●" };
        let status_color = if repo.is_clean {
            status_icon.blue()
        } else {
            status_icon.yellow()
        };

        let indicators = format_indicators(repo);
        
        println!(
            "  {}. {} {} {}",
            idx + 1,
            status_color,
            repo.path.display().to_string().bright_white(),
            indicators
        );
    }

    println!();

    // Ask what to do
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .item("Fix a single repository")
        .item("Fix multiple repositories")
        .item("Show suggestions for all")
        .item("Exit")
        .default(0)
        .interact()?;

    match selection {
        0 => fix_single_repository(&repos_with_issues),
        1 => fix_multiple_repositories(&repos_with_issues),
        2 => show_all_suggestions(&repos_with_issues),
        3 => {
            println!("Goodbye!");
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Fix a single repository interactively
fn fix_single_repository(repos: &[&RepoStatus]) -> Result<()> {
    let items: Vec<String> = repos
        .iter()
        .map(|r| {
            format!(
                "{} {}",
                r.path.display(),
                format_indicators(r).bright_black()
            )
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select repository to fix")
        .items(&items)
        .interact()?;

    let repo = repos[selection];
    fix_repository_interactive(repo, false)?;

    Ok(())
}

/// Fix multiple repositories interactively
fn fix_multiple_repositories(repos: &[&RepoStatus]) -> Result<()> {
    let items: Vec<String> = repos
        .iter()
        .map(|r| {
            format!(
                "{} {}",
                r.path.display(),
                format_indicators(r).bright_black()
            )
        })
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select repositories to fix (use Space to select, Enter to confirm)")
        .items(&items)
        .interact()?;

    if selections.is_empty() {
        println!("No repositories selected.");
        return Ok(());
    }

    println!("\n{} repositories selected.\n", selections.len());

    // Ask for batch action or individual
    let mode = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("How would you like to fix them?")
        .item("Apply same action to all")
        .item("Fix each one individually")
        .default(0)
        .interact()?;

    match mode {
        0 => batch_fix_repositories(&selections.iter().map(|&i| repos[i]).collect::<Vec<_>>()),
        1 => {
            for &idx in &selections {
                let repo = repos[idx];
                println!("\n{}", "─".repeat(60).bright_black());
                println!("Fixing: {}", repo.path.display().to_string().bold());
                fix_repository_interactive(repo, false)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Apply the same action to multiple repositories
fn batch_fix_repositories(repos: &[&RepoStatus]) -> Result<()> {
    let actions = vec![
        "Commit all changes (WIP)",
        "Stash all changes",
        "Pull from remote",
        "Push to remote",
        "Sync (pull + push)",
    ];

    let action_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action to apply")
        .items(&actions)
        .interact()?;

    let action = match action_idx {
        0 => Action::CommitWip {
            message: "WIP: Batch commit by git-nexus".to_string(),
        },
        1 => Action::Stash {
            message: Some("Batch stash by git-nexus".to_string()),
        },
        2 => Action::Pull,
        3 => Action::Push,
        4 => Action::Sync,
        _ => return Ok(()),
    };

    // Confirm
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Apply '{}' to {} repositories?",
            action.description(),
            repos.len()
        ))
        .default(false)
        .interact()?;

    if !confirmed {
        println!("Cancelled.");
        return Ok(());
    }

    // Apply to all with progress bar
    let pb = ProgressBar::new(repos.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
            .progress_chars("█▓▒░ "),
    );

    for repo in repos {
        pb.set_message(format!("{}", repo.path.display()));
        
        match apply_action(&repo.path, &action, false) {
            Ok(result) => {
                if result.success {
                    pb.println(format!("  {} {}", "✓".green(), repo.path.display()));
                } else {
                    pb.println(format!(
                        "  {} {} - {}",
                        "✗".red(),
                        repo.path.display(),
                        result.message
                    ));
                }
            }
            Err(e) => {
                pb.println(format!(
                    "  {} {} - Error: {}",
                    "✗".red(),
                    repo.path.display(),
                    e
                ));
            }
        }
        
        pb.inc(1);
    }

    pb.finish_with_message("Done!");
    Ok(())
}

/// Show suggestions for all repositories
fn show_all_suggestions(repos: &[&RepoStatus]) -> Result<()> {
    println!("\n{}", "SUGGESTIONS FOR ALL REPOSITORIES".bold().cyan());
    println!("{}", "═".repeat(60).bright_black());

    for repo in repos {
        println!("\n{}", repo.path.display().to_string().bold());
        let suggestions = generate_suggestions(repo);
        
        if suggestions.is_empty() {
            println!("  {} No issues detected", "✓".green());
        } else {
            for suggestion in suggestions.iter().take(3) {
                // Show top 3 suggestions
                suggestion.display();
            }
        }
    }

    println!("\n{}", "═".repeat(60).bright_black());
    Ok(())
}

/// Fix a single repository with interactive prompts
pub fn fix_repository_interactive(repo: &RepoStatus, dry_run: bool) -> Result<()> {
    println!("\n{}", "─".repeat(60).cyan());
    println!("{}: {}", "Repository".bold(), repo.path.display());
    println!("{}", "─".repeat(60).cyan());

    // Show current status
    print_status(repo);

    // Generate suggestions
    let suggestions = generate_suggestions(repo);

    if suggestions.is_empty() {
        println!("\n{} No issues detected!", "✓".green().bold());
        return Ok(());
    }

    println!("\n{}", "Suggestions:".bold().yellow());
    for (idx, suggestion) in suggestions.iter().enumerate() {
        println!("\n  {}. {}", idx + 1, suggestion.title.bold());
        println!("     {}", suggestion.description.bright_black());
        println!("     {} {}", "→".cyan(), suggestion.action.git_command().cyan());
    }

    println!();

    // Ask user to select an action
    let mut items: Vec<String> = suggestions
        .iter()
        .map(|s| s.title.clone())
        .collect();
    items.push("Do nothing (skip)".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action to apply")
        .items(&items)
        .default(0)
        .interact()?;

    if selection == items.len() - 1 {
        println!("Skipped.");
        return Ok(());
    }

    let suggestion = &suggestions[selection];

    // Confirm if destructive
    if suggestion.action.is_destructive() {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "⚠️  This action is DESTRUCTIVE and cannot be undone. Continue?"
            ))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Apply the action
    println!("\nApplying action...");
    match apply_action(&repo.path, &suggestion.action, dry_run) {
        Ok(result) => result.display(),
        Err(e) => println!("  {} Error: {}", "✗".red(), e),
    }

    Ok(())
}

/// Format status indicators for display
fn format_indicators(repo: &RepoStatus) -> String {
    let mut parts = Vec::new();

    if !repo.is_clean {
        parts.push("DIRTY".red().to_string());
    }

    if repo.ahead > 0 {
        parts.push(format!("↑{}", repo.ahead).yellow().to_string());
    }

    if repo.behind > 0 {
        parts.push(format!("↓{}", repo.behind).red().to_string());
    }

    if let Some(ref branch) = repo.branch {
        if branch.starts_with("detached@") {
            parts.push("DETACHED".bright_red().to_string());
        }
    }

    if parts.is_empty() {
        "CLEAN".green().to_string()
    } else {
        format!("[{}]", parts.join(" "))
    }
}

/// Print detailed status of a repository
fn print_status(repo: &RepoStatus) {
    println!("Branch:  {}", repo.branch.as_deref().unwrap_or("unknown").bright_cyan());
    println!(
        "Status:  {}",
        if repo.is_clean {
            "Clean".green()
        } else {
            "Dirty".red()
        }
    );

    if repo.ahead > 0 {
        println!("Ahead:   {} commits", repo.ahead.to_string().yellow());
    }

    if repo.behind > 0 {
        println!("Behind:  {} commits", repo.behind.to_string().red());
    }

    if let Some(modified) = repo.modified_count {
        if modified > 0 {
            println!("Modified: {} files", modified);
        }
    }

    if let Some(untracked) = repo.untracked_count {
        if untracked > 0 {
            println!("Untracked: {} files", untracked);
        }
    }

    if let Some(stash_count) = repo.stash_count {
        if stash_count > 0 {
            println!("Stashes: {}", stash_count);
        }
    }
}
