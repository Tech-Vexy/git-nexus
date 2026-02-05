mod config;
mod export;
mod github;
mod hooks;
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
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive TUI mode
    Tui,
    
    /// Watch mode - continuously monitor for changes
    Watch,
    
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
    path: PathBuf,
    is_clean: bool,
    ahead: usize,
    behind: usize,
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stash_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    untracked_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_commit: Option<CommitInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hooks: Option<hooks::GitHooks>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CommitInfo {
    message: String,
    author: String,
    timestamp: String,
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
        for repo in repos {
            display_repo_status(&repo, cli.verbose, cli.show_hooks);
        }
    }

    Ok(())
}

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

fn count_stashes(repo: &Repository) -> usize {
    if let Ok(reflog) = repo.reflog("refs/stash") {
        reflog.len()
    } else {
        0
    }
}

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
