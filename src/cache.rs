//! Scan cache system for git-nexus.
//!
//! Caches repository scan results to speed up repeated scans.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache entry for a repository scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Last scan timestamp
    pub timestamp: SystemTime,
    /// Repository path
    pub path: PathBuf,
    /// Cached repository status data
    pub data: Vec<u8>,
    /// Hash of repository state (for invalidation)
    pub state_hash: String,
}

/// Scan cache manager
#[derive(Debug)]
pub struct ScanCache {
    cache_dir: PathBuf,
    entries: HashMap<PathBuf, CacheEntry>,
}

impl ScanCache {
    /// Create a new cache manager
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;
        
        let entries = Self::load_cache(&cache_dir)?;
        
        Ok(Self {
            cache_dir,
            entries,
        })
    }
    
    /// Get the cache directory path
    fn get_cache_dir() -> Result<PathBuf> {
        let cache_dir = if let Some(dir) = dirs::cache_dir() {
            dir.join("git-nexus")
        } else {
            PathBuf::from(".git-nexus-cache")
        };
        Ok(cache_dir)
    }
    
    /// Load cache from disk
    fn load_cache(cache_dir: &Path) -> Result<HashMap<PathBuf, CacheEntry>> {
        let cache_file = cache_dir.join("scan-cache.bin");
        
        if !cache_file.exists() {
            return Ok(HashMap::new());
        }
        
        let data = fs::read(&cache_file)?;
        let entries: HashMap<PathBuf, CacheEntry> = bincode::deserialize(&data)?;
        
        Ok(entries)
    }
    
    /// Save cache to disk
    fn save_cache(&self) -> Result<()> {
        let cache_file = self.cache_dir.join("scan-cache.bin");
        let data = bincode::serialize(&self.entries)?;
        fs::write(&cache_file, data)?;
        Ok(())
    }
    
    /// Get cached entry for a repository
    pub fn get(&self, repo_path: &Path) -> Option<&CacheEntry> {
        self.entries.get(repo_path)
    }
    
    /// Store an entry in the cache
    pub fn store(&mut self, repo_path: PathBuf, data: Vec<u8>, state_hash: String) -> Result<()> {
        let entry = CacheEntry {
            timestamp: SystemTime::now(),
            path: repo_path.clone(),
            data,
            state_hash,
        };
        
        self.entries.insert(repo_path, entry);
        self.save_cache()?;
        
        Ok(())
    }
    
    /// Check if a cache entry is still valid
    pub fn is_valid(&self, repo_path: &Path, max_age_secs: u64) -> bool {
        if let Some(entry) = self.get(repo_path) {
            if let Ok(elapsed) = entry.timestamp.elapsed() {
                if elapsed.as_secs() <= max_age_secs {
                    // Check if repository state has changed
                    if let Ok(current_hash) = Self::compute_state_hash(repo_path) {
                        return current_hash == entry.state_hash;
                    }
                }
            }
        }
        false
    }
    
    /// Compute a hash of the repository state for invalidation
    fn compute_state_hash(repo_path: &Path) -> Result<String> {
        // Simple hash based on HEAD ref and index modification time
        let git_dir = repo_path.join(".git");
        let head_file = git_dir.join("HEAD");
        let index_file = git_dir.join("index");
        
        let mut hash_data = String::new();
        
        if let Ok(head_content) = fs::read_to_string(&head_file) {
            hash_data.push_str(&head_content);
        }
        
        if let Ok(metadata) = fs::metadata(&index_file) {
            if let Ok(modified) = metadata.modified() {
                hash_data.push_str(&format!("{:?}", modified));
            }
        }
        
        // Simple hash (in production, use a proper hash function)
        Ok(format!("{:x}", hash_data.len() ^ hash_data.chars().count()))
    }
    
    /// Invalidate a specific cache entry
    pub fn invalidate(&mut self, repo_path: &Path) -> Result<()> {
        self.entries.remove(repo_path);
        self.save_cache()?;
        Ok(())
    }
    
    /// Clear all cache entries
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save_cache()?;
        Ok(())
    }
    
    /// Remove stale entries older than the given age
    pub fn prune(&mut self, max_age_secs: u64) -> Result<usize> {
        let now = SystemTime::now();
        let mut removed_count = 0;
        
        self.entries.retain(|_, entry| {
            if let Ok(elapsed) = now.duration_since(entry.timestamp) {
                if elapsed.as_secs() > max_age_secs {
                    removed_count += 1;
                    return false;
                }
            }
            true
        });
        
        self.save_cache()?;
        Ok(removed_count)
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_entries = self.entries.len();
        let total_size: usize = self.entries.values().map(|e| e.data.len()).sum();
        
        CacheStats {
            total_entries,
            total_size_bytes: total_size,
        }
    }
}

impl Default for ScanCache {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            cache_dir: PathBuf::from(".git-nexus-cache"),
            entries: HashMap::new(),
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
}

impl CacheStats {
    /// Format size in human-readable format
    pub fn size_human_readable(&self) -> String {
        let size = self.total_size_bytes as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        // Just test that cache can be created (may fail without dirs crate)
        let _ = ScanCache::default();
    }

    #[test]
    fn test_cache_stats_format() {
        let stats = CacheStats {
            total_entries: 10,
            total_size_bytes: 1024 * 1024, // 1 MB
        };
        
        let formatted = stats.size_human_readable();
        assert!(formatted.contains("MB"));
    }
}
