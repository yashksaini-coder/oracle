//! Main Oracle TUI application

use crate::analyzer::AnalyzedItem;
use crate::analyzer::CrateInfo;
use crate::crates_io::CrateDocInfo;
use crate::ui::animation::AnimationState;
use crate::ui::dependency_view::{self, DependencyDocView, DependencyView};
use crate::ui::components::TabBar;
use crate::ui::search::{CompletionCandidate, SearchBar, SearchCompletion};
use crate::ui::inspector::InspectorPanel;
use crate::ui::theme::Theme;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
};

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

/// Main Oracle UI widget
pub struct OracleUi<'a> {
    // Data
    items: &'a [AnalyzedItem],
    /// Full list for impl lookup (project items or installed crate items)
    all_items_impl_lookup: Option<&'a [AnalyzedItem]>,
    filtered_items: &'a [&'a AnalyzedItem],
    candidates: &'a [CompletionCandidate],
    crate_info: Option<&'a CrateInfo>,
    dependency_tree: &'a [(String, usize)],
    /// Indices into dependency_tree for current search filter (Crates tab, top level). Empty = show all.
    filtered_dependency_indices: &'a [usize],
    /// Fetched doc for selected crate (when not root, not inside a crate)
    crate_doc: Option<&'a CrateDocInfo>,
    crate_doc_loading: bool,
    crate_doc_failed: bool,

    // Crates tab: when Enter on a crate, we show that crate's items
    selected_installed_crate: Option<&'a crate::analyzer::InstalledCrate>,
    installed_crate_items: &'a [&'a AnalyzedItem],

    // UI State
    search_input: &'a str,
    current_tab: Tab,
    focus: Focus,
    list_selected: Option<usize>,
    selected_item: Option<&'a AnalyzedItem>,
    completion_selected: usize,
    show_completion: bool,
    show_help: bool,
    show_settings: bool,
    status_message: &'a str,
    inspector_scroll: usize,

    // Animation
    animation: Option<&'a AnimationState>,

    // Theme
    theme: &'a Theme,
}

impl<'a> OracleUi<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            items: &[],
            all_items_impl_lookup: None,
            filtered_items: &[],
            candidates: &[],
            crate_info: None,
            dependency_tree: &[],
            filtered_dependency_indices: &[],
            crate_doc: None,
            crate_doc_loading: false,
            crate_doc_failed: false,
            selected_installed_crate: None,
            installed_crate_items: &[],
            search_input: "",
            current_tab: Tab::default(),
            focus: Focus::default(),
            list_selected: None,
            selected_item: None,
            completion_selected: 0,
            show_completion: false,
            show_help: false,
            show_settings: false,
            status_message: "",
            inspector_scroll: 0,
            animation: None,
            theme,
        }
    }

    pub fn items(mut self, items: &'a [AnalyzedItem]) -> Self {
        self.items = items;
        self
    }

    pub fn all_items_impl_lookup(mut self, items: Option<&'a [AnalyzedItem]>) -> Self {
        self.all_items_impl_lookup = items;
        self
    }

    pub fn filtered_items(mut self, items: &'a [&'a AnalyzedItem]) -> Self {
        self.filtered_items = items;
        self
    }

    pub fn selected_installed_crate(mut self, crate_info: Option<&'a crate::analyzer::InstalledCrate>) -> Self {
        self.selected_installed_crate = crate_info;
        self
    }

    pub fn installed_crate_items(mut self, items: &'a [&'a AnalyzedItem]) -> Self {
        self.installed_crate_items = items;
        self
    }

    pub fn list_selected(mut self, selected: Option<usize>) -> Self {
        self.list_selected = selected;
        self
    }

    pub fn candidates(mut self, candidates: &'a [CompletionCandidate]) -> Self {
        self.candidates = candidates;
        self
    }

    pub fn crate_info(mut self, info: Option<&'a CrateInfo>) -> Self {
        self.crate_info = info;
        self
    }

    pub fn dependency_tree(mut self, tree: &'a [(String, usize)]) -> Self {
        self.dependency_tree = tree;
        self
    }

    pub fn filtered_dependency_indices(mut self, indices: &'a [usize]) -> Self {
        self.filtered_dependency_indices = indices;
        self
    }

    pub fn crate_doc(mut self, doc: Option<&'a CrateDocInfo>) -> Self {
        self.crate_doc = doc;
        self
    }

    pub fn crate_doc_loading(mut self, loading: bool) -> Self {
        self.crate_doc_loading = loading;
        self
    }

    pub fn crate_doc_failed(mut self, failed: bool) -> Self {
        self.crate_doc_failed = failed;
        self
    }

    pub fn search_input(mut self, input: &'a str) -> Self {
        self.search_input = input;
        self
    }

    pub fn current_tab(mut self, tab: Tab) -> Self {
        self.current_tab = tab;
        self
    }

    pub fn focus(mut self, focus: Focus) -> Self {
        self.focus = focus;
        self
    }

    pub fn selected_item(mut self, item: Option<&'a AnalyzedItem>) -> Self {
        self.selected_item = item;
        self
    }

    pub fn completion_selected(mut self, index: usize) -> Self {
        self.completion_selected = index;
        self
    }

    pub fn show_completion(mut self, show: bool) -> Self {
        self.show_completion = show;
        self
    }

    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    pub fn show_settings(mut self, show: bool) -> Self {
        self.show_settings = show;
        self
    }

    pub fn status_message(mut self, msg: &'a str) -> Self {
        self.status_message = msg;
        self
    }

    pub fn inspector_scroll(mut self, scroll: usize) -> Self {
        self.inspector_scroll = scroll;
        self
    }

    pub fn animation_state(mut self, animation: &'a AnimationState) -> Self {
        self.animation = Some(animation);
        self
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        // Cyberpunk-style ASCII art banner
        let banner = vec![
            "  ⚡ ORACLE CODE INSPECTOR ⚡",
            " ╔═══════════════════════════════════════════════════════╗",
            " ║  Rust Code Analysis & Inspection Terminal Interface  ║",
            " ╚═══════════════════════════════════════════════════════╝",
        ];

        let crate_name = self.crate_info
            .map(|c| c.name.as_str())
            .unwrap_or("oracle");
        let version = self.crate_info
            .map(|c| format!("v{}", c.version))
            .unwrap_or_else(|| "v0.1.0".to_string());

        if area.height < 5 {
            let title = Line::from(vec![
                Span::styled("⚡ ", self.theme.style_accent()),
                Span::styled("Oracle", self.theme.style_accent_bold()),
                Span::styled(" :: ", self.theme.style_border_focused()),
                Span::styled(crate_name, self.theme.style_normal()),
                Span::styled(format!(" {}", version), self.theme.style_dim()),
                Span::styled(" :: Rust Inspector", self.theme.style_muted()),
            ]);
            let header = Paragraph::new(title).block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(self.theme.style_border()),
            );
            header.render(area, buf);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(58),
                Constraint::Min(20),
            ])
            .split(area);

        let banner_lines: Vec<Line> = banner
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let style = if idx == 0 || idx == 3 {
                    self.theme.style_accent_bold()
                } else {
                    self.theme.style_border()
                };
                Line::from(Span::styled(*line, style))
            })
            .collect();
        Paragraph::new(banner_lines).render(chunks[0], buf);

        let info_lines = vec![
            Line::from(vec![
                Span::styled("┌─ STATUS ", self.theme.style_border_focused()),
                Span::styled("─", self.theme.style_border()),
            ]),
            Line::from(vec![
                Span::styled("│ Target: ", self.theme.style_dim()),
                Span::styled(crate_name, self.theme.style_accent()),
                Span::styled(format!(" ({})", version), self.theme.style_muted()),
            ]),
            Line::from(vec![
                Span::styled("│ Mode: ", self.theme.style_dim()),
                Span::styled("ANALYSIS", self.theme.style_accent()),
            ]),
            Line::from(vec![
                Span::styled("└─ COMMANDS ", self.theme.style_border_focused()),
                Span::styled("─────────────────────────", self.theme.style_border()),
            ]),
            Line::from(vec![
                Span::styled("  [?]", self.theme.style_accent()),
                Span::styled(" help  ", self.theme.style_dim()),
                Span::styled("[q]", self.theme.style_accent()),
                Span::styled(" quit  ", self.theme.style_dim()),
                Span::styled("[⇆]", self.theme.style_accent()),
                Span::styled(" switch tabs", self.theme.style_dim()),
            ]),
        ];
        Paragraph::new(info_lines).render(chunks[1], buf);
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles: Vec<&str> = Tab::all().iter().map(|t| t.title()).collect();
        let tab_bar = TabBar::new(titles, self.theme).select(self.current_tab.index());
        tab_bar.render(area, buf);
    }

    fn render_search(&self, area: Rect, buf: &mut Buffer) {
        // Placeholder and completion context only for the active tab
        let placeholder = match self.current_tab {
            Tab::Types => "Search types... (struct, enum, type)",
            Tab::Functions => "Search functions...",
            Tab::Modules => "Search modules...",
            Tab::Crates => {
                if self.selected_installed_crate.is_some() {
                    "Filter items... (e.g., de::Deserialize)"
                } else {
                    "Search crates... (filter by name)"
                }
            }
        };

        let search = SearchBar::new(self.search_input, self.theme)
            .focused(self.focus == Focus::Search)
            .placeholder(placeholder);

        search.render(area, buf);
    }

    fn render_completion(&self, search_area: Rect, buf: &mut Buffer) {
        if !self.show_completion || self.candidates.is_empty() {
            return;
        }

        let max_height = 12.min(self.candidates.len() as u16 + 2);
        let dropdown_area = Rect {
            x: search_area.x + 2,
            y: search_area.y + search_area.height,
            width: search_area.width.saturating_sub(4).min(60),
            height: max_height,
        };

        let completion = SearchCompletion::new(self.candidates, self.theme)
            .selected(self.completion_selected)
            .filter(self.search_input)
            .max_visible(10);

        completion.render(dropdown_area, buf);
    }

    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{List, ListItem};

        // Crates tab: crate list (filtered by search) or (when Enter on a crate) that crate's items
        if self.current_tab == Tab::Crates {
            if self.selected_installed_crate.is_some() {
                self.render_installed_crates_list(area, buf);
            } else {
                self.render_dependencies_list(area, buf);
            }
            return;
        }

        let selected = self.list_selected;
        
        // Get animation highlight value
        let highlight_intensity = self.animation
            .map(|a| a.selection_highlight)
            .unwrap_or(1.0);
        
        // Calculate visible area (account for borders)
        let visible_height = area.height.saturating_sub(2) as usize;
        let total_items = self.filtered_items.len();
        
        // Calculate scroll offset to keep selection visible
        let scroll_offset = if let Some(sel) = selected {
            if visible_height == 0 {
                0
            } else if sel >= visible_height {
                // Keep selection near bottom with some context
                sel.saturating_sub(visible_height - 1)
            } else {
                0
            }
        } else {
            0
        };
        
        let items: Vec<ListItem> = self
            .filtered_items
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(idx, item)| {
                let kind_style = match item.kind() {
                    "fn" => self.theme.style_function(),
                    "struct" | "enum" | "type" => self.theme.style_type(),
                    "trait" => self.theme.style_keyword(),
                    "mod" => self.theme.style_accent(),
                    "const" | "static" => self.theme.style_string(),
                    _ => self.theme.style_dim(),
                };

                let is_selected = Some(idx) == selected;
                let base_style = if is_selected {
                    // Animate the selection highlight
                    if highlight_intensity < 1.0 {
                        self.theme.style_selected().add_modifier(Modifier::BOLD)
                    } else {
                        self.theme.style_selected()
                    }
                } else {
                    Style::default()
                };

                let prefix = if is_selected { "❱❱ " } else { "  " };

                // Show visibility indicator with cyberpunk styling
                let vis = item.visibility()
                    .map(|v| match v {
                        crate::analyzer::Visibility::Public => "●",
                        crate::analyzer::Visibility::Crate => "◐",
                        _ => "○",
                    })
                    .unwrap_or("○");

                // Show short name for local project (no module prefix needed)
                // For crates tab, module path will be shown
                let display_name = item.name().to_string();

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, self.theme.style_accent()),
                    Span::styled(format!("[{}]", vis), self.theme.style_dim()),
                    Span::raw(" "),
                    Span::styled(format!("{:6}", item.kind()), kind_style),
                    Span::raw(" "),
                    Span::styled(display_name, self.theme.style_normal()),
                ]))
                .style(base_style)
            })
            .collect();

        let border_style = if self.focus == Focus::List {
            self.theme.style_border_focused()
        } else {
            self.theme.style_border()
        };

        // Show item count, filter info, and scroll position
        let scroll_indicator = if total_items > visible_height {
            let pos = selected.unwrap_or(0) + 1;
            format!(" ◇ {}/{}", pos, total_items)
        } else {
            String::new()
        };
        
        let title = if self.search_input.is_empty() {
            format!(" ◆ RESULTS ({}){}  ", self.filtered_items.len(), scroll_indicator)
        } else {
            format!(" ◆ FILTERED ({}/{}){}  ", self.filtered_items.len(), self.items.len(), scroll_indicator)
        };

        let list_area = Rect {
            width: area.width.saturating_sub(1),
            ..area
        };
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(title),
            )
            .highlight_style(self.theme.style_selected())
            .highlight_symbol("❱ ");

        Widget::render(list, list_area, buf);

        if total_items > visible_height {
            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y,
                width: 1,
                height: area.height,
            };
            let mut state = ScrollbarState::new(total_items).position(scroll_offset);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            StatefulWidget::render(scrollbar, scrollbar_area, buf, &mut state);
        }
    }

    fn render_dependencies_list(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{List, ListItem};

        let border_style = if self.focus == Focus::List {
            self.theme.style_border_focused()
        } else {
            self.theme.style_border()
        };
        let visible_height = area.height.saturating_sub(2) as usize;
        let selected = self.list_selected.unwrap_or(0);
        let (items_slice, total) = if self.filtered_dependency_indices.is_empty() && !self.dependency_tree.is_empty() {
            (&[][..], 1usize)
        } else if self.dependency_tree.is_empty() {
            (&[][..], 1usize)
        } else {
            let indices = self.filtered_dependency_indices;
            (indices, indices.len())
        };
        let total = total.max(1);
        let scroll_offset = if visible_height == 0 {
            0
        } else if selected >= visible_height {
            selected.saturating_sub(visible_height - 1)
        } else {
            0
        };

        let items: Vec<ListItem> = if self.dependency_tree.is_empty() {
            let is_selected = selected == 0;
            let style = if is_selected { self.theme.style_selected() } else { Style::default() };
            vec![ListItem::new(Line::from(vec![
                Span::styled(if is_selected { "▸ " } else { "  " }, self.theme.style_accent()),
                Span::styled("○ ", self.theme.style_muted()),
                Span::styled("No Cargo project", self.theme.style_dim()),
            ])).style(style)]
        } else if items_slice.is_empty() {
            let is_selected = selected == 0;
            let style = if is_selected { self.theme.style_selected() } else { Style::default() };
            vec![ListItem::new(Line::from(vec![
                Span::styled(if is_selected { "▸ " } else { "  " }, self.theme.style_accent()),
                Span::styled("○ ", self.theme.style_muted()),
                Span::styled("No matches for search", self.theme.style_dim()),
            ])).style(style)]
        } else {
            items_slice
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(visible_height)
                .map(|(display_idx, &tree_idx)| {
                    let (name, _) = &self.dependency_tree[tree_idx];
                    let is_selected = Some(display_idx) == self.list_selected;
                    let style = if is_selected { self.theme.style_selected() } else { Style::default() };
                    ListItem::new(Line::from(vec![
                        Span::styled(if is_selected { "▸ " } else { "  " }, self.theme.style_accent()),
                        Span::styled("📦 ", self.theme.style_dim()),
                        Span::styled(name.clone(), self.theme.style_normal()),
                    ])).style(style)
                })
                .collect()
        };

        let scroll_info = if total > visible_height {
            format!(" [{}/{}]", selected + 1, total)
        } else {
            String::new()
        };
        let title = format!(" Crates ({}){} ", total, scroll_info);
        let list_area = Rect {
            width: area.width.saturating_sub(1),
            ..area
        };
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(title),
            )
            .highlight_style(self.theme.style_selected())
            .highlight_symbol("▸ ");
        Widget::render(list, list_area, buf);

        if total > visible_height {
            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y,
                width: 1,
                height: area.height,
            };
            let mut state = ScrollbarState::new(total).position(selected);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            StatefulWidget::render(scrollbar, scrollbar_area, buf, &mut state);
        }
    }

    fn render_installed_crates_list(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{List, ListItem};

        let selected = self.list_selected;
        let border_style = if self.focus == Focus::List {
            self.theme.style_border_focused()
        } else {
            self.theme.style_border()
        };
        
        // Calculate visible area (account for borders)
        let visible_height = area.height.saturating_sub(2) as usize;

        if let Some(crate_info) = self.selected_installed_crate {
            // Calculate scroll offset for crate items
            let total_items = self.installed_crate_items.len();
            let scroll_offset = if let Some(sel) = selected {
                if visible_height == 0 {
                    0
                } else if sel >= visible_height {
                    sel.saturating_sub(visible_height - 1)
                } else {
                    0
                }
            } else {
                0
            };
            
            // Show items within selected crate
            let items: Vec<ListItem> = self
                .installed_crate_items
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(visible_height)
                .map(|(idx, item)| {
                    let kind_style = match item.kind() {
                        "fn" => self.theme.style_function(),
                        "struct" | "enum" | "type" => self.theme.style_type(),
                        "trait" => self.theme.style_keyword(),
                        "mod" => self.theme.style_accent(),
                        "const" | "static" => self.theme.style_string(),
                        _ => self.theme.style_dim(),
                    };

                    let is_selected = Some(idx) == selected;
                    let base_style = if is_selected {
                        self.theme.style_selected()
                    } else {
                        Style::default()
                    };

                    let prefix = if is_selected { "▸ " } else { "  " };
                    let vis = item.visibility()
                        .map(|v| match v {
                            crate::analyzer::Visibility::Public => "●",
                            crate::analyzer::Visibility::Crate => "◐",
                            _ => "○",
                        })
                        .unwrap_or("○");

                    // Show item name with short module hint
                    // module_path is like ["crate_name", "submod", "subsubmod"]
                    let module_path = item.module_path();
                    let display_name = if module_path.len() > 2 {
                        // Has submodules - show last submodule as hint
                        let last_mod = &module_path[module_path.len() - 1];
                        format!("{}::{}", last_mod, item.name())
                    } else if module_path.len() == 2 {
                        // In a direct submodule
                        format!("{}::{}", module_path[1], item.name())
                    } else {
                        // Root level
                        item.name().to_string()
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, self.theme.style_accent()),
                        Span::styled(vis, self.theme.style_dim()),
                        Span::raw(" "),
                        Span::styled(format!("{:6} ", item.kind()), kind_style),
                        Span::styled(display_name, self.theme.style_normal()),
                    ]))
                    .style(base_style)
                })
                .collect();

            // Scroll indicator
            let scroll_info = if total_items > visible_height {
                format!(" [{}/{}]", selected.unwrap_or(0) + 1, total_items)
            } else {
                String::new()
            };
            
            let title = format!(" 📦 {} v{} ({} items){} [Esc] ", 
                crate_info.name, crate_info.version, total_items, scroll_info);

            let list_area = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title(title),
                )
                .highlight_style(self.theme.style_selected());

            Widget::render(list, list_area, buf);

            if total_items > visible_height {
                let scrollbar_area = Rect {
                    x: area.x + area.width.saturating_sub(1),
                    y: area.y,
                    width: 1,
                    height: area.height,
                };
                let mut state = ScrollbarState::new(total_items).position(scroll_offset);
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"));
                StatefulWidget::render(scrollbar, scrollbar_area, buf, &mut state);
            }
        }
    }

    fn render_inspector(&self, area: Rect, buf: &mut Buffer) {
        if self.current_tab == Tab::Crates && self.selected_installed_crate.is_some() {
            // Dependencies: inside a crate — show crate info or selected item
            if self.selected_item.is_none() {
                self.render_installed_crate_info(area, buf);
            } else {
                let inspector = InspectorPanel::new(self.theme)
                    .item(self.selected_item)
                    .all_items(self.all_items_impl_lookup)
                    .focused(self.focus == Focus::Inspector)
                    .scroll(self.inspector_scroll);
                inspector.render(area, buf);
            }
        } else if self.current_tab == Tab::Crates {
            let root_name = self.dependency_tree.first().map(|(n, _)| n.as_str());
            let selected_name = self
                .list_selected
                .and_then(|i| self.filtered_dependency_indices.get(i).copied())
                .and_then(|tree_idx| self.dependency_tree.get(tree_idx))
                .map(|(n, _)| n.as_str());
            let showing_root = root_name.zip(selected_name).map(|(r, s)| r == s).unwrap_or(true);
            if showing_root {
                let dep_view = DependencyView::new(self.theme)
                    .crate_info(self.crate_info)
                    .focused(self.focus == Focus::Inspector)
                    .scroll(self.inspector_scroll)
                    .show_browser_hint(true);
                dep_view.render(area, buf);
            } else if let Some(name) = selected_name {
                if self.crate_doc_loading {
                    dependency_view::render_doc_loading(self.theme, area, buf, name);
                } else if self.crate_doc_failed {
                    dependency_view::render_doc_failed(self.theme, area, buf, name);
                } else if let Some(doc) = self.crate_doc {
                    let doc_view = DependencyDocView::new(self.theme, doc)
                        .focused(self.focus == Focus::Inspector)
                        .scroll(self.inspector_scroll)
                        .show_browser_hint(true);
                    doc_view.render(area, buf);
                } else {
                    dependency_view::render_doc_loading(self.theme, area, buf, name);
                }
            } else {
                let dep_view = DependencyView::new(self.theme)
                    .crate_info(self.crate_info)
                    .focused(self.focus == Focus::Inspector)
                    .scroll(self.inspector_scroll)
                    .show_browser_hint(true);
                dep_view.render(area, buf);
            }
        } else {
            let inspector = InspectorPanel::new(self.theme)
                .item(self.selected_item)
                .all_items(self.all_items_impl_lookup)
                .focused(self.focus == Focus::Inspector)
                .scroll(self.inspector_scroll);
            inspector.render(area, buf);
        }
    }

    fn render_installed_crate_info(&self, area: Rect, buf: &mut Buffer) {
        let crate_info = match self.selected_installed_crate {
            Some(c) => c,
            None => return,
        };

        let border_style = if self.focus == Focus::Inspector {
            self.theme.style_border_focused()
        } else {
            self.theme.style_border()
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("📦 ", Style::default()),
                Span::styled(&crate_info.name, self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED)),
                Span::raw(" "),
                Span::styled(format!("v{}", crate_info.version), self.theme.style_muted()),
            ]),
            Line::from(""),
        ];

        // Description
        if let Some(ref desc) = crate_info.description {
            lines.push(Line::from(vec![
                Span::styled("━━━ ", self.theme.style_muted()),
                Span::styled("Description", self.theme.style_accent()),
                Span::styled(" ━━━", self.theme.style_muted()),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(desc.clone(), self.theme.style_normal()),
            ]));
            lines.push(Line::from(""));
        }

        // License
        if let Some(ref license) = crate_info.license {
            lines.push(Line::from(vec![
                Span::styled("  License: ", self.theme.style_dim()),
                Span::styled(license.clone(), self.theme.style_normal()),
            ]));
        }

        // Authors
        if !crate_info.authors.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  Authors: ", self.theme.style_dim()),
                Span::styled(crate_info.authors.join(", "), self.theme.style_normal()),
            ]));
        }

        // Repository
        if let Some(ref repo) = crate_info.repository {
            lines.push(Line::from(vec![
                Span::styled("  Repository: ", self.theme.style_dim()),
                Span::styled(repo.clone(), self.theme.style_accent()),
            ]));
        }

        // Documentation
        if let Some(ref docs) = crate_info.documentation {
            lines.push(Line::from(vec![
                Span::styled("  Docs: ", self.theme.style_dim()),
                Span::styled(docs.clone(), self.theme.style_accent()),
            ]));
        }

        // Keywords
        if !crate_info.keywords.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("━━━ ", self.theme.style_muted()),
                Span::styled("Keywords", self.theme.style_accent()),
                Span::styled(" ━━━", self.theme.style_muted()),
            ]));
            lines.push(Line::from(""));
            let keywords: Vec<Span> = crate_info.keywords.iter()
                .map(|k| Span::styled(format!(" {} ", k), self.theme.style_keyword()))
                .collect();
            lines.push(Line::from(vec![Span::raw("  ")]).patch_style(Style::default()));
            for kw in keywords {
                lines.push(Line::from(vec![Span::raw("  "), kw]));
            }
        }

        // Categories
        if !crate_info.categories.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("━━━ ", self.theme.style_muted()),
                Span::styled("Categories", self.theme.style_accent()),
                Span::styled(" ━━━", self.theme.style_muted()),
            ]));
            lines.push(Line::from(""));
            for cat in &crate_info.categories {
                lines.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(cat.clone(), self.theme.style_type()),
                ]));
            }
        }

        // Path
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("━━━ ", self.theme.style_muted()),
            Span::styled("Location", self.theme.style_accent()),
            Span::styled(" ━━━", self.theme.style_muted()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  📁 "),
            Span::styled(crate_info.path.display().to_string(), self.theme.style_muted()),
        ]));

        // Items count
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("━━━ ", self.theme.style_muted()),
            Span::styled("Analysis", self.theme.style_accent()),
            Span::styled(" ━━━", self.theme.style_muted()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{} items found", self.installed_crate_items.len()), self.theme.style_normal()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("Select an item to view details", self.theme.style_muted()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" [o] ", self.theme.style_accent()),
            Span::styled("docs.rs  ", self.theme.style_dim()),
            Span::styled(" [c] ", self.theme.style_accent()),
            Span::styled("crates.io", self.theme.style_dim()),
        ]));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::block::BorderType::Rounded)
            .border_style(border_style)
            .title(" ◇ Crate Info ");

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false });

        paragraph.render(area, buf);
    }

    fn render_status(&self, area: Rect, buf: &mut Buffer) {
        // Focus indicator
        let focus_indicator = match self.focus {
            Focus::Search => ("🔍", "Search"),
            Focus::List => ("📋", "List"),
            Focus::Inspector => ("🔬", "Inspector"),
        };

        // Different footer for different contexts
        let status_line = if self.current_tab == Tab::Crates && self.selected_installed_crate.is_some() {
            // Dependencies: inside a crate
            if let Some(crate_info) = self.selected_installed_crate {
                let selection_info = if let Some(selected) = self.list_selected {
                    format!("[{}/{}]", selected + 1, self.installed_crate_items.len())
                } else {
                    format!("[0/{}]", self.installed_crate_items.len())
                };
                Line::from(vec![
                    Span::styled(" 📦 ", self.theme.style_accent()),
                    Span::styled(&crate_info.name, self.theme.style_normal()),
                    Span::styled(format!(" v{}", crate_info.version), self.theme.style_dim()),
                    Span::styled(" │ ", self.theme.style_muted()),
                    Span::styled(selection_info, self.theme.style_muted()),
                    Span::styled(" │ ", self.theme.style_muted()),
                    Span::styled(focus_indicator.0, self.theme.style_accent()),
                    Span::styled(format!(" {} ", focus_indicator.1), self.theme.style_dim()),
                    Span::styled("│ Esc: back", self.theme.style_muted()),
                ])
            } else {
                Line::from(vec![
                    Span::styled(" 📦 Crates ", self.theme.style_accent()),
                    Span::styled(focus_indicator.0, self.theme.style_accent()),
                    Span::styled(format!(" {} ", focus_indicator.1), self.theme.style_dim()),
                    Span::styled("│ Esc: back", self.theme.style_muted()),
                ])
            }
        } else if self.current_tab == Tab::Crates {
            Line::from(vec![
                Span::styled("Commands: ", self.theme.style_dim()),
                Span::styled("Tab", self.theme.style_accent()),
                Span::styled(" focus ", self.theme.style_muted()),
                Span::styled("↑/↓", self.theme.style_accent()),
                Span::styled(" list ",
                self.theme.style_muted()),
                Span::styled("Enter", self.theme.style_accent()),
                Span::styled(" open crate ",
                self.theme.style_muted()),
                Span::styled("[o]", self.theme.style_accent()),
                Span::styled(" docs.rs ", self.theme.style_muted()),
                Span::styled("[c]", self.theme.style_accent()),
                Span::styled(" crates.io ", self.theme.style_muted()),
                Span::styled("│ ", self.theme.style_dim()),
                Span::styled("📦", self.theme.style_accent()),
                Span::styled(format!(" Crates ({}) ", self.filtered_dependency_indices.len()), self.theme.style_normal()),
            ])
        } else if !self.status_message.is_empty() {
            // Custom status message
            Line::from(vec![
                Span::styled(format!(" {} ", self.status_message), self.theme.style_string()),
                Span::styled("│ ", self.theme.style_muted()),
                Span::styled(focus_indicator.0, self.theme.style_accent()),
                Span::styled(format!(" {}", focus_indicator.1), self.theme.style_dim()),
            ])
        } else {
            // Normal footer with counts
            let (fn_count, struct_count, _enum_count, _trait_count) = self.items.iter().fold(
                (0usize, 0usize, 0usize, 0usize),
                |(f, s, e, t), item| match item.kind() {
                    "fn" => (f + 1, s, e, t),
                    "struct" => (f, s + 1, e, t),
                    "enum" => (f, s, e + 1, t),
                    "trait" => (f, s, e, t + 1),
                    _ => (f, s, e, t),
                },
            );
            
            let selection_info = if let Some(selected) = self.list_selected {
                format!("[{}/{}]", selected + 1, self.filtered_items.len())
            } else {
                format!("[0/{}]", self.filtered_items.len())
            };

            Line::from(vec![
                Span::styled("Commands: ", self.theme.style_dim()),
                Span::styled("Tab", self.theme.style_accent()),
                Span::styled(" focus ", self.theme.style_muted()),
                Span::styled("↑/↓", self.theme.style_accent()),
                Span::styled(" list ", self.theme.style_muted()),
                Span::styled("1-4", self.theme.style_accent()),
                Span::styled(" tabs ", self.theme.style_muted()),
                Span::styled("? ", self.theme.style_accent()),
                Span::styled("help ", self.theme.style_muted()),
                Span::styled("q ", self.theme.style_accent()),
                Span::styled("quit ", self.theme.style_muted()),
                Span::styled("│ ", self.theme.style_dim()),
                Span::styled("fn:", self.theme.style_function()),
                Span::styled(format!("{} ", fn_count), self.theme.style_normal()),
                Span::styled("struct:", self.theme.style_type()),
                Span::styled(format!("{} ", struct_count), self.theme.style_normal()),
                Span::styled("selection ", self.theme.style_muted()),
                Span::styled(selection_info, self.theme.style_dim()),
            ])
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.style_border())
            .style(Style::default().bg(self.theme.bg_panel));
        let inner = block.inner(area);
        block.render(area, buf);
        let paragraph = Paragraph::new(status_line)
            .alignment(ratatui::layout::Alignment::Center);
        paragraph.render(inner, buf);
    }

    fn render_settings_overlay(&self, area: Rect, buf: &mut Buffer) {
        if !self.show_settings {
            return;
        }

        let w = 48.min(area.width.saturating_sub(4));
        let h = 10.min(area.height.saturating_sub(4));
        let settings_area = Rect {
            x: area.x + (area.width - w) / 2,
            y: area.y + (area.height - h) / 2,
            width: w,
            height: h,
        };

        Clear.render(settings_area, buf);

        let text = vec![
            Line::from(Span::styled(
                " Settings ",
                self.theme.style_accent_bold(),
            )),
            Line::from(""),
            Line::from(Span::styled("Theme", self.theme.style_dim())),
            Line::from(vec![
                Span::raw("  Press "),
                Span::styled("t", self.theme.style_accent()),
                Span::raw(" to cycle theme"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc or S to close",
                self.theme.style_muted(),
            )),
        ];

        let block = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.style_border_focused())
                .title(" Settings ")
                .style(Style::default().bg(self.theme.bg_panel)),
        );
        block.render(settings_area, buf);
    }

    fn render_help_overlay(&self, area: Rect, buf: &mut Buffer) {
        if !self.show_help {
            return;
        }

        let help_width = 55.min(area.width.saturating_sub(4));
        let help_height = 24.min(area.height.saturating_sub(4));
        let help_area = Rect {
            x: area.x + (area.width - help_width) / 2,
            y: area.y + (area.height - help_height) / 2,
            width: help_width,
            height: help_height,
        };

        Clear.render(help_area, buf);

        let help_text = vec![
            Line::from(Span::styled(
                "⌨️  Keyboard Shortcuts",
                self.theme.style_accent_bold(),
            )),
            Line::from(""),
            Line::from(Span::styled("Navigation", self.theme.style_dim())),
            Line::from(vec![
                Span::styled("  Tab        ", self.theme.style_accent()),
                Span::raw("Next panel"),
            ]),
            Line::from(vec![
                Span::styled("  Shift+Tab  ", self.theme.style_accent()),
                Span::raw("Previous panel"),
            ]),
            Line::from(vec![
                Span::styled("  ↑/↓  j/k   ", self.theme.style_accent()),
                Span::raw("Navigate list / Scroll inspector"),
            ]),
            Line::from(vec![
                Span::styled("  Enter/→/l  ", self.theme.style_accent()),
                Span::raw("View item details"),
            ]),
            Line::from(vec![
                Span::styled("  ←/h        ", self.theme.style_accent()),
                Span::raw("Back to list"),
            ]),
            Line::from(vec![
                Span::styled("  g/G        ", self.theme.style_accent()),
                Span::raw("First/Last item"),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/PgDn  ", self.theme.style_accent()),
                Span::raw("Page up/down"),
            ]),
            Line::from(""),
            Line::from(Span::styled("Search & Tabs", self.theme.style_dim())),
            Line::from(vec![
                Span::styled("  /          ", self.theme.style_accent()),
                Span::raw("Focus search"),
            ]),
            Line::from(vec![
                Span::styled("  1-4        ", self.theme.style_accent()),
                Span::raw("Switch to tab (Types/Fn/Mod/Crates)"),
            ]),
            Line::from(vec![
                Span::styled("  Esc        ", self.theme.style_accent()),
                Span::raw("Clear search / Close popup / Exit"),
            ]),
            Line::from(""),
            Line::from(Span::styled("Other", self.theme.style_dim())),
            Line::from(vec![
                Span::styled("  q          ", self.theme.style_accent()),
                Span::raw("Quit"),
            ]),
            Line::from(vec![
                Span::styled("  ?          ", self.theme.style_accent()),
                Span::raw("Toggle this help"),
            ]),
            Line::from(vec![
                Span::styled("  t          ", self.theme.style_accent()),
                Span::raw("Cycle theme"),
            ]),
            Line::from(vec![
                Span::styled("  S          ", self.theme.style_accent()),
                Span::raw("Settings (theme)"),
            ]),
            Line::from(vec![
                Span::styled("  o / O      ", self.theme.style_accent()),
                Span::raw("Open crate in browser (Crates): [o] docs.rs  [c] crates.io"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to close",
                self.theme.style_muted(),
            )),
        ];

        let help = Paragraph::new(help_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.style_border_focused())
                .title(" Help ")
                .style(Style::default().bg(self.theme.bg_panel)),
        );

        help.render(help_area, buf);
    }
}

impl Widget for OracleUi<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header_height = if area.height >= 30 { 6 } else { 2 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        self.render_header(chunks[0], buf);
        self.render_tabs(chunks[1], buf);
        self.render_search(chunks[2], buf);

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ])
            .margin(0)
            .split(chunks[3]);

        self.render_list(content_chunks[0], buf);
        self.render_inspector(content_chunks[1], buf);
        self.render_status(chunks[4], buf);

        self.render_completion(chunks[2], buf);
        self.render_settings_overlay(area, buf);
        self.render_help_overlay(area, buf);
    }
}
