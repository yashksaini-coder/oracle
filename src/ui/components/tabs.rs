//! Tab bar component

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Tabs, Widget},
};

use crate::ui::theme::Theme;

/// A tab bar widget
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
        let selected_style = Style::default()
            .fg(self.theme.accent)
            .bg(self.theme.bg_highlight)
            .add_modifier(Modifier::BOLD);

        let tabs = Tabs::new(self.titles)
            .select(self.selected)
            .style(self.theme.style_dim())
            .highlight_style(selected_style)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(self.theme.style_border()),
            )
            .divider(" │ ");

        tabs.render(area, buf);
    }
}
