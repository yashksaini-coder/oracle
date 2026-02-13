//! Installed crates registry inspector
//!
//! Scans ~/.cargo/registry to find and analyze installed crates

use crate::analyzer::{AnalyzedItem, RustAnalyzer};
use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Information about an installed crate
#[derive(Debug, Clone)]
pub struct InstalledCrate {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub readme: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
}

/// Registry of installed crates
pub struct CrateRegistry {
    crates: HashMap<String, Vec<InstalledCrate>>,
    registry_path: PathBuf,
}

impl CrateRegistry {
    /// Create a new registry scanner
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let registry_path = PathBuf::from(home).join(".cargo/registry/src");
        
        Self {
            crates: HashMap::new(),
            registry_path,
        }
    }

    /// Create with custom registry path
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            crates: HashMap::new(),
            registry_path: path,
        }
    }

    /// Scan the registry for installed crates
    pub fn scan(&mut self) -> Result<()> {
        self.crates.clear();

        if !self.registry_path.exists() {
            return Ok(());
        }

        // Registry has subdirectories for different indices (e.g., index.crates.io-xxx)
        for index_entry in fs::read_dir(&self.registry_path)? {
            let index_entry = index_entry?;
            let index_path = index_entry.path();
            
            if index_path.is_dir() {
                self.scan_index_directory(&index_path)?;
            }
        }

        Ok(())
    }

    fn scan_index_directory(&mut self, index_path: &Path) -> Result<()> {
        for entry in fs::read_dir(index_path)? {
            let entry = entry?;
            let crate_path = entry.path();
            
            if crate_path.is_dir() {
                if let Some(crate_info) = self.parse_crate_directory(&crate_path) {
                    self.crates
                        .entry(crate_info.name.clone())
                        .or_default()
                        .push(crate_info);
                }
            }
        }

        Ok(())
    }

    fn parse_crate_directory(&self, path: &Path) -> Option<InstalledCrate> {
        let dir_name = path.file_name()?.to_str()?;
        
        // Parse name and version from directory name (e.g., "serde-1.0.193")
        let (name, version) = Self::parse_crate_name_version(dir_name)?;
        
        // Try to read Cargo.toml for metadata
        let cargo_toml_path = path.join("Cargo.toml");
        let (description, authors, license, repository, documentation, keywords, categories) = 
            if cargo_toml_path.exists() {
                Self::parse_cargo_toml(&cargo_toml_path)
            } else {
                (None, vec![], None, None, None, vec![], vec![])
            };

        // Try to read README
        let readme = Self::find_and_read_readme(path);

        Some(InstalledCrate {
            name,
            version,
            path: path.to_path_buf(),
            readme,
            license,
            description,
            authors,
            repository,
            documentation,
            keywords,
            categories,
        })
    }

    fn parse_crate_name_version(dir_name: &str) -> Option<(String, String)> {
        // Find the last hyphen followed by a version number
        let mut last_version_start = None;
        let chars: Vec<char> = dir_name.chars().collect();
        
        for i in (0..chars.len()).rev() {
            if chars[i] == '-' && i + 1 < chars.len() {
                // Check if what follows looks like a version
                let rest = &dir_name[i + 1..];
                if rest.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    last_version_start = Some(i);
                    break;
                }
            }
        }

        let i = last_version_start?;
        let name = dir_name[..i].to_string();
        let version = dir_name[i + 1..].to_string();
        
        Some((name, version))
    }

    fn parse_cargo_toml(path: &Path) -> (Option<String>, Vec<String>, Option<String>, Option<String>, Option<String>, Vec<String>, Vec<String>) {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return (None, vec![], None, None, None, vec![], vec![]),
        };

        let toml: toml::Value = match content.parse() {
            Ok(t) => t,
            Err(_) => return (None, vec![], None, None, None, vec![], vec![]),
        };

        let package = toml.get("package");
        
        let description = package
            .and_then(|p| p.get("description"))
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let authors = package
            .and_then(|p| p.get("authors"))
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let license = package
            .and_then(|p| p.get("license"))
            .and_then(|l| l.as_str())
            .map(|s| s.to_string());

        let repository = package
            .and_then(|p| p.get("repository"))
            .and_then(|r| r.as_str())
            .map(|s| s.to_string());

        let documentation = package
            .and_then(|p| p.get("documentation"))
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let keywords = package
            .and_then(|p| p.get("keywords"))
            .and_then(|k| k.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let categories = package
            .and_then(|p| p.get("categories"))
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        (description, authors, license, repository, documentation, keywords, categories)
    }

    fn find_and_read_readme(path: &Path) -> Option<String> {
        let readme_names = ["README.md", "README", "Readme.md", "readme.md", "README.txt"];
        
        for name in &readme_names {
            let readme_path = path.join(name);
            if readme_path.exists() {
                if let Ok(content) = fs::read_to_string(&readme_path) {
                    // Truncate if too long, ensuring we don't split UTF-8 chars
                    let max_len = 10000;
                    if content.len() > max_len {
                        // Find valid UTF-8 boundary
                        let truncate_at = content
                            .char_indices()
                            .take_while(|(i, _)| *i < max_len)
                            .last()
                            .map(|(i, c)| i + c.len_utf8())
                            .unwrap_or(0);
                        return Some(content[..truncate_at].to_string() + "\n...[truncated]");
                    }
                    return Some(content);
                }
            }
        }
        None
    }

    /// Get all installed crate names
    pub fn crate_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.crates.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get all versions of a crate
    pub fn versions(&self, name: &str) -> Vec<&InstalledCrate> {
        self.crates
            .get(name)
            .map(|v| {
                let mut versions: Vec<_> = v.iter().collect();
                versions.sort_by(|a, b| {
                    // Sort by version descending (newest first)
                    b.version.cmp(&a.version)
                });
                versions
            })
            .unwrap_or_default()
    }

    /// Get the latest version of a crate
    pub fn latest(&self, name: &str) -> Option<&InstalledCrate> {
        self.versions(name).into_iter().next()
    }

    /// Get a specific version of a crate
    pub fn get(&self, name: &str, version: &str) -> Option<&InstalledCrate> {
        self.crates.get(name)?.iter().find(|c| c.version == version)
    }

    /// Check if a crate is installed
    pub fn is_installed(&self, name: &str) -> bool {
        self.crates.contains_key(name)
    }

    /// Get total number of installed crates
    pub fn count(&self) -> usize {
        self.crates.len()
    }

    /// Analyze a specific installed crate
    pub fn analyze_crate(&self, name: &str, version: Option<&str>) -> Result<Vec<AnalyzedItem>> {
        let crate_info = match version {
            Some(v) => self.get(name, v),
            None => self.latest(name),
        };

        let crate_info = match crate_info {
            Some(c) => c,
            None => return Ok(vec![]),
        };

        let analyzer = RustAnalyzer::new();
        let src_path = crate_info.path.join("src");
        
        let mut items = Vec::new();
        // Use crate name (with underscores instead of hyphens) as base module path
        let crate_module_name = name.replace('-', "_");
        
        if src_path.exists() {
            Self::analyze_directory(&analyzer, &src_path, &mut items, &crate_module_name)?;
        }

        Ok(items)
    }

    fn analyze_directory(analyzer: &RustAnalyzer, dir: &Path, items: &mut Vec<AnalyzedItem>, crate_name: &str) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::analyze_directory(analyzer, &path, items, crate_name)?;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                // Build module path: crate_name + path from src/
                let module_path = Self::build_module_path(&path, crate_name);
                if let Ok(file_items) = analyzer.analyze_file_with_module(&path, module_path) {
                    items.extend(file_items);
                }
            }
        }

        Ok(())
    }

    /// Build module path from file path relative to src/
    /// Returns clean path like ["serde", "de", "value"] not registry paths
    fn build_module_path(file_path: &Path, crate_name: &str) -> Vec<String> {
        let mut result = vec![crate_name.to_string()];
        
        // Convert to string and find src/ 
        let path_str = file_path.to_string_lossy();
        
        // Find the last "src/" in the path
        if let Some(src_idx) = path_str.rfind("/src/") {
            let after_src = &path_str[src_idx + 5..]; // Skip "/src/"
            
            for part in after_src.split('/') {
                if part.ends_with(".rs") {
                    let module = part.trim_end_matches(".rs");
                    // Skip lib.rs, main.rs, mod.rs
                    if module != "lib" && module != "main" && module != "mod" {
                        result.push(module.to_string());
                    }
                } else if !part.is_empty() {
                    result.push(part.to_string());
                }
            }
        }
        
        result
    }

    /// Search for crates by name
    pub fn search(&self, query: &str) -> Vec<&InstalledCrate> {
        let query_lower = query.to_lowercase();
        
        self.crates
            .values()
            .flatten()
            .filter(|c| c.name.to_lowercase().contains(&query_lower))
            .collect()
    }
}

impl Default for CrateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_crate_name_version() {
        let cases = vec![
            ("serde-1.0.193", Some(("serde".to_string(), "1.0.193".to_string()))),
            ("serde_json-1.0.108", Some(("serde_json".to_string(), "1.0.108".to_string()))),
            ("tokio-1.35.0", Some(("tokio".to_string(), "1.35.0".to_string()))),
            ("my-crate-name-0.1.0", Some(("my-crate-name".to_string(), "0.1.0".to_string()))),
        ];

        for (input, expected) in cases {
            assert_eq!(CrateRegistry::parse_crate_name_version(input), expected, "Failed for: {}", input);
        }
    }
}
