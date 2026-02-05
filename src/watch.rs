use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::{scan_repositories, display_repo_status, Config};

pub fn watch_mode(path: &PathBuf, config: &Config, verbose: bool) -> Result<()> {
    println!("👁️  Watch mode activated. Monitoring for git changes...");
    println!("   Press Ctrl+C to exit\n");

    let (tx, rx) = channel();
    
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            let _ = tx.send(res);
        },
        notify::Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;
    
    watcher.watch(path, RecursiveMode::Recursive)?;

    // Initial scan
    print_scan(path, config, verbose);

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                // Check if it's a git-related change
                let path_str = format!("{:?}", event);
                if path_str.contains(".git") {
                    println!("\n🔄 Git change detected, rescanning...\n");
                    print_scan(path, config, verbose);
                }
            }
            Ok(Err(e)) => {
                eprintln!("Watch error: {}", e);
            }
            Err(e) => {
                eprintln!("Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn print_scan(path: &PathBuf, config: &Config, verbose: bool) {
    let repos = scan_repositories(path, config.scan_depth, verbose, &config.ignore_dirs, false);
    
    println!("🔍 Scan complete at {}", chrono::Local::now().format("%H:%M:%S"));
    println!("✓ {} repositories found\n", repos.len());
    
    for repo in repos {
        display_repo_status(&repo, verbose, false);
    }
    
    println!("\n---");
}
