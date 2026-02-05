use std::fs;
use std::path::Path;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GitHooks {
    pub pre_commit: bool,
    pub pre_push: bool,
    pub post_commit: bool,
    pub post_merge: bool,
    pub prepare_commit_msg: bool,
    pub commit_msg: bool,
}

impl GitHooks {
    pub fn detect(repo_path: &Path) -> Option<Self> {
        let hooks_dir = repo_path.join(".git/hooks");
        
        if !hooks_dir.exists() {
            return None;
        }
        
        Some(Self {
            pre_commit: hook_exists(&hooks_dir, "pre-commit"),
            pre_push: hook_exists(&hooks_dir, "pre-push"),
            post_commit: hook_exists(&hooks_dir, "post-commit"),
            post_merge: hook_exists(&hooks_dir, "post-merge"),
            prepare_commit_msg: hook_exists(&hooks_dir, "prepare-commit-msg"),
            commit_msg: hook_exists(&hooks_dir, "commit-msg"),
        })
    }
    
    pub fn has_any(&self) -> bool {
        self.pre_commit
            || self.pre_push
            || self.post_commit
            || self.post_merge
            || self.prepare_commit_msg
            || self.commit_msg
    }
    
    pub fn active_hooks(&self) -> Vec<&str> {
        let mut hooks = Vec::new();
        if self.pre_commit {
            hooks.push("pre-commit");
        }
        if self.pre_push {
            hooks.push("pre-push");
        }
        if self.post_commit {
            hooks.push("post-commit");
        }
        if self.post_merge {
            hooks.push("post-merge");
        }
        if self.prepare_commit_msg {
            hooks.push("prepare-commit-msg");
        }
        if self.commit_msg {
            hooks.push("commit-msg");
        }
        hooks
    }
}

fn hook_exists(hooks_dir: &Path, hook_name: &str) -> bool {
    let hook_path = hooks_dir.join(hook_name);
    if !hook_path.exists() {
        return false;
    }
    
    // Check if it's executable (not just a .sample file)
    if let Ok(_metadata) = fs::metadata(&hook_path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = _metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
        
        #[cfg(not(unix))]
        {
            return true;
        }
    }
    
    false
}
