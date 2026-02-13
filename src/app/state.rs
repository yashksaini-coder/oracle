//! Application state management

use crate::analyzer::{CrateInfo, CrateRegistry, DependencyAnalyzer, AnalyzedItem, InstalledCrate, RustAnalyzer};
use crate::config::Settings;
use crate::error::Result;
use crate::ui::{filter_candidates, CandidateKind, CompletionCandidate, Focus, Tab};
use crate::ui::theme::Theme;

use ratatui::widgets::ListState;
use std::path::PathBuf;

/// Main application state
pub struct App {
    // Analysis data
    pub items: Vec<AnalyzedItem>,
    pub filtered_items: Vec<usize>,
    pub crate_info: Option<CrateInfo>,
    pub dependency_tree: Vec<(String, usize)>,

    // Installed crates registry
    pub crate_registry: CrateRegistry,
    pub installed_crates_list: Vec<String>,
    pub selected_installed_crate: Option<InstalledCrate>,
    pub installed_crate_items: Vec<AnalyzedItem>,
    pub installed_crate_filtered: Vec<usize>,

    // UI state
    pub search_input: String,
    pub current_tab: Tab,
    pub focus: Focus,
    pub list_state: ListState,
    pub completion_selected: usize,
    pub show_completion: bool,
    pub show_help: bool,
    pub status_message: String,

    // Search
    pub candidates: Vec<CompletionCandidate>,
    pub filtered_candidates: Vec<CompletionCandidate>,

    // Config
    pub settings: Settings,
    pub theme: Theme,

    // Control
    pub should_quit: bool,
    pub project_path: Option<PathBuf>,
}

impl App {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            filtered_items: Vec::new(),
            crate_info: None,
            dependency_tree: Vec::new(),
            crate_registry: CrateRegistry::new(),
            installed_crates_list: Vec::new(),
            selected_installed_crate: None,
            installed_crate_items: Vec::new(),
            installed_crate_filtered: Vec::new(),
            search_input: String::new(),
            current_tab: Tab::default(),
            focus: Focus::default(),
            list_state: ListState::default(),
            completion_selected: 0,
            show_completion: false,
            show_help: false,
            status_message: String::from("Ready"),
            candidates: Vec::new(),
            filtered_candidates: Vec::new(),
            settings: Settings::default(),
            theme: Theme::default(),
            should_quit: false,
            project_path: None,
        }
    }

    /// Load settings from config file
    pub fn load_settings(&mut self) -> Result<()> {
        self.settings = Settings::load()?;
        Ok(())
    }

    /// Analyze a Rust project
    pub fn analyze_project(&mut self, path: &PathBuf) -> Result<()> {
        self.project_path = Some(path.clone());
        self.status_message = format!("Analyzing {}...", path.display());

        // Try to analyze Cargo.toml for dependencies
        let manifest_path = path.join("Cargo.toml");
        if manifest_path.exists() {
            match DependencyAnalyzer::from_manifest(&manifest_path) {
                Ok(analyzer) => {
                    if let Some(root) = analyzer.root_package() {
                        self.dependency_tree = analyzer.dependency_tree(&root.name);
                        self.crate_info = Some(root);
                    }
                }
                Err(e) => {
                    self.status_message = format!("Cargo analysis failed: {}", e);
                }
            }
        }

        // Analyze Rust source files
        let analyzer = RustAnalyzer::new().with_private(self.settings.analyzer.include_private);

        let src_path = path.join("src");
        if src_path.exists() {
            self.analyze_directory(&analyzer, &src_path)?;
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            self.items = analyzer.analyze_file(path)?;
        }

        self.update_candidates();
        self.filter_items();
        self.status_message = format!("Found {} items", self.items.len());

        Ok(())
    }

    fn analyze_directory(&mut self, analyzer: &RustAnalyzer, dir: &PathBuf) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.analyze_directory(analyzer, &path)?;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                match analyzer.analyze_file(&path) {
                    Ok(items) => self.items.extend(items),
                    Err(e) => {
                        // Log but continue
                        eprintln!("Warning: Failed to analyze {}: {}", path.display(), e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Update completion candidates from analyzed items
    pub fn update_candidates(&mut self) {
        self.candidates = self
            .items
            .iter()
            .map(|item| {
                let kind = match item {
                    AnalyzedItem::Function(_) => CandidateKind::Function,
                    AnalyzedItem::Struct(_) => CandidateKind::Struct,
                    AnalyzedItem::Enum(_) => CandidateKind::Enum,
                    AnalyzedItem::Trait(_) => CandidateKind::Trait,
                    AnalyzedItem::Module(_) => CandidateKind::Module,
                    AnalyzedItem::TypeAlias(_) => CandidateKind::Type,
                    AnalyzedItem::Const(_) | AnalyzedItem::Static(_) => CandidateKind::Const,
                    _ => CandidateKind::Other,
                };

                let secondary = item.documentation().map(|d| {
                    let first_line = d.lines().next().unwrap_or("");
                    if first_line.len() > 40 {
                        format!("{}...", &first_line[..37])
                    } else {
                        first_line.to_string()
                    }
                });

                CompletionCandidate {
                    primary: item.name().to_string(),
                    secondary,
                    kind,
                    score: 0,
                }
            })
            .collect();

        self.filtered_candidates = self.candidates.clone();
    }

    /// Filter items based on search input and current tab
    pub fn filter_items(&mut self) {
        let query = self.search_input.to_lowercase();

        // Handle installed crates tab separately
        if self.current_tab == Tab::InstalledCrates {
            self.filter_installed_crates();
            return;
        }

        self.filtered_items = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                // Filter by tab
                let tab_match = match self.current_tab {
                    Tab::Types => matches!(
                        item,
                        AnalyzedItem::Struct(_)
                            | AnalyzedItem::Enum(_)
                            | AnalyzedItem::TypeAlias(_)
                    ),
                    Tab::Functions => matches!(item, AnalyzedItem::Function(_)),
                    Tab::Modules => matches!(item, AnalyzedItem::Module(_)),
                    Tab::Dependencies => true, // Show all in dependencies tab
                    Tab::InstalledCrates => false, // Handled separately
                };

                // Filter by search
                let search_match =
                    query.is_empty() || item.name().to_lowercase().contains(&query);

                tab_match && search_match
            })
            .map(|(i, _)| i)
            .collect();

        // Reset selection if out of bounds
        if self.list_state.selected().is_some_and(|s| s >= self.filtered_items.len()) {
            self.list_state.select(Some(0));
        }

        // Update completion candidates
        self.filtered_candidates = filter_candidates(&self.candidates, &self.search_input);
        self.completion_selected = 0;
    }

    /// Scan for installed crates
    pub fn scan_installed_crates(&mut self) -> Result<()> {
        self.status_message = "Scanning installed crates...".to_string();
        self.crate_registry.scan()?;
        self.installed_crates_list = self.crate_registry.crate_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        self.status_message = format!("Found {} installed crates", self.installed_crates_list.len());
        Ok(())
    }

    /// Filter installed crates based on search
    /// Supports qualified path search like "serde::de::Deserialize"
    fn filter_installed_crates(&mut self) {
        let query = self.search_input.to_lowercase();
        
        if self.selected_installed_crate.is_some() {
            // Filter items within selected crate by qualified path or name
            self.installed_crate_filtered = self.installed_crate_items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    if query.is_empty() {
                        return true;
                    }
                    // Check if query contains :: for path matching
                    if query.contains("::") {
                        // Match against qualified path
                        item.qualified_name().to_lowercase().contains(&query) ||
                        // Or match partial module path
                        item.module_path().iter()
                            .any(|p| p.to_lowercase().contains(&query.replace("::", "")))
                    } else {
                        // Simple name match
                        item.name().to_lowercase().contains(&query)
                    }
                })
                .map(|(i, _)| i)
                .collect();
        }
        
        // Reset selection if out of bounds
        if self.list_state.selected().is_some_and(|s| s >= self.get_current_list_len()) {
            self.list_state.select(Some(0));
        }
    }

    /// Parse qualified path and navigate to crate + filter items
    /// E.g., "serde::de::Deserialize" -> select serde crate, filter for de::Deserialize
    pub fn search_qualified_path(&mut self) -> bool {
        let query = self.search_input.clone();
        let query = query.trim();
        
        // Check for qualified path (contains ::)
        if !query.contains("::") {
            return false;
        }
        
        let parts: Vec<&str> = query.split("::").collect();
        if parts.is_empty() {
            return false;
        }
        
        let crate_name = parts[0].to_string();
        
        // Check if crate exists
        let crate_exists = self.installed_crates_list.iter()
            .any(|name| name.to_lowercase() == crate_name.to_lowercase() ||
                        name.to_lowercase().replace('-', "_") == crate_name.to_lowercase());
        
        if !crate_exists {
            self.status_message = format!("Crate '{}' not found", crate_name);
            return false;
        }
        
        // Find actual crate name (might have hyphens)
        let actual_name = self.installed_crates_list.iter()
            .find(|name| name.to_lowercase() == crate_name.to_lowercase() ||
                         name.to_lowercase().replace('-', "_") == crate_name.to_lowercase())
            .cloned();
        
        // Select the crate if not already selected
        let already_selected = self.selected_installed_crate
            .as_ref()
            .map(|c| c.name.to_lowercase() == crate_name.to_lowercase())
            .unwrap_or(false);
        
        if !already_selected {
            if let Some(name) = actual_name {
                let _ = self.select_installed_crate(&name);
            }
        }
        
        // Set search to remaining path for filtering
        if parts.len() > 1 {
            // Keep the module path part for filtering
            self.search_input = parts[1..].join("::");
            self.filter_installed_crates();
        }
        
        true
    }

    /// Select an installed crate and analyze it
    pub fn select_installed_crate(&mut self, name: &str) -> Result<()> {
        if let Some(crate_info) = self.crate_registry.latest(name) {
            self.selected_installed_crate = Some(crate_info.clone());
            self.status_message = format!("Analyzing {}...", name);
            
            match self.crate_registry.analyze_crate(name, None) {
                Ok(items) => {
                    self.installed_crate_items = items;
                    self.installed_crate_filtered = (0..self.installed_crate_items.len()).collect();
                    self.status_message = format!("{}: {} items", name, self.installed_crate_items.len());
                }
                Err(e) => {
                    self.status_message = format!("Analysis failed: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Clear selected installed crate (go back to list)
    pub fn clear_installed_crate(&mut self) {
        self.selected_installed_crate = None;
        self.installed_crate_items.clear();
        self.installed_crate_filtered.clear();
        self.list_state.select(Some(0));
    }

    /// Get current list length based on tab and selection state
    pub fn get_current_list_len(&self) -> usize {
        if self.current_tab == Tab::InstalledCrates {
            if self.selected_installed_crate.is_some() {
                self.installed_crate_filtered.len()
            } else {
                self.installed_crates_list.len()
            }
        } else {
            self.filtered_items.len()
        }
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&AnalyzedItem> {
        if self.current_tab == Tab::InstalledCrates {
            if self.selected_installed_crate.is_some() {
                self.list_state
                    .selected()
                    .and_then(|i| self.installed_crate_filtered.get(i))
                    .and_then(|&idx| self.installed_crate_items.get(idx))
            } else {
                None
            }
        } else {
            self.list_state
                .selected()
                .and_then(|i| self.filtered_items.get(i))
                .and_then(|&idx| self.items.get(idx))
        }
    }

    /// Get filtered items as references
    pub fn get_filtered_items(&self) -> Vec<&AnalyzedItem> {
        if self.current_tab == Tab::InstalledCrates && self.selected_installed_crate.is_some() {
            self.installed_crate_filtered
                .iter()
                .filter_map(|&i| self.installed_crate_items.get(i))
                .collect()
        } else {
            self.filtered_items
                .iter()
                .filter_map(|&i| self.items.get(i))
                .collect()
        }
    }

    // Navigation methods
    pub fn next_item(&mut self) {
        let len = self.get_current_list_len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn prev_item(&mut self) {
        let len = self.get_current_list_len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.checked_sub(1).unwrap_or(len - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
        self.list_state.select(Some(0));
        self.filter_items();
        
        // Scan crates if switching to installed crates tab
        if self.current_tab == Tab::InstalledCrates && self.installed_crates_list.is_empty() {
            let _ = self.scan_installed_crates();
        }
    }

    pub fn prev_tab(&mut self) {
        self.current_tab = self.current_tab.prev();
        self.list_state.select(Some(0));
        self.filter_items();
        
        // Scan crates if switching to installed crates tab
        if self.current_tab == Tab::InstalledCrates && self.installed_crates_list.is_empty() {
            let _ = self.scan_installed_crates();
        }
    }

    pub fn next_focus(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn prev_focus(&mut self) {
        self.focus = self.focus.prev();
    }

    pub fn next_completion(&mut self) {
        if !self.filtered_candidates.is_empty() {
            self.completion_selected = (self.completion_selected + 1) % self.filtered_candidates.len();
        }
    }

    pub fn prev_completion(&mut self) {
        if !self.filtered_candidates.is_empty() {
            self.completion_selected = self
                .completion_selected
                .checked_sub(1)
                .unwrap_or(self.filtered_candidates.len() - 1);
        }
    }

    pub fn select_completion(&mut self) {
        if let Some(candidate) = self.filtered_candidates.get(self.completion_selected) {
            self.search_input = candidate.primary.clone();
            self.show_completion = false;
            self.filter_items();
        }
    }

    // Input handling
    pub fn on_char(&mut self, c: char) {
        self.search_input.push(c);
        self.filter_items();
        self.show_completion = self.search_input.len() >= 2;
    }

    pub fn on_backspace(&mut self) {
        self.search_input.pop();
        self.filter_items();
        self.show_completion = self.search_input.len() >= 2;
    }

    pub fn clear_search(&mut self) {
        self.search_input.clear();
        self.show_completion = false;
        self.filter_items();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
