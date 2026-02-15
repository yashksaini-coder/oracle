//! Crate installation detection and management

use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of checking if a crate is available
#[derive(Debug, Clone)]
pub struct CrateAvailability {
    pub name: String,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub latest_version: Option<String>,
    pub is_local: bool,
    pub local_path: Option<PathBuf>,
}

impl CrateAvailability {
    /// Check if a crate needs installation
    pub fn needs_install(&self) -> bool {
        !self.is_installed && !self.is_local
    }

    /// Check if an update is available
    pub fn has_update(&self) -> bool {
        if let (Some(installed), Some(latest)) = (&self.installed_version, &self.latest_version) {
            installed != latest && version_compare(installed, latest).is_lt()
        } else {
            false
        }
    }

    /// Generate install command suggestion
    pub fn install_command(&self) -> String {
        if self.is_local {
            if let Some(path) = &self.local_path {
                return format!("cargo add --path {}", path.display());
            }
        }
        format!("cargo add {}", self.name)
    }
}

/// Compare semantic versions (simplified)
fn version_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.split('-').next()?.parse().ok())
            .collect()
    };

    let va = parse_version(a);
    let vb = parse_version(b);

    va.cmp(&vb)
}

/// Check if a crate is available in the local cargo cache
pub fn check_crate_in_registry(name: &str) -> Option<String> {
    // Check cargo's registry cache
    let cargo_home = dirs::home_dir()?.join(".cargo");
    let registry_src = cargo_home.join("registry").join("src");

    if !registry_src.exists() {
        return None;
    }

    // Look for the crate in any registry
    for entry in std::fs::read_dir(&registry_src).ok()? {
        let registry_path = entry.ok()?.path();
        for crate_entry in std::fs::read_dir(registry_path).ok()? {
            let crate_path = crate_entry.ok()?.path();
            let dir_name = crate_path.file_name()?.to_string_lossy();

            // Parse crate directory name (format: name-version)
            if let Some(crate_name) = dir_name.rsplit('-').next_back() {
                if crate_name == name {
                    // Extract version from directory name
                    let version = dir_name
                        .strip_prefix(name)
                        .and_then(|s| s.strip_prefix('-'))
                        .map(String::from);
                    return version;
                }
            }
        }
    }

    None
}

/// Check if a crate binary is installed
pub fn check_crate_binary(name: &str) -> bool {
    let cargo_bin = dirs::home_dir()
        .map(|h| h.join(".cargo").join("bin"))
        .unwrap_or_default();

    let binary_name = if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };

    cargo_bin.join(&binary_name).exists()
}

/// Get installed crate version from Cargo.lock if available
pub fn get_locked_version(project_path: &Path, crate_name: &str) -> Option<String> {
    let lock_path = project_path.join("Cargo.lock");
    if !lock_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&lock_path).ok()?;

    // Simple Cargo.lock parser - look for package entries
    let mut in_package = false;
    let mut current_name = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line == "[[package]]" {
            in_package = true;
            current_name.clear();
            continue;
        }

        if in_package {
            if line.starts_with("name = ") {
                current_name = line.strip_prefix("name = ")?.trim_matches('"').to_string();
            } else if line.starts_with("version = ") && current_name == crate_name {
                return Some(
                    line.strip_prefix("version = ")?
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.is_empty() || line.starts_with('[') {
                in_package = false;
            }
        }
    }

    None
}

/// Fetch latest version from crates.io (sync version)
pub fn fetch_latest_version_sync(crate_name: &str) -> Option<String> {
    // Use cargo search for simple lookup
    let output = Command::new("cargo")
        .args(["search", crate_name, "--limit", "1"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with(crate_name) {
            // Format: "crate_name = \"version\" # description"
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
    }

    None
}

/// Check overall crate availability
pub fn check_availability(name: &str, project_path: Option<&PathBuf>) -> CrateAvailability {
    let is_local = project_path.is_some();
    let local_path = project_path.cloned();

    // Check if installed in project
    let installed_version = project_path.and_then(|p| get_locked_version(p, name));

    // Check if in cargo registry cache
    let registry_version = check_crate_in_registry(name);

    CrateAvailability {
        name: name.to_string(),
        is_installed: installed_version.is_some() || registry_version.is_some(),
        installed_version: installed_version.or(registry_version),
        latest_version: None, // Filled async if needed
        is_local,
        local_path,
    }
}

/// Suggestions for common crate operations
#[derive(Debug, Clone)]
pub struct CrateSuggestion {
    pub action: SuggestedAction,
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestedAction {
    Install,
    Update,
    AddDependency,
    ViewDocs,
    ViewSource,
}

impl CrateSuggestion {
    pub fn install(name: &str) -> Self {
        Self {
            action: SuggestedAction::Install,
            command: format!("cargo add {}", name),
            description: format!("Add {} to your project dependencies", name),
        }
    }

    pub fn update(name: &str, version: &str) -> Self {
        Self {
            action: SuggestedAction::Update,
            command: format!("cargo update -p {}@{}", name, version),
            description: format!("Update {} to version {}", name, version),
        }
    }

    pub fn view_docs(name: &str) -> Self {
        Self {
            action: SuggestedAction::ViewDocs,
            command: format!("cargo doc -p {} --open", name),
            description: format!("Open documentation for {}", name),
        }
    }

    pub fn view_online_docs(name: &str) -> Self {
        Self {
            action: SuggestedAction::ViewDocs,
            command: format!("xdg-open https://docs.rs/{}", name),
            description: format!("Open docs.rs page for {}", name),
        }
    }
}

/// Generate suggestions for a crate
pub fn generate_suggestions(availability: &CrateAvailability) -> Vec<CrateSuggestion> {
    let mut suggestions = Vec::new();

    if availability.needs_install() {
        suggestions.push(CrateSuggestion::install(&availability.name));
    }

    if availability.has_update() {
        if let Some(ref version) = availability.latest_version {
            suggestions.push(CrateSuggestion::update(&availability.name, version));
        }
    }

    suggestions.push(CrateSuggestion::view_online_docs(&availability.name));

    if availability.is_installed {
        suggestions.push(CrateSuggestion::view_docs(&availability.name));
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert!(version_compare("1.0.0", "1.0.1").is_lt());
        assert!(version_compare("1.0.1", "1.0.0").is_gt());
        assert!(version_compare("1.0.0", "1.0.0").is_eq());
        assert!(version_compare("0.9.0", "1.0.0").is_lt());
    }
}
