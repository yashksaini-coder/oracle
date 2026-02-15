//! Right panel: vertical divider, inspector (item details / dependency view / crate info).

use crate::ui::dependency_view::{self, DependencyDocView, DependencyView};
use crate::ui::inspector::InspectorPanel;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{block::BorderType, Block, Borders, Paragraph, Widget},
};

use super::types::{Focus, Tab};
use super::OracleUi;

impl<'a> OracleUi<'a> {
    pub(super) fn render_vertical_divider(&self, area: Rect, buf: &mut Buffer) {
        let style = self.theme.style_border();
        let symbol = "│";
        for y in area.top()..area.bottom() {
            if area.width > 0 {
                if let Some(cell) = buf.cell_mut((area.x, y)) {
                    cell.set_symbol(symbol).set_style(style);
                }
            }
        }
    }

    pub(super) fn render_inspector(&self, area: Rect, buf: &mut Buffer) {
        if self.current_tab == Tab::Crates && self.selected_installed_crate.is_some() {
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
            let showing_root = root_name
                .zip(selected_name)
                .map(|(r, s)| r == s)
                .unwrap_or(true);
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

    pub(super) fn render_installed_crate_info(&self, area: Rect, buf: &mut Buffer) {
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
                Span::styled(
                    &crate_info.name,
                    self.theme
                        .style_accent_bold()
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::raw(" "),
                Span::styled(format!("v{}", crate_info.version), self.theme.style_muted()),
            ]),
            Line::from(""),
        ];
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
        if let Some(ref license) = crate_info.license {
            lines.push(Line::from(vec![
                Span::styled("  License: ", self.theme.style_dim()),
                Span::styled(license.clone(), self.theme.style_normal()),
            ]));
        }
        if !crate_info.authors.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  Authors: ", self.theme.style_dim()),
                Span::styled(crate_info.authors.join(", "), self.theme.style_normal()),
            ]));
        }
        if let Some(ref repo) = crate_info.repository {
            lines.push(Line::from(vec![
                Span::styled("  Repository: ", self.theme.style_dim()),
                Span::styled(repo.clone(), self.theme.style_accent()),
            ]));
        }
        if let Some(ref docs) = crate_info.documentation {
            lines.push(Line::from(vec![
                Span::styled("  Docs: ", self.theme.style_dim()),
                Span::styled(docs.clone(), self.theme.style_accent()),
            ]));
        }
        if !crate_info.keywords.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("━━━ ", self.theme.style_muted()),
                Span::styled("Keywords", self.theme.style_accent()),
                Span::styled(" ━━━", self.theme.style_muted()),
            ]));
            lines.push(Line::from(""));
            let keywords: Vec<Span> = crate_info
                .keywords
                .iter()
                .map(|k| Span::styled(format!(" {} ", k), self.theme.style_keyword()))
                .collect();
            lines.push(Line::from(vec![Span::raw("  ")]).patch_style(Style::default()));
            for kw in keywords {
                lines.push(Line::from(vec![Span::raw("  "), kw]));
            }
        }
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
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("━━━ ", self.theme.style_muted()),
            Span::styled("Location", self.theme.style_accent()),
            Span::styled(" ━━━", self.theme.style_muted()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  📁 "),
            Span::styled(
                crate_info.path.display().to_string(),
                self.theme.style_muted(),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("━━━ ", self.theme.style_muted()),
            Span::styled("Analysis", self.theme.style_accent()),
            Span::styled(" ━━━", self.theme.style_muted()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} items found", self.installed_crate_items.len()),
                self.theme.style_normal(),
            ),
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
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(" ◇ Crate Info ");
        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false });
        paragraph.render(area, buf);
    }
}
