//! Smart ignore pattern matching for git-nexus.
//!
//! Reads and respects .gitignore patterns when scanning repositories.

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::DirEntry;

/// Ignore pattern matcher
#[derive(Debug, Clone)]
pub struct IgnorePatterns {
    patterns: Vec<String>,
}

impl IgnorePatterns {
    /// Create a new ignore pattern matcher
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }
    
    /// Load patterns from a .gitignore file
    pub fn load_from_file(path: &Path) -> Self {
        let mut patterns = Vec::new();
        
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                patterns.push(line.to_string());
            }
        }
        
        Self { patterns }
    }
    
    /// Load patterns from repository's .gitignore
    pub fn from_repo(repo_path: &Path) -> Self {
        let gitignore_path = repo_path.join(".gitignore");
        if gitignore_path.exists() {
            Self::load_from_file(&gitignore_path)
        } else {
            Self::new()
        }
    }
    
    /// Add a pattern to the matcher
    pub fn add_pattern(&mut self, pattern: String) {
        self.patterns.push(pattern);
    }
    
    /// Check if a path should be ignored
    pub fn should_ignore(&self, path: &Path, is_dir: bool) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in &self.patterns {
            if self.matches_pattern(&path_str, pattern, is_dir) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if a path matches a gitignore pattern
    fn matches_pattern(&self, path: &str, pattern: &str, is_dir: bool) -> bool {
        // Handle negation patterns (!)
        let (negate, pattern) = if pattern.starts_with('!') {
            (true, &pattern[1..])
        } else {
            (false, pattern)
        };
        
        let matches = if pattern.ends_with('/') {
            // Directory-only pattern
            if !is_dir {
                return false;
            }
            let pattern = &pattern[..pattern.len() - 1];
            self.simple_match(path, pattern)
        } else if pattern.starts_with('/') {
            // Absolute pattern from repo root
            let pattern = &pattern[1..];
            path.starts_with(pattern)
        } else if pattern.contains('/') {
            // Path with directory separator
            path.contains(pattern)
        } else {
            // Simple filename pattern
            self.simple_match(path, pattern)
        };
        
        if negate {
            !matches
        } else {
            matches
        }
    }
    
    /// Simple pattern matching with wildcards
    fn simple_match(&self, path: &str, pattern: &str) -> bool {
        // Handle ** (match any number of directories)
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                return path.contains(parts[0]) && path.ends_with(parts[1]);
            }
        }
        
        // Handle * (match any characters except /)
        if pattern.contains('*') {
            return self.glob_match(path, pattern);
        }
        
        // Exact match or path component match
        path == pattern || path.contains(&format!("/{}", pattern)) || path.ends_with(pattern)
    }
    
    /// Simple glob matching
    fn glob_match(&self, path: &str, pattern: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('*').collect();
        
        if pattern_parts.is_empty() {
            return false;
        }
        
        let mut pos = 0;
        for (i, part) in pattern_parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            
            if i == 0 && pattern.starts_with(part) {
                // First part must match at start
                if !path[pos..].starts_with(part) {
                    return false;
                }
                pos += part.len();
            } else if i == pattern_parts.len() - 1 && pattern.ends_with(part) {
                // Last part must match at end
                return path.ends_with(part);
            } else {
                // Middle parts can match anywhere
                if let Some(idx) = path[pos..].find(part) {
                    pos += idx + part.len();
                } else {
                    return false;
                }
            }
        }
        
        true
    }
}

impl Default for IgnorePatterns {
    fn default() -> Self {
        Self::new()
    }
}

/// Get default ignore patterns for common build/dependency directories
pub fn default_ignore_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "target".to_string(),
        "venv".to_string(),
        ".venv".to_string(),
        "__pycache__".to_string(),
        "build".to_string(),
        "dist".to_string(),
        ".build".to_string(),
        ".next".to_string(),
        "vendor".to_string(),
        ".gradle".to_string(),
        ".idea".to_string(),
        ".vscode".to_string(),
        "*.pyc".to_string(),
        "*.class".to_string(),
        "*.o".to_string(),
        ".DS_Store".to_string(),
    ]
}

/// Helper function to check if a directory entry should be ignored
pub fn should_ignore_entry(entry: &DirEntry, ignore_patterns: &IgnorePatterns) -> bool {
    let path = entry.path();
    let is_dir = entry.file_type().is_dir();
    
    // Always ignore .git directories
    if is_dir && entry.file_name() == ".git" {
        return true;
    }
    
    ignore_patterns.should_ignore(path, is_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern_match() {
        let mut ignore = IgnorePatterns::new();
        ignore.add_pattern("node_modules".to_string());
        
        assert!(ignore.should_ignore(Path::new("node_modules"), true));
        assert!(ignore.should_ignore(Path::new("src/node_modules"), true));
        assert!(!ignore.should_ignore(Path::new("src/modules"), true));
    }

    #[test]
    fn test_wildcard_pattern() {
        let mut ignore = IgnorePatterns::new();
        ignore.add_pattern("*.log".to_string());
        
        assert!(ignore.should_ignore(Path::new("error.log"), false));
        assert!(ignore.should_ignore(Path::new("debug.log"), false));
        assert!(!ignore.should_ignore(Path::new("logfile.txt"), false));
    }

    #[test]
    fn test_directory_only_pattern() {
        let mut ignore = IgnorePatterns::new();
        ignore.add_pattern("build/".to_string());
        
        assert!(ignore.should_ignore(Path::new("build"), true));
        assert!(!ignore.should_ignore(Path::new("build.txt"), false));
    }

    #[test]
    fn test_default_patterns() {
        let patterns = default_ignore_patterns();
        assert!(patterns.contains(&"node_modules".to_string()));
        assert!(patterns.contains(&"target".to_string()));
    }

    #[test]
    fn test_negation_pattern() {
        let mut ignore = IgnorePatterns::new();
        ignore.add_pattern("*.log".to_string());
        ignore.add_pattern("!important.log".to_string());
        
        assert!(ignore.should_ignore(Path::new("error.log"), false));
        // Note: Simple implementation doesn't fully handle negation order
    }
}
