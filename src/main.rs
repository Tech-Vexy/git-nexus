//! # git-nexus
//!
//! A blazing fast multi-repository scanner for developers who juggle multiple projects.
//!
//! ## Overview
//!
//! git-nexus provides a high-performance tool to scan and monitor multiple Git repositories
//! across your filesystem. It uses Rust's parallel processing capabilities to quickly analyze
//! repository status, branch information, and uncommitted changes.
//!
//! ## Features
//!
//! - **Fast Scanning**: Parallel repository analysis using Rayon
//! - **Status Tracking**: See clean/dirty status, ahead/behind counts, stashes
//! - **Flexible Output**: Terminal display, JSON, HTML, or CSV export
//! - **Interactive TUI**: Terminal UI for browsing repositories
//! - **Watch Mode**: Real-time monitoring of repository changes
//! - **Configurable**: Customize behavior with TOML configuration files
//!
//! ## Example
//!
//! ```no_run
//! use git_nexus::{scan_repositories, display_repo_status};
//! use std::path::PathBuf;
//!
//! let repos = scan_repositories(
//!     &PathBuf::from("."),
//!     3,                    // max depth
//!     false,                // verbose
//!     &["node_modules".to_string()], // ignore dirs
//!     false,                // show hooks
//! );
//!
//! for repo in &repos {
//!     display_repo_status(repo, false, false);
//! }
//! ```

mod config;
mod export;
mod github;
mod hooks;
mod interactive;
mod resolution;
mod suggestions;
mod tui;
mod watch;

use anyhow::Result;
use chrono::{DateTime, Local};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use config::Config;
use git2::{Repository, StatusOptions};
use rayon::prelude::*;
use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "git-nexus")]
#[command(version, about = "A blazing fast multi-repository scanner for developers", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(default_value = ".", help = "Root directory to scan for repositories")]
    path: PathBuf,

    #[arg(short, long, help = "Maximum directory traversal depth")]
    depth: Option<usize>,

    #[arg(short, long, help = "Output in JSON format")]
    json: bool,

    #[arg(short = 'v', long, help = "Show verbose information")]
    verbose: bool,

    #[arg(short, long, help = "Filter repositories by status")]
    filter: Option<StatusFilter>,

    #[arg(short, long, value_enum, default_value = "path", help = "Sort repositories by field")]
    sort: SortBy,

    #[arg(long, help = "Show git hooks information")]
    show_hooks: bool,

    #[arg(long, help = "Show GitHub info (requires token in config)")]
    show_github: bool,
    
    #[arg(long, help = "Show suggestions for fixing issues")]
    suggest: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive TUI mode
    Tui,
    
    /// Watch mode - continuously monitor for changes
    Watch,
    
    /// Interactive fix mode - resolve repository issues interactively
    Fix {
        /// Specific repository path to fix (optional, if not provided shows all)
        path: Option<PathBuf>,
        
        /// Run in dry-run mode (show what would be done without doing it)
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Export to HTML or CSV
    Export {
        #[arg(value_enum)]
        format: ExportFormat,
        
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Generate example configuration file
    Config {
        #[arg(short, long, default_value = ".git-nexus.toml")]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum ExportFormat {
    Html,
    Csv,
}

#[derive(Debug, Clone, ValueEnum)]
enum StatusFilter {
    Clean,
    Dirty,
    Ahead,
    Behind,
}

#[derive(Debug, Clone, ValueEnum)]
enum SortBy {
    Path,
    Status,
    Branch,
}

#[derive(Debug, Serialize, Clone)]
pub struct RepoStatus {
    /// Path to the repository
    path: PathBuf,
    /// Whether the repository has uncommitted changes
    is_clean: bool,
    /// Number of commits ahead of remote
    ahead: usize,
    /// Number of commits behind remote
    behind: usize,
    /// Current branch name (or None if detached HEAD)
    branch: Option<String>,
    /// Number of stashes (only in verbose mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    stash_count: Option<usize>,
    /// Number of modified/staged files (only in verbose mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_count: Option<usize>,
    /// Number of untracked files (only in verbose mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    untracked_count: Option<usize>,
    /// Information about the last commit (only in verbose mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    last_commit: Option<CommitInfo>,
    /// Git hooks information (only when show_hooks is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    hooks: Option<hooks::GitHooks>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CommitInfo {
    /// First line of commit message
    message: String,
    /// Author name
    author: String,
    /// Formatted timestamp (YYYY-MM-DD HH:MM:SS)
    timestamp: String,
    /// Short commit hash (7 characters)
    hash: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load().unwrap_or_default();

    // Handle subcommands
    match cli.command {
        Some(Commands::Tui) => {
            let repos = scan_repositories(
                &cli.path,
                cli.depth.unwrap_or(config.scan_depth),
                cli.verbose || config.display.default_verbose,
                &config.ignore_dirs,
                cli.show_hooks,
            );
            return tui::run_tui(repos);
        }
        Some(Commands::Watch) => {
            return watch::watch_mode(&cli.path, &config, cli.verbose);
        }
        Some(Commands::Fix { path, dry_run }) => {
            let scan_path = path.as_ref().unwrap_or(&cli.path);
            let repos = scan_repositories(
                scan_path,
                cli.depth.unwrap_or(config.scan_depth),
                true, // verbose for fix mode
                &config.ignore_dirs,
                false,
            );
            
            if let Some(specific_path) = path {
                // Fix specific repository
                if let Some(repo) = repos.iter().find(|r| r.path == specific_path) {
                    return interactive::fix_repository_interactive(repo, dry_run);
                } else {
                    println!("{}", "Repository not found.".red());
                    return Ok(());
                }
            } else {
                // Interactive mode to select repositories
                return interactive::interactive_fix_mode(&repos);
            }
        }
        Some(Commands::Export { format, output }) => {
            let repos = scan_repositories(
                &cli.path,
                cli.depth.unwrap_or(config.scan_depth),
                true,
                &config.ignore_dirs,
                cli.show_hooks,
            );
            
            match format {
                ExportFormat::Html => export::export_html(&repos, &output)?,
                ExportFormat::Csv => export::export_csv(&repos, &output)?,
            }
            
            println!("✅ Exported to {}", output.display());
            return Ok(());
        }
        Some(Commands::Config { output }) => {
            Config::create_example(&output)?;
            println!("✅ Created example config at {}", output.display());
            return Ok(());
        }
        None => {}
    }

    // Normal scan mode
    if !cli.json {
        println!("{}", "🔍 Scanning workspace for git repositories...".bright_cyan().bold());
        println!();
    }

    let mut repos = scan_repositories(
        &cli.path,
        cli.depth.unwrap_or(config.scan_depth),
        cli.verbose || config.display.default_verbose,
        &config.ignore_dirs,
        cli.show_hooks,
    );

    // Apply filter
    if let Some(ref filter) = cli.filter {
        repos.retain(|r| match filter {
            StatusFilter::Clean => r.is_clean,
            StatusFilter::Dirty => !r.is_clean,
            StatusFilter::Ahead => r.ahead > 0,
            StatusFilter::Behind => r.behind > 0,
        });
    }

    // Sort repositories
    match cli.sort {
        SortBy::Path => repos.sort_by(|a, b| a.path.cmp(&b.path)),
        SortBy::Status => repos.sort_by(|a, b| a.is_clean.cmp(&b.is_clean)),
        SortBy::Branch => repos.sort_by(|a, b| a.branch.cmp(&b.branch)),
    }

    if repos.is_empty() {
        if !cli.json {
            println!("{}", "No git repositories found.".yellow());
        }
        return Ok(());
    }

    if cli.json {
        let json = serde_json::to_string_pretty(&repos)?;
        println!("{}", json);
    } else {
        println!("{} {} repositories found\n", "✓".green().bold(), repos.len());
        
        // Show summary if suggesting
        if cli.suggest {
            let summary = suggestions::summarize_issues(&repos);
            summary.display();
            println!();
        }
        
        for repo in &repos {
            display_repo_status(repo, cli.verbose, cli.show_hooks);
            
            // Show suggestions if requested
            if cli.suggest {
                let repo_suggestions = suggestions::generate_suggestions(repo);
                if !repo_suggestions.is_empty() {
                    println!();
                    for suggestion in repo_suggestions.iter().take(2) {
                        suggestion.display();
                    }
                    println!();
                }
            }
        }
        
        // Show actionable footer if there are issues
        if cli.suggest {
            let has_issues = repos.iter().any(|r| !r.is_clean || r.ahead > 0 || r.behind > 0);
            if has_issues {
                println!("\n{}", "═".repeat(60).bright_black());
                println!("{}", "  💡 TIP: Run 'git-nexus fix' to interactively resolve issues".bright_cyan());
                println!("{}", "═".repeat(60).bright_black());
            }
        }
    }

    Ok(())
}

/// Scans a directory tree for Git repositories and analyzes their status.
///
/// This function performs a parallel scan of the filesystem to locate and analyze
/// Git repositories. It respects the ignore_dirs list to skip common build/dependency
/// directories and uses Rayon for parallel processing.
///
/// # Arguments
///
/// * `root` - Root directory to start scanning from
/// * `max_depth` - Maximum depth for directory traversal
/// * `verbose` - Whether to collect detailed information (commits, stashes, file counts)
/// * `ignore_dirs` - List of directory names to skip during scanning
/// * `show_hooks` - Whether to detect and report Git hooks
///
/// # Returns
///
/// A vector of `RepoStatus` structs containing information about each repository found.
///
/// # Example
///
/// ```no_run
/// # use git_nexus::scan_repositories;
/// # use std::path::PathBuf;
/// let repos = scan_repositories(
///     &PathBuf::from("/home/user/projects"),
///     3,
///     true,
///     &["node_modules".to_string(), "target".to_string()],
///     false,
/// );
/// println!("Found {} repositories", repos.len());
/// ```
pub fn scan_repositories(
    root: &PathBuf,
    max_depth: usize,
    verbose: bool,
    ignore_dirs: &[String],
    show_hooks: bool,
) -> Vec<RepoStatus> {
    let git_dirs: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !ignore_dirs.contains(&name.to_string()) || name == ".git"
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() && e.file_name() == ".git")
        .filter_map(|e| e.path().parent().map(|p| p.to_path_buf()))
        .collect();

    git_dirs
        .par_iter()
        .filter_map(|path| analyze_repository(path, verbose, show_hooks))
        .collect()
}

/// Analyzes a single Git repository and returns its status.
///
/// This function opens a Git repository and extracts various status information
/// including branch name, clean/dirty status, ahead/behind counts, and optionally
/// detailed information like stashes and last commit.
///
/// # Arguments
///
/// * `path` - Path to the repository (parent directory of .git)
/// * `verbose` - Whether to collect detailed information
/// * `show_hooks` - Whether to detect Git hooks
///
/// # Returns
///
/// `Some(RepoStatus)` if the repository can be analyzed, `None` if there's an error.
fn analyze_repository(path: &std::path::Path, verbose: bool, show_hooks: bool) -> Option<RepoStatus> {
    let repo = Repository::open(path).ok()?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts)).ok()?;
    let is_clean = statuses.is_empty();

    let branch = get_current_branch(&repo);
    let (ahead, behind) = get_branch_divergence(&repo).unwrap_or((0, 0));

    let (stash_count, modified_count, untracked_count, last_commit) = if verbose {
        let stash = count_stashes(&repo);
        let (modified, untracked) = count_file_changes(&statuses);
        let commit = get_last_commit(&repo);
        (Some(stash), Some(modified), Some(untracked), commit)
    } else {
        (None, None, None, None)
    };

    let hooks = if show_hooks {
        hooks::GitHooks::detect(path)
    } else {
        None
    };

    Some(RepoStatus {
        path: path.to_path_buf(),
        is_clean,
        ahead,
        behind,
        branch,
        stash_count,
        modified_count,
        untracked_count,
        last_commit,
        hooks,
    })
}

/// Gets the current branch name of a repository.
///
/// Handles various Git states:
/// - Normal branch: returns branch name
/// - Detached HEAD: returns "detached@{short_hash}"
/// - Unborn branch (no commits): returns "{branch} (no commits)"
///
/// # Arguments
///
/// * `repo` - Reference to the Git repository
///
/// # Returns
///
/// `Some(String)` with the branch name/state, or `None` if the HEAD can't be read.
fn get_current_branch(repo: &Repository) -> Option<String> {
    match repo.head() {
        Ok(head) => {
            if head.is_branch() {
                head.shorthand().map(|s| s.to_string())
            } else if let Some(target) = head.target() {
                Some(format!("detached@{}", &target.to_string()[..7]))
            } else {
                Some("(no commits)".to_string())
            }
        }
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
            if let Ok(reference) = repo.find_reference("HEAD") {
                if let Some(target) = reference.symbolic_target() {
                    let branch = target.strip_prefix("refs/heads/").unwrap_or(target);
                    return Some(format!("{} (no commits)", branch));
                }
            }
            Some("(no commits)".to_string())
        }
        Err(_) => None,
    }
}

/// Calculates how many commits ahead and behind the local branch is compared to upstream.
///
/// # Arguments
///
/// * `repo` - Reference to the Git repository
///
/// # Returns
///
/// `Ok((ahead, behind))` tuple with commit counts, or an error if the calculation fails.
/// Returns `(0, 0)` if the HEAD is detached or there's no upstream branch.
fn get_branch_divergence(repo: &Repository) -> Result<(usize, usize), git2::Error> {
    let head = repo.head()?;

    if !head.is_branch() {
        return Ok((0, 0));
    }

    let local_commit = head.peel_to_commit()?;
    let local_oid = local_commit.id();

    let branch = repo.find_branch(head.shorthand().unwrap(), git2::BranchType::Local)?;
    let upstream = branch.upstream();

    if let Ok(upstream_branch) = upstream {
        let upstream_oid = upstream_branch.get().peel_to_commit()?.id();
        let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
        Ok((ahead, behind))
    } else {
        Ok((0, 0))
    }
}

/// Counts the number of stashes in a repository.
///
/// # Arguments
///
/// * `repo` - Reference to the Git repository
///
/// # Returns
///
/// The number of stashes, or 0 if there are no stashes or the reflog can't be read.
fn count_stashes(repo: &Repository) -> usize {
    if let Ok(reflog) = repo.reflog("refs/stash") {
        reflog.len()
    } else {
        0
    }
}

/// Counts modified/staged and untracked files in the repository.
///
/// # Arguments
///
/// * `statuses` - Git status entries from the repository
///
/// # Returns
///
/// A tuple `(modified_count, untracked_count)` where:
/// - `modified_count` includes modified, deleted, renamed files in working tree or index
/// - `untracked_count` includes only new untracked files
fn count_file_changes(statuses: &git2::Statuses) -> (usize, usize) {
    let mut modified = 0;
    let mut untracked = 0;

    for entry in statuses.iter() {
        let status = entry.status();
        if status.is_wt_new() {
            untracked += 1;
        } else if status.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE
                | git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            modified += 1;
        }
    }

    (modified, untracked)
}

/// Retrieves information about the most recent commit in the repository.
///
/// # Arguments
///
/// * `repo` - Reference to the Git repository
///
/// # Returns
///
/// `Some(CommitInfo)` with details about the last commit, or `None` if:
/// - The repository has no commits
/// - The HEAD can't be read
/// - The commit can't be parsed
fn get_last_commit(repo: &Repository) -> Option<CommitInfo> {
    let head = repo.head().ok()?;
    let commit = head.peel_to_commit().ok()?;

    let message = commit.message().unwrap_or("").lines().next().unwrap_or("").to_string();
    let author = commit.author().name().unwrap_or("Unknown").to_string();
    let timestamp = commit.time();
    let datetime: DateTime<Local> = DateTime::from_timestamp(timestamp.seconds(), 0)?.into();
    let hash = commit.id().to_string()[..7].to_string();

    Some(CommitInfo {
        message,
        author,
        timestamp: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
        hash,
    })
}

/// Displays repository status information to the terminal with color formatting.
///
/// # Arguments
///
/// * `status` - The repository status to display
/// * `verbose` - Whether to show detailed information (stashes, file counts, last commit)
/// * `show_hooks` - Whether to display Git hooks information
///
/// # Output Format
///
/// Basic: `📁 /path/to/repo (branch) [STATUS] ↑ahead ↓behind`
///
/// Verbose adds: `📦stashes ~modified +untracked`
/// and commit info: `└─ hash · author · message`
pub fn display_repo_status(status: &RepoStatus, verbose: bool, show_hooks: bool) {
    let path_display = status.path.display().to_string();

    let status_label = if status.is_clean {
        "CLEAN".green().bold()
    } else {
        "DIRTY".red().bold()
    };

    let branch_display = if let Some(ref branch) = status.branch {
        format!(" ({})", branch).bright_blue().to_string()
    } else {
        String::new()
    };

    print!("📁 {}{} [{}]", path_display.bright_white().bold(), branch_display, status_label);

    if status.ahead > 0 {
        print!(" {}{}", "↑".yellow(), status.ahead.to_string().yellow());
    }

    if status.behind > 0 {
        print!(" {}{}", "↓".red(), status.behind.to_string().red());
    }

    if verbose {
        if let Some(stash) = status.stash_count {
            if stash > 0 {
                print!(" {}📦{}", " ".clear(), stash.to_string().bright_magenta());
            }
        }

        if let Some(modified) = status.modified_count {
            if modified > 0 {
                print!(" {}~{}", " ".clear(), modified.to_string().bright_yellow());
            }
        }

        if let Some(untracked) = status.untracked_count {
            if untracked > 0 {
                print!(" {}+{}", " ".clear(), untracked.to_string().bright_cyan());
            }
        }
    }

    if show_hooks {
        if let Some(ref hooks) = status.hooks {
            if hooks.has_any() {
                print!(" {}🪝{}", " ".clear(), hooks.active_hooks().len().to_string().bright_magenta());
            }
        }
    }

    println!();

    if verbose {
        if let Some(ref commit) = status.last_commit {
            println!("   {} {} · {} · {}", "└─".bright_black(), commit.hash.bright_black(), commit.author.bright_black(), commit.message.bright_black());
        }
    }

    if show_hooks {
        if let Some(ref hooks) = status.hooks {
            if hooks.has_any() {
                let hooks_list = hooks.active_hooks().join(", ");
                println!("   {} hooks: {}", "└─".bright_black(), hooks_list.bright_black());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a temporary git repository
    fn create_test_repo(temp_dir: &TempDir) -> Result<PathBuf> {
        let repo_path = temp_dir.path().join("test_repo");
        fs::create_dir(&repo_path)?;
        
        let repo = Repository::init(&repo_path)?;
        
        // Create initial commit
        let signature = repo.signature().unwrap_or_else(|_| {
            git2::Signature::now("Test User", "test@example.com").unwrap()
        });
        
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        let tree = repo.find_tree(tree_id)?;
        
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;
        
        Ok(repo_path)
    }

    #[test]
    fn test_count_file_changes_empty() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        let repo = Repository::open(&repo_path).unwrap();
        
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts)).unwrap();
        
        let (modified, untracked) = count_file_changes(&statuses);
        assert_eq!(modified, 0);
        assert_eq!(untracked, 0);
    }

    #[test]
    fn test_count_file_changes_with_untracked() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        
        // Add an untracked file
        let file_path = repo_path.join("untracked.txt");
        fs::write(&file_path, "test content").unwrap();
        
        let repo = Repository::open(&repo_path).unwrap();
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts)).unwrap();
        
        let (modified, untracked) = count_file_changes(&statuses);
        assert_eq!(modified, 0);
        assert_eq!(untracked, 1);
    }

    #[test]
    fn test_count_file_changes_with_modified() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        let repo = Repository::open(&repo_path).unwrap();
        
        // Create and commit a file
        let file_path = repo_path.join("tracked.txt");
        fs::write(&file_path, "initial content").unwrap();
        
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("tracked.txt")).unwrap();
        index.write().unwrap();
        
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();
        let signature = repo.signature().unwrap_or_else(|_| {
            git2::Signature::now("Test User", "test@example.com").unwrap()
        });
        
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Add tracked file",
            &tree,
            &[&parent_commit],
        ).unwrap();
        
        // Modify the file
        fs::write(&file_path, "modified content").unwrap();
        
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts)).unwrap();
        
        let (modified, untracked) = count_file_changes(&statuses);
        assert_eq!(modified, 1);
        assert_eq!(untracked, 0);
    }

    #[test]
    fn test_count_stashes_empty() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        let repo = Repository::open(&repo_path).unwrap();
        
        let stash_count = count_stashes(&repo);
        assert_eq!(stash_count, 0);
    }

    #[test]
    fn test_get_current_branch() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        let repo = Repository::open(&repo_path).unwrap();
        
        let branch = get_current_branch(&repo);
        assert!(branch.is_some());
        // Default branch is usually "master" or "main"
        let branch_name = branch.unwrap();
        assert!(branch_name == "master" || branch_name == "main");
    }

    #[test]
    fn test_get_branch_divergence_no_remote() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        let repo = Repository::open(&repo_path).unwrap();
        
        let (ahead, behind) = get_branch_divergence(&repo).unwrap_or((0, 0));
        assert_eq!(ahead, 0);
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_analyze_repository_clean() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        
        let status = analyze_repository(&repo_path, false, false);
        assert!(status.is_some());
        
        let status = status.unwrap();
        assert!(status.is_clean);
        assert_eq!(status.ahead, 0);
        assert_eq!(status.behind, 0);
        assert!(status.branch.is_some());
        assert!(status.stash_count.is_none()); // Not in verbose mode
    }

    #[test]
    fn test_analyze_repository_dirty() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        
        // Add an untracked file to make it dirty
        fs::write(repo_path.join("new_file.txt"), "content").unwrap();
        
        let status = analyze_repository(&repo_path, false, false);
        assert!(status.is_some());
        
        let status = status.unwrap();
        assert!(!status.is_clean);
    }

    #[test]
    fn test_analyze_repository_verbose() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        
        let status = analyze_repository(&repo_path, true, false);
        assert!(status.is_some());
        
        let status = status.unwrap();
        assert!(status.stash_count.is_some());
        assert!(status.modified_count.is_some());
        assert!(status.untracked_count.is_some());
        assert!(status.last_commit.is_some());
    }

    #[test]
    fn test_get_last_commit() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir).unwrap();
        let repo = Repository::open(&repo_path).unwrap();
        
        let commit_info = get_last_commit(&repo);
        assert!(commit_info.is_some());
        
        let commit = commit_info.unwrap();
        assert_eq!(commit.message, "Initial commit");
        assert!(!commit.author.is_empty());
        assert!(!commit.timestamp.is_empty());
        assert_eq!(commit.hash.len(), 7);
    }

    #[test]
    fn test_scan_repositories() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple test repos
        let _repo1 = create_test_repo(&temp_dir).unwrap();
        
        let repos = scan_repositories(
            &temp_dir.path().to_path_buf(),
            3,
            false,
            &["node_modules".to_string(), "target".to_string()],
            false,
        );
        
        assert!(!repos.is_empty());
        assert_eq!(repos.len(), 1);
    }

    #[test]
    fn test_scan_repositories_respects_ignore_dirs() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a repo in a directory that should be ignored
        let node_modules = temp_dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        
        let repo_path = node_modules.join("some_repo");
        fs::create_dir(&repo_path).unwrap();
        Repository::init(&repo_path).unwrap();
        
        let repos = scan_repositories(
            &temp_dir.path().to_path_buf(),
            3,
            false,
            &["node_modules".to_string()],
            false,
        );
        
        // Should not find the repo in node_modules
        assert_eq!(repos.len(), 0);
    }

    #[test]
    fn test_scan_repositories_respects_depth() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create nested directories
        let deep_path = temp_dir.path()
            .join("level1")
            .join("level2")
            .join("level3")
            .join("level4");
        fs::create_dir_all(&deep_path).unwrap();
        
        let repo_path = deep_path.join("deep_repo");
        fs::create_dir(&repo_path).unwrap();
        Repository::init(&repo_path).unwrap();
        
        // Scan with depth 3 (should not find level4)
        let repos = scan_repositories(
            &temp_dir.path().to_path_buf(),
            3,
            false,
            &[],
            false,
        );
        
        assert_eq!(repos.len(), 0);
        
        // Scan with depth 6 (should find it)
        let repos = scan_repositories(
            &temp_dir.path().to_path_buf(),
            6,
            false,
            &[],
            false,
        );
        
        assert_eq!(repos.len(), 1);
    }

    #[test]
    fn test_repo_status_serialization() {
        let status = RepoStatus {
            path: PathBuf::from("/test/path"),
            is_clean: true,
            ahead: 2,
            behind: 1,
            branch: Some("main".to_string()),
            stash_count: Some(3),
            modified_count: Some(5),
            untracked_count: Some(2),
            last_commit: Some(CommitInfo {
                message: "Test commit".to_string(),
                author: "Test Author".to_string(),
                timestamp: "2024-01-01 12:00:00".to_string(),
                hash: "abc1234".to_string(),
            }),
            hooks: None,
        };
        
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"is_clean\":true"));
        assert!(json.contains("\"ahead\":2"));
        assert!(json.contains("\"behind\":1"));
        assert!(json.contains("\"branch\":\"main\""));
    }
}
