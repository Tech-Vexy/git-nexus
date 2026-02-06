//! Configuration management for git-nexus.
//!
//! This module handles loading, saving, and managing configuration files for git-nexus.
//! Configuration files use TOML format and can be placed in multiple locations.
//!
//! # Configuration File Locations
//!
//! git-nexus looks for configuration files in the following order:
//! 1. `./.git-nexus.toml` (current directory)
//! 2. `~/.config/git-nexus/config.toml` (XDG config directory)
//! 3. `~/.git-nexus.toml` (home directory)
//!
//! # Example Configuration
//!
//! ```toml
//! scan_depth = 3
//! ignore_dirs = ["node_modules", "target", "venv"]
//!
//! [display]
//! show_branch = true
//! show_colors = true
//! default_verbose = false
//!
//! [github]
//! token = "ghp_xxxxx"
//! check_issues = true
//! check_prs = true
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure for git-nexus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum depth for directory traversal when scanning for repositories
    #[serde(default = "default_scan_depth")]
    pub scan_depth: usize,
    
    /// List of directory names to ignore during scanning (e.g., "node_modules", "target")
    #[serde(default = "default_ignore_dirs")]
    pub ignore_dirs: Vec<String>,
    
    /// Optional GitHub API configuration
    #[serde(default)]
    pub github: Option<GitHubConfig>,
    
    /// Display preferences
    #[serde(default)]
    pub display: DisplayConfig,
    
    /// Export settings
    #[serde(default)]
    pub export: ExportConfig,
}

/// Configuration for GitHub API integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub personal access token (optional)
    pub token: Option<String>,
    /// Whether to check for issues in repositories
    pub check_issues: bool,
    /// Whether to check for pull requests in repositories
    pub check_prs: bool,
}

/// Display preferences for terminal output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Show branch names in output
    #[serde(default = "default_true")]
    pub show_branch: bool,
    
    /// Use colored output in terminal
    #[serde(default = "default_true")]
    pub show_colors: bool,
    
    /// Use verbose mode by default
    #[serde(default)]
    pub default_verbose: bool,
}

/// Export configuration for HTML/CSV output.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportConfig {
    /// Default export format ("html" or "csv")
    pub default_format: Option<String>,
    /// Path to custom HTML template
    pub html_template: Option<PathBuf>,
}

fn default_scan_depth() -> usize {
    3
}

fn default_ignore_dirs() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "target".to_string(),
        "venv".to_string(),
        ".build".to_string(),
        "build".to_string(),
        "dist".to_string(),
        ".next".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_depth: default_scan_depth(),
            ignore_dirs: default_ignore_dirs(),
            github: None,
            display: DisplayConfig::default(),
            export: ExportConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_branch: true,
            show_colors: true,
            default_verbose: false,
        }
    }
}



impl Config {
    /// Loads configuration from the first available config file.
    ///
    /// Searches for configuration files in the following order:
    /// 1. `./.git-nexus.toml`
    /// 2. `~/.config/git-nexus/config.toml`
    /// 3. `~/.git-nexus.toml`
    ///
    /// # Returns
    ///
    /// Returns the loaded configuration, or a default configuration if no file is found.
    ///
    /// # Errors
    ///
    /// Returns an error if a config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let config_paths = vec![
            PathBuf::from(".git-nexus.toml"),
            dirs::home_dir()
                .map(|h| h.join(".config/git-nexus/config.toml"))
                .unwrap_or_default(),
            dirs::home_dir()
                .map(|h| h.join(".git-nexus.toml"))
                .unwrap_or_default(),
        ];

        for path in config_paths {
            if path.exists() {
                let contents = fs::read_to_string(&path)?;
                let config: Config = toml::from_str(&contents)?;
                return Ok(config);
            }
        }

        Ok(Config::default())
    }

    /// Saves the configuration to a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the configuration file should be saved
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or if serialization fails.
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    /// Creates an example configuration file with default settings.
    ///
    /// This is useful for users who want to customize their configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the example configuration should be created
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn create_example(path: &PathBuf) -> Result<()> {
        let example = Config::default();
        example.save(path)
    }
}

// Add dirs dependency helper
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.scan_depth, 3);
        assert!(config.ignore_dirs.contains(&"node_modules".to_string()));
        assert!(config.display.show_branch);
        assert!(config.display.show_colors);
        assert!(!config.display.default_verbose);
    }

    #[test]
    fn test_display_config_default() {
        let display = DisplayConfig::default();
        assert!(display.show_branch);
        assert!(display.show_colors);
        assert!(!display.default_verbose);
    }

    #[test]
    fn test_export_config_default() {
        let export = ExportConfig::default();
        assert!(export.default_format.is_none());
        assert!(export.html_template.is_none());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        let mut config = Config::default();
        config.scan_depth = 5;
        config.ignore_dirs = vec!["custom_dir".to_string()];
        
        // Save config
        config.save(&config_path).unwrap();
        assert!(config_path.exists());
        
        // Load and verify
        let contents = fs::read_to_string(&config_path).unwrap();
        let loaded: Config = toml::from_str(&contents).unwrap();
        
        assert_eq!(loaded.scan_depth, 5);
        assert_eq!(loaded.ignore_dirs, vec!["custom_dir".to_string()]);
    }

    #[test]
    fn test_config_create_example() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("example-config.toml");
        
        Config::create_example(&config_path).unwrap();
        assert!(config_path.exists());
        
        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(contents.contains("scan_depth"));
        assert!(contents.contains("ignore_dirs"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        
        assert!(toml_str.contains("scan_depth"));
        assert!(toml_str.contains("ignore_dirs"));
        assert!(toml_str.contains("[display]"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            scan_depth = 5
            ignore_dirs = ["test1", "test2"]

            [display]
            show_branch = false
            show_colors = true
            default_verbose = true
        "#;
        
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.scan_depth, 5);
        assert_eq!(config.ignore_dirs, vec!["test1", "test2"]);
        assert!(!config.display.show_branch);
        assert!(config.display.show_colors);
        assert!(config.display.default_verbose);
    }

    #[test]
    fn test_github_config() {
        let toml_str = r#"
            scan_depth = 3
            
            [github]
            token = "test_token"
            check_issues = true
            check_prs = false
        "#;
        
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.github.is_some());
        
        let github = config.github.unwrap();
        assert_eq!(github.token, Some("test_token".to_string()));
        assert!(github.check_issues);
        assert!(!github.check_prs);
    }

    #[test]
    fn test_export_config_serialization() {
        let mut config = Config::default();
        config.export.default_format = Some("html".to_string());
        config.export.html_template = Some(PathBuf::from("/path/to/template.html"));
        
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("default_format"));
        assert!(toml_str.contains("html"));
    }
}
