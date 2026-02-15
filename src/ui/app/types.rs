//! Shared UI types: tabs, focus.

/// Active tab in the UI (Crates = project crates from Cargo.toml + open crate items)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Types,
    Functions,
    Modules,
    Crates,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Types, Tab::Functions, Tab::Modules, Tab::Crates]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Types => "Types",
            Tab::Functions => "Functions",
            Tab::Modules => "Modules",
            Tab::Crates => "Crates",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Types => 0,
            Tab::Functions => 1,
            Tab::Modules => 2,
            Tab::Crates => 3,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => Tab::Types,
            1 => Tab::Functions,
            2 => Tab::Modules,
            _ => Tab::Crates,
        }
    }

    pub fn next(&self) -> Self {
        Self::from_index(self.index() + 1)
    }

    pub fn prev(&self) -> Self {
        Self::from_index(self.index().wrapping_sub(1).min(3))
    }
}

/// Focus state for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Search,
    List,
    Inspector,
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Focus::Search => Focus::List,
            Focus::List => Focus::Inspector,
            Focus::Inspector => Focus::Search,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Focus::Search => Focus::Inspector,
            Focus::List => Focus::Search,
            Focus::Inspector => Focus::List,
        }
    }
}
