//! Theme and color customization for git-nexus output.
//!
//! Provides predefined color themes and custom color schemes for terminal output.

use colored::*;
use serde::{Deserialize, Serialize};

/// Available color themes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)] // Public API - used for theme selection
pub enum Theme {
    /// Default theme with vibrant colors
    Default,
    /// Minimalist theme with subtle colors
    Minimal,
    /// High contrast theme for accessibility
    HighContrast,
    /// Dark theme optimized for dark terminals
    Dark,
    /// Light theme optimized for light terminals
    Light,
    /// Monochrome theme (no colors)
    Monochrome,
}

impl Theme {
    /// Parse theme from string
    #[allow(dead_code)] // Public API - used for theme selection from config
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "default" => Some(Theme::Default),
            "minimal" => Some(Theme::Minimal),
            "high-contrast" | "highcontrast" => Some(Theme::HighContrast),
            "dark" => Some(Theme::Dark),
            "light" => Some(Theme::Light),
            "monochrome" | "mono" => Some(Theme::Monochrome),
            _ => None,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Default
    }
}

/// Theme-aware color scheme
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - used for theme-based coloring
pub struct ColorScheme {
    theme: Theme,
}

impl ColorScheme {
    /// Create a new color scheme with the given theme
    #[allow(dead_code)] // Public API - used for theme-based coloring
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }
    
    /// Get color for success/clean status
    #[allow(dead_code)] // Public API - used for theme-based coloring
    pub fn success(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.green(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_green().bold(),
            Theme::Dark => text.bright_green(),
            Theme::Light => text.green(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for error/dirty status
    #[allow(dead_code)]
    pub fn error(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.red(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_red().bold(),
            Theme::Dark => text.bright_red(),
            Theme::Light => text.red(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for warnings
    #[allow(dead_code)]
    pub fn warning(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.yellow(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_yellow().bold(),
            Theme::Dark => text.bright_yellow(),
            Theme::Light => text.yellow(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for info/neutral status
    #[allow(dead_code)]
    pub fn info(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_cyan(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_cyan().bold(),
            Theme::Dark => text.bright_blue(),
            Theme::Light => text.blue(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for paths
    #[allow(dead_code)]
    pub fn path(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_white().bold(),
            Theme::Minimal => text.white(),
            Theme::HighContrast => text.bright_white().bold(),
            Theme::Dark => text.bright_white(),
            Theme::Light => text.black(),
            Theme::Monochrome => text.bold(),
        }
    }
    
    /// Get color for branch names
    #[allow(dead_code)]
    pub fn branch(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_magenta(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_magenta().bold(),
            Theme::Dark => text.bright_magenta(),
            Theme::Light => text.magenta(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for counts/numbers
    #[allow(dead_code)]
    pub fn count(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_yellow(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_white().bold(),
            Theme::Dark => text.bright_yellow(),
            Theme::Light => text.yellow(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for dimmed/secondary text
    #[allow(dead_code)]
    pub fn dimmed(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_black(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.white(),
            Theme::Dark => text.bright_black(),
            Theme::Light => text.bright_black(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for health status - excellent
    #[allow(dead_code)]
    pub fn health_excellent(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_green(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_green().bold(),
            Theme::Dark => text.bright_green(),
            Theme::Light => text.green(),
            Theme::Monochrome => text.bold(),
        }
    }
    
    /// Get color for health status - good
    #[allow(dead_code)]
    pub fn health_good(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_blue(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_blue().bold(),
            Theme::Dark => text.bright_blue(),
            Theme::Light => text.blue(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for health status - fair
    #[allow(dead_code)]
    pub fn health_fair(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_yellow(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_yellow().bold(),
            Theme::Dark => text.bright_yellow(),
            Theme::Light => text.yellow(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for health status - poor
    #[allow(dead_code)]
    pub fn health_poor(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.yellow(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.yellow().bold(),
            Theme::Dark => text.yellow(),
            Theme::Light => text.yellow().bold(),
            Theme::Monochrome => text.normal(),
        }
    }
    
    /// Get color for health status - critical
    #[allow(dead_code)]
    pub fn health_critical(&self, text: &str) -> ColoredString {
        match self.theme {
            Theme::Default => text.bright_red(),
            Theme::Minimal => text.bright_black(),
            Theme::HighContrast => text.bright_red().bold(),
            Theme::Dark => text.bright_red(),
            Theme::Light => text.red().bold(),
            Theme::Monochrome => text.bold(),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::new(Theme::Default)
    }
}

/// List all available themes
pub fn list_themes() {
    println!("Available themes:");
    println!("  {} - Vibrant colors (default)", "default".bright_cyan());
    println!("  {} - Subtle, minimalist colors", "minimal".bright_black());
    println!("  {} - High contrast for accessibility", "high-contrast".bright_white().bold());
    println!("  {} - Optimized for dark terminals", "dark".bright_green());
    println!("  {} - Optimized for light terminals", "light".black());
    println!("  {} - No colors, text only", "monochrome".normal());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_str() {
        assert_eq!(Theme::from_str("default"), Some(Theme::Default));
        assert_eq!(Theme::from_str("minimal"), Some(Theme::Minimal));
        assert_eq!(Theme::from_str("high-contrast"), Some(Theme::HighContrast));
        assert_eq!(Theme::from_str("dark"), Some(Theme::Dark));
        assert_eq!(Theme::from_str("light"), Some(Theme::Light));
        assert_eq!(Theme::from_str("monochrome"), Some(Theme::Monochrome));
        assert_eq!(Theme::from_str("invalid"), None);
    }

    #[test]
    fn test_color_scheme_creation() {
        let scheme = ColorScheme::new(Theme::Default);
        assert_eq!(scheme.theme, Theme::Default);
    }

    #[test]
    fn test_monochrome_theme() {
        let scheme = ColorScheme::new(Theme::Monochrome);
        let text = scheme.success("test");
        // Monochrome should not add colors
        assert!(!text.to_string().contains("\x1b["));
    }
}
