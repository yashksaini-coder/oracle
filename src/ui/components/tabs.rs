//! Tab bar component

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Tabs, Widget},
};

use crate::ui::theme::Theme;

/// A tab bar widget
pub struct TabBar<'a> {
    titles: Vec<&'a str>,
    selected: usize,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> TabBar<'a> {
    pub fn new(titles: Vec<&'a str>, theme: &'a Theme) -> Self {
        Self {
            titles,
            selected: 0,
            theme,
            focused: false,
        }
    }

    pub fn select(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected_style = self.theme.style_tab_active();
        let inactive_style = self.theme.style_dim();

        let border_style = if self.focused {
            self.theme.style_border_focused()
        } else {
            self.theme.style_border()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(self.theme.bg_panel))
            .title(" Tabs ");

        let tabs = Tabs::new(self.titles)
            .select(self.selected)
            .style(inactive_style)
            .highlight_style(selected_style)
            .block(block)
            .divider(" │ ")
            .padding("    ", "    ");

        tabs.render(area, buf);
    }
}
