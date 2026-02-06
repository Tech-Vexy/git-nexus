//! Desktop notifications for git-nexus events.
//!
//! Provides native desktop notifications for repository changes in watch mode.

#[cfg(feature = "notifications")]
use notify_rust::{Notification, Timeout};

/// Send a notification about repository changes
#[cfg(feature = "notifications")]
pub fn notify_repo_change(repo_path: &str, change_type: &str) {
    let _ = Notification::new()
        .summary("git-nexus")
        .body(&format!("{}: {}", repo_path, change_type))
        .timeout(Timeout::Milliseconds(5000))
        .show();
}

/// Send a notification about multiple repository changes
#[cfg(feature = "notifications")]
pub fn notify_multiple_changes(count: usize) {
    let _ = Notification::new()
        .summary("git-nexus")
        .body(&format!("{} repositories have changed", count))
        .timeout(Timeout::Milliseconds(5000))
        .show();
}

/// Send a notification about issues detected
#[cfg(feature = "notifications")]
pub fn notify_issues(repo_path: &str, issue_count: usize) {
    let _ = Notification::new()
        .summary("git-nexus - Issues Detected")
        .body(&format!("{}: {} issues found", repo_path, issue_count))
        .timeout(Timeout::Milliseconds(7000))
        .show();
}

/// Send a success notification
#[cfg(feature = "notifications")]
pub fn notify_success(message: &str) {
    let _ = Notification::new()
        .summary("git-nexus - Success")
        .body(message)
        .timeout(Timeout::Milliseconds(4000))
        .show();
}

/// Send an error notification
#[cfg(feature = "notifications")]
pub fn notify_error(message: &str) {
    let _ = Notification::new()
        .summary("git-nexus - Error")
        .body(message)
        .timeout(Timeout::Milliseconds(7000))
        .show();
}

// Stub implementations when notifications feature is disabled
#[cfg(not(feature = "notifications"))]
pub fn notify_repo_change(_repo_path: &str, _change_type: &str) {}

#[cfg(not(feature = "notifications"))]
pub fn notify_multiple_changes(_count: usize) {}

#[cfg(not(feature = "notifications"))]
pub fn notify_issues(_repo_path: &str, _issue_count: usize) {}

#[cfg(not(feature = "notifications"))]
pub fn notify_success(_message: &str) {}

#[cfg(not(feature = "notifications"))]
pub fn notify_error(_message: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifications_compile() {
        // Just ensure the module compiles with both features on/off
        notify_repo_change("/test/repo", "modified");
        notify_multiple_changes(5);
        notify_issues("/test/repo", 3);
        notify_success("Test");
        notify_error("Error");
    }
}
