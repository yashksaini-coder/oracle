//! Header block: ORACLE logo + live metrics (items, crates, target size, creator).

use crate::utils::format_bytes;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use super::OracleUi;

const ORACLE_ART: [&str; 6] = [
    " ██████╗ ██████╗  █████╗  ██████╗██╗     ███████╗",
    "██╔═══██╗██╔══██╗██╔══██╗██╔════╝██║     ██╔════╝",
    "██║   ██║██████╔╝███████║██║     ██║     █████╗  ",
    "██║   ██║██╔══██╗██╔══██║██║     ██║     ██╔══╝  ",
    "╚██████╔╝██║  ██║██║  ██║╚██████╗███████╗███████╗",
    " ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚══════╝",
];

impl<'a> OracleUi<'a> {
    /// Renders the header: left = ASCII art ORACLE logo, right = live metrics.
    pub(super) fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let (fn_count, struct_count, enum_count, trait_count, mod_count) = self.items.iter().fold(
            (0usize, 0usize, 0usize, 0usize, 0usize),
            |(f, s, e, t, m), item| match item.kind() {
                "fn" => (f + 1, s, e, t, m),
                "struct" => (f, s + 1, e, t, m),
                "enum" => (f, s, e + 1, t, m),
                "trait" => (f, s, e, t + 1, m),
                "mod" => (f, s, e, t, m + 1),
                _ => (f, s, e, t, m),
            },
        );
        let types_count = struct_count + enum_count + trait_count;
        let line1 = format!("📦 {} types · {} fns · {} mods", types_count, fn_count, mod_count);
        let crates_count = self.dependency_tree.len();
        let line2 = if let Some(bytes) = self.target_size_bytes {
            format!("📚 {} crates · target {}", crates_count, format_bytes(bytes))
        } else {
            format!("📚 {} crates", crates_count)
        };
        let line3 = "👤 created by yashksaini-coder";

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Min(30)])
            .split(area);
        let logo_area = header_chunks[0];
        let tagline_area = header_chunks[1];
        let logo_lines: Vec<Line> = ORACLE_ART
            .iter()
            .take(logo_area.height as usize)
            .map(|s| Line::from(Span::styled(*s, self.theme.style_accent())))
            .collect();
        Paragraph::new(logo_lines).render(logo_area, buf);

        let row_height = tagline_area.height / 3;
        let tagline_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(row_height),
                Constraint::Length(row_height),
                Constraint::Length(tagline_area.height.saturating_sub(2 * row_height)),
            ])
            .split(tagline_area);

        let lines_content = [line1, line2, line3.to_string()];
        for (i, content) in lines_content.iter().enumerate() {
            if let Some(rect) = tagline_rows.get(i) {
                let line = Line::from(Span::styled(content.as_str(), self.theme.style_dim()));
                Paragraph::new(line)
                    .alignment(Alignment::Right)
                    .render(*rect, buf);
            }
        }
    }
}
