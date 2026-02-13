//! Selectable list component

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::ui::theme::Theme;

/// A selectable list widget with fuzzy highlight support
pub struct SelectableList<'a> {
    items: Vec<ListItem<'a>>,
    title: Option<&'a str>,
    theme: &'a Theme,
    highlight_style: Style,
    border_style: Style,
}

impl<'a> SelectableList<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            items: Vec::new(),
            title: None,
            theme,
            highlight_style: theme.style_selected(),
            border_style: theme.style_border(),
        }
    }

    pub fn items<I, T>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<ListItem<'a>>,
    {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        if focused {
            self.border_style = self.theme.style_border_focused();
        }
        self
    }

    #[allow(dead_code)]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
}

impl<'a> StatefulWidget for SelectableList<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style);

        if let Some(title) = self.title {
            block = block.title(format!(" {} ", title));
        }

        let list = List::new(self.items)
            .block(block)
            .highlight_style(self.highlight_style)
            .highlight_symbol("▸ ");

        StatefulWidget::render(list, area, buf, state);
    }
}

/// Create a line with highlighted matching characters for fuzzy search
#[allow(dead_code)]
pub fn highlight_fuzzy_match<'a>(text: &'a str, query: &str, theme: &Theme) -> Line<'a> {
    if query.is_empty() {
        return Line::from(text.to_string());
    }

    let lower_query = query.to_lowercase();
    let mut spans = Vec::new();
    let mut last_end = 0;

    // Find all matching character positions
    let mut query_chars = lower_query.chars().peekable();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut match_indices = Vec::new();

    for (i, c) in &chars {
        if let Some(&qc) = query_chars.peek() {
            if c.to_lowercase().next() == qc.to_lowercase().next() {
                match_indices.push(*i);
                query_chars.next();
            }
        }
    }

    // Build spans with highlights
    for idx in match_indices {
        if idx > last_end {
            spans.push(Span::raw(text[last_end..idx].to_string()));
        }
        let char_len = text[idx..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        spans.push(Span::styled(
            text[idx..idx + char_len].to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
        last_end = idx + char_len;
    }

    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }

    Line::from(spans)
}

/// Create a line with substring match highlighted
#[allow(dead_code)]
pub fn highlight_substring_match<'a>(text: &'a str, query: &str, theme: &Theme) -> Line<'a> {
    if query.is_empty() {
        return Line::from(text.to_string());
    }

    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();

    if let Some(start) = lower_text.find(&lower_query) {
        let end = start + query.len();
        Line::from(vec![
            Span::raw(text[..start].to_string()),
            Span::styled(
                text[start..end].to_string(),
                Style::default()
                    .bg(theme.accent)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(text[end..].to_string()),
        ])
    } else {
        Line::from(text.to_string())
    }
}
