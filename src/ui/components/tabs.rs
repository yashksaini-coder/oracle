//! Tab bar component with cyberpunk styling

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::ui::theme::Theme;

/// A cyberpunk-styled tab bar widget
pub struct TabBar<'a> {
    titles: Vec<&'a str>,
    selected: usize,
    theme: &'a Theme,
}

impl<'a> TabBar<'a> {
    pub fn new(titles: Vec<&'a str>, theme: &'a Theme) -> Self {
        Self {
            titles,
            selected: 0,
            theme,
        }
    }

    pub fn select(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{Paragraph, Wrap};

        // Build custom tab bar with cyberpunk styling
        let mut line_spans = vec![];
        
        line_spans.push(Span::styled("┏", self.theme.style_border()));
        line_spans.push(Span::styled("━", self.theme.style_border()));
        
        for (i, title) in self.titles.iter().enumerate() {
            if i > 0 {
                line_spans.push(Span::styled(" ┃ ", self.theme.style_border()));
            }
            
            if i == self.selected {
                // Active tab: bright neon with bold
                line_spans.push(Span::styled(
                    format!(" [ {} ] ", title),
                    self.theme.style_accent_bold().add_modifier(Modifier::BOLD),
                ));
                // Add focused border indicator
                line_spans.push(Span::styled("▣", self.theme.style_border_focused()));
            } else {
                // Inactive tab: dimmed
                line_spans.push(Span::styled(
                    format!(" {} ", title),
                    self.theme.style_muted(),
                ));
            }
        }
        
        line_spans.push(Span::styled(" ━", self.theme.style_border()));
        line_spans.push(Span::styled("┓", self.theme.style_border()));
        
        let tab_line = Line::from(line_spans);
        let tab_widget = Paragraph::new(tab_line)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(self.theme.style_border()),
            )
            .wrap(Wrap { trim: true });

        tab_widget.render(area, buf);
    }
}
