//! Application settings and configuration

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub ui: UiSettings,
    pub analyzer: AnalyzerSettings,
    pub keybindings: KeybindingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub show_line_numbers: bool,
    pub vim_mode: bool,
    pub tab_width: usize,
    pub wrap_text: bool,
    pub accent_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerSettings {
    pub include_private: bool,
    pub include_tests: bool,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingSettings {
    pub quit: String,
    pub search: String,
    pub help: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub select: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui: UiSettings {
                theme: "default".into(),
                show_line_numbers: true,
                vim_mode: false,
                tab_width: 4,
                wrap_text: false,
                accent_color: "#4EBF71".into(),
            },
            analyzer: AnalyzerSettings {
                include_private: true,
                include_tests: false,
                max_depth: 10,
            },
            keybindings: KeybindingSettings {
                quit: "q".into(),
                search: "/".into(),
                help: "?".into(),
                next_tab: "Tab".into(),
                prev_tab: "Shift+Tab".into(),
                select: "Enter".into(),
            },
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let settings: Settings = serde_yaml::from_str(&content)?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::error::OracleError::Config("No config directory".into()))?;
        Ok(config_dir.join("oracle").join("config.yaml"))
    }
}
