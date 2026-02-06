//! User-friendly error messages and help system.
//!
//! Provides context and solutions for common errors.

use colored::*;

/// Display a user-friendly error message
#[allow(dead_code)]
pub fn display_error(error: &anyhow::Error) {
    eprintln!("\n{}", "═".repeat(60).red());
    eprintln!("{} {}", "ERROR:".red().bold(), error.to_string().bright_white());
    
    // Try to provide helpful context
    let error_str = error.to_string().to_lowercase();
    
    if error_str.contains("permission denied") {
        eprintln!("\n{}", "💡 Tip:".bright_yellow());
        eprintln!("  This directory may require elevated permissions.");
        eprintln!("  Try running with appropriate access rights.");
    } else if error_str.contains("not a git repository") ||  error_str.contains("failed to open repository") {
        eprintln!("\n{}", "💡 Tip:".bright_yellow());
        eprintln!("  Make sure the path points to a valid git repository.");
        eprintln!("  Check that the .git directory exists.");
    } else if error_str.contains("no such file or directory") {
        eprintln!("\n{}", "💡 Tip:".bright_yellow());
        eprintln!("  The specified path doesn't exist.");
        eprintln!("  Verify the path is correct and accessible.");
    } else if error_str.contains("authentication") || error_str.contains("credentials") {
        eprintln!("\n{}", "💡 Tip:".bright_yellow());
        eprintln!("  Git authentication failed.");
        eprintln!("  Check your SSH keys or Git credentials.");
        eprintln!("  For HTTPS, you may need a personal access token.");
    } else if error_str.contains("merge conflict") {
        eprintln!("\n{}", "💡 Tip:".bright_yellow());
        eprintln!("  There are merge conflicts that need manual resolution.");
        eprintln!("  Resolve conflicts and try again.");
    }
    
    eprintln!("{}", "═".repeat(60).red());
    eprintln!();
}

/// Display a warning message
#[allow(dead_code)]
pub fn display_warning(message: &str) {
    println!("\n{} {}", "⚠️  WARNING:".yellow().bold(), message.bright_white());
}

/// Display an info message
#[allow(dead_code)]
pub fn display_info(message: &str) {
    println!("{} {}", "ℹ️ ".bright_cyan(), message);
}

/// Display success message
#[allow(dead_code)]
pub fn display_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message.green());
}

/// Show help for common operations
#[allow(dead_code)]
pub fn show_quick_help() {
    println!("\n{}", "Quick Help:".bold().bright_cyan());
    println!("  {} git-nexus              # Scan current directory", "▸".bright_black());
    println!("  {} git-nexus --suggest    # Show fix suggestions", "▸".bright_black());
    println!("  {} git-nexus --health     # Show health scores", "▸".bright_black());
    println!("  {} git-nexus fix          # Interactive fix mode", "▸".bright_black());
    println!("  {} git-nexus tui          # Terminal UI mode", "▸".bright_black());
    println!("  {} git-nexus --help       # Full help", "▸".bright_black());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = anyhow::anyhow!("permission denied");
        display_error(&err);
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_warning_display() {
        display_warning("Test warning");
        // Just ensure it doesn't panic
    }
}
