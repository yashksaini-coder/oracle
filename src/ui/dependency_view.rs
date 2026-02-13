//! Dependency tree view

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::analyzer::{CrateInfo, DependencyKind};
use crate::ui::theme::Theme;

/// View for displaying dependency information
pub struct DependencyView<'a> {
    crate_info: Option<&'a CrateInfo>,
    dependency_tree: &'a [(String, usize)],
    theme: &'a Theme,
    focused: bool,
}

impl<'a> DependencyView<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            crate_info: None,
            dependency_tree: &[],
            theme,
            focused: false,
        }
    }

    pub fn crate_info(mut self, info: Option<&'a CrateInfo>) -> Self {
        self.crate_info = info;
        self
    }

    pub fn dependency_tree(mut self, tree: &'a [(String, usize)]) -> Self {
        self.dependency_tree = tree;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    fn render_empty(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.style_border())
            .title(" Dependencies ");

        let inner = block.inner(area);
        block.render(area, buf);

        let text = Paragraph::new("No crate selected")
            .style(self.theme.style_muted());
        text.render(inner, buf);
    }

    fn render_crate_info(&self, info: &CrateInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        // Header
        lines.push(Line::from(vec![
            Span::styled(
                &info.name,
                self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED),
            ),
            Span::raw(" "),
            Span::styled(
                format!("v{}", info.version),
                self.theme.style_dim(),
            ),
        ]));
        lines.push(Line::from(""));

        // Description
        if let Some(ref desc) = info.description {
            lines.push(Line::from(Span::styled(desc.clone(), self.theme.style_normal())));
            lines.push(Line::from(""));
        }

        // Metadata
        if let Some(ref license) = info.license {
            lines.push(Line::from(vec![
                Span::styled("License: ", self.theme.style_dim()),
                Span::raw(license.clone()),
            ]));
        }

        if !info.authors.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Authors: ", self.theme.style_dim()),
                Span::raw(info.authors.join(", ")),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("Edition: ", self.theme.style_dim()),
            Span::raw(&info.edition),
        ]));

        if let Some(ref rust_ver) = info.rust_version {
            lines.push(Line::from(vec![
                Span::styled("MSRV: ", self.theme.style_dim()),
                Span::raw(rust_ver.clone()),
            ]));
        }

        // Links
        lines.push(Line::from(""));
        if let Some(ref repo) = info.repository {
            lines.push(Line::from(vec![
                Span::styled("Repository: ", self.theme.style_dim()),
                Span::styled(repo.clone(), self.theme.style_accent()),
            ]));
        }
        if let Some(ref docs) = info.documentation {
            lines.push(Line::from(vec![
                Span::styled("Documentation: ", self.theme.style_dim()),
                Span::styled(docs.clone(), self.theme.style_accent()),
            ]));
        }

        // Features
        if !info.features.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("Features ({}):", info.features.len()),
                self.theme.style_dim(),
            )));

            for feature in info.features.iter().take(10) {
                let is_default = info.default_features.contains(feature);
                let marker = if is_default { " [default]" } else { "" };
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(feature.clone(), self.theme.style_string()),
                    Span::styled(marker, self.theme.style_muted()),
                ]));
            }

            if info.features.len() > 10 {
                lines.push(Line::from(Span::styled(
                    format!("  ... and {} more", info.features.len() - 10),
                    self.theme.style_muted(),
                )));
            }
        }

        // Dependencies summary
        lines.push(Line::from(""));
        let normal_deps = info.dependencies.iter().filter(|d| d.kind == DependencyKind::Normal).count();
        let dev_deps = info.dependencies.iter().filter(|d| d.kind == DependencyKind::Dev).count();
        let build_deps = info.dependencies.iter().filter(|d| d.kind == DependencyKind::Build).count();

        lines.push(Line::from(Span::styled("Dependencies:", self.theme.style_dim())));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{}", normal_deps), self.theme.style_accent()),
            Span::raw(" normal, "),
            Span::styled(format!("{}", dev_deps), self.theme.style_accent()),
            Span::raw(" dev, "),
            Span::styled(format!("{}", build_deps), self.theme.style_accent()),
            Span::raw(" build"),
        ]));

        // List direct dependencies
        lines.push(Line::from(""));
        for dep in info.dependencies.iter().filter(|d| d.kind == DependencyKind::Normal).take(15) {
            let optional = if dep.optional { " (optional)" } else { "" };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&dep.name, self.theme.style_type()),
                Span::styled(format!(" {}", dep.version), self.theme.style_muted()),
                Span::styled(optional, self.theme.style_dim()),
            ]));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.focused {
                self.theme.style_border_focused()
            } else {
                self.theme.style_border()
            })
            .title(" Dependencies ");

        let inner = block.inner(area);
        block.render(area, buf);

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(inner, buf);
    }
}

impl Widget for DependencyView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.crate_info {
            Some(info) => self.render_crate_info(info, area, buf),
            None => self.render_empty(area, buf),
        }
    }
}
