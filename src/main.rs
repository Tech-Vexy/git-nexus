use clap::Parser;
use colored::*;
use git2::{Repository, StatusOptions};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "git-nexus")]
#[command(about = "A blazing fast multi-repository scanner for developers", long_about = None)]
struct Cli {
    #[arg(default_value = ".", help = "Root directory to scan for repositories")]
    path: PathBuf,

    #[arg(short, long, default_value = "3", help = "Maximum directory traversal depth")]
    depth: usize,
}

#[derive(Debug)]
struct RepoStatus {
    path: PathBuf,
    is_clean: bool,
    ahead: usize,
    behind: usize,
    branch: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    
    println!("{}", "🔍 Scanning workspace for git repositories...".bright_cyan().bold());
    println!();

    let repos = scan_repositories(&cli.path, cli.depth);
    
    if repos.is_empty() {
        println!("{}", "No git repositories found.".yellow());
        return;
    }

    println!("{} {} repositories found\n", "✓".green().bold(), repos.len());

    for repo in repos {
        display_repo_status(&repo);
    }
}

fn scan_repositories(root: &PathBuf, max_depth: usize) -> Vec<RepoStatus> {
    let mut repos = Vec::new();
    let ignore_dirs = ["node_modules", "target", "venv", ".build", "build", "dist", ".next"];

    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !ignore_dirs.contains(&name.as_ref()) || name == ".git"
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            if let Some(repo_path) = entry.path().parent() {
                if let Some(status) = analyze_repository(repo_path) {
                    repos.push(status);
                }
            }
        }
    }

    repos
}

fn analyze_repository(path: &std::path::Path) -> Option<RepoStatus> {
    let repo = Repository::open(path).ok()?;
    
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.include_ignored(false);
    
    let statuses = repo.statuses(Some(&mut opts)).ok()?;
    let is_clean = statuses.is_empty();
    
    let branch = get_current_branch(&repo);
    let (ahead, behind) = get_branch_divergence(&repo).unwrap_or((0, 0));
    
    Some(RepoStatus {
        path: path.to_path_buf(),
        is_clean,
        ahead,
        behind,
        branch,
    })
}

fn get_current_branch(repo: &Repository) -> Option<String> {
    match repo.head() {
        Ok(head) => {
            if head.is_branch() {
                head.shorthand().map(|s| s.to_string())
            } else if let Some(target) = head.target() {
                // Detached HEAD state
                Some(format!("detached@{}", &target.to_string()[..7]))
            } else {
                Some("(no commits)".to_string())
            }
        }
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
            // Repository exists but has no commits yet
            // Try to get the branch name from HEAD reference
            if let Ok(reference) = repo.find_reference("HEAD") {
                if let Some(target) = reference.symbolic_target() {
                    // Extract branch name from refs/heads/branch_name
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

fn display_repo_status(status: &RepoStatus) {
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
    
    print!("📁 {}{} [{}]", 
        path_display.bright_white().bold(), 
        branch_display,
        status_label
    );
    
    if status.ahead > 0 {
        print!(" {}{}", "↑".yellow(), status.ahead.to_string().yellow());
    }
    
    if status.behind > 0 {
        print!(" {}{}", "↓".red(), status.behind.to_string().red());
    }
    
    println!();
}
