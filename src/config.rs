use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_scan_depth")]
    pub scan_depth: usize,
    
    #[serde(default = "default_ignore_dirs")]
    pub ignore_dirs: Vec<String>,
    
    #[serde(default)]
    pub github: Option<GitHubConfig>,
    
    #[serde(default)]
    pub display: DisplayConfig,
    
    #[serde(default)]
    pub export: ExportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub check_issues: bool,
    pub check_prs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_branch: bool,
    
    #[serde(default = "default_true")]
    pub show_colors: bool,
    
    #[serde(default)]
    pub default_verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub default_format: Option<String>,
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

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            default_format: None,
            html_template: None,
        }
    }
}

impl Config {
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

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

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
